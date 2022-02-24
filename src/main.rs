use itertools::izip;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::vec;

fn load_words(file: String) -> Vec<String> {
    let mut words = vec![];
    let file = File::open(file).expect("coulnd't find word list");
    let rdr = BufReader::new(file);
    for line in rdr.lines() {
        if let Ok(word) = line {
            words.push(word.to_string());
        } else {
            panic!("error parsing word list")
        }
    }
    words
}

#[derive(Clone)]
struct LetterFreq {
    positions: Vec<Vec<usize>>,
    totals: Vec<usize>,
    max_positions: Vec<usize>,
    max_totals: usize,
}

impl LetterFreq {
    fn new() -> LetterFreq {
        LetterFreq {
            positions: vec![vec![0; 26]; 5],
            totals: vec![0; 26],
            max_positions: vec![0; 5],
            max_totals: 0,
        }
    }

    fn inc(&mut self, pos: usize, letter: char) {
        self.positions[pos][letter as usize - 'a' as usize] += 1;
        self.totals[letter as usize - 'a' as usize] += 1;
    }

    fn calc_maxs(&mut self) {
        for (pos_freqs, max_freq) in izip!(&self.positions, &mut self.max_positions) {
            *max_freq = *pos_freqs.iter().max().unwrap();
        }
        self.max_totals = *self.totals.iter().max().unwrap();
    }

    fn get_pos_count(&self, pos: usize, letter: char) -> usize {
        self.positions[pos][letter as usize - 'a' as usize]
    }

    fn get_total_count(&self, letter: char) -> usize {
        self.totals[letter as usize - 'a' as usize]
    }

    fn get_pos_max(&self, pos: usize) -> usize {
        self.max_positions[pos]
    }

    fn get_total_max(&self) -> usize {
        self.max_totals
    }
}

fn get_letter_freqs(words: &Vec<String>) -> LetterFreq {
    let mut ret: LetterFreq = LetterFreq::new();
    for word in words {
        for i in 0..5 {
            let c = word.chars().nth(i).unwrap();
            ret.inc(i, c);
        }
    }
    ret.calc_maxs();
    ret
}

#[derive(Clone)]
enum PositionInfo {
    None,
    Not(Vec<char>),
    Is(char),
}

#[derive(Clone, Copy)]
struct GeneralInfo {
    exactly: bool,
    count: usize,
}

impl GeneralInfo {
    fn new() -> GeneralInfo {
        GeneralInfo {
            exactly: false,
            count: 0,
        }
    }

    fn is_enough(&self, found_count: usize) -> bool {
        if self.exactly {
            if found_count == self.count {
                return true;
            }
        } else {
            if found_count >= self.count {
                return true;
            }
        }
        false
    }
}

/*
Positional info:
- grey/yellow: this character will not be here
- green: this character will be here
General info:
- for a character on a guess:
    - green/yellow + no grey: at least this number of this char
    - green/yellow + grey: exactly this many of this char
 */
struct KnownInfo {
    pos_info: Vec<PositionInfo>,
    count: Vec<GeneralInfo>,
}

impl KnownInfo {
    fn new() -> KnownInfo {
        KnownInfo {
            pos_info: vec![PositionInfo::None; 5],
            count: vec![GeneralInfo::new(); 26],
        }
    }

    fn get_count(&self, letter: char) -> GeneralInfo {
        self.count[letter as usize - 'a' as usize]
    }

    fn set_count(&mut self, letter: char, amount: usize, exactly: bool) {
        self.count[letter as usize - 'a' as usize].count = amount;
        self.count[letter as usize - 'a' as usize].exactly = exactly;
    }
}

fn get_possible_words(info: &KnownInfo, words: &Vec<String>) -> Vec<String> {
    let mut possible_words = vec![];
    'word_loop: for word in words {
        for (i, c) in word.chars().enumerate() {
            // check the positional info to see if there are any contradictions
            match &info.pos_info[i] {
                PositionInfo::None => {}
                PositionInfo::Not(not_chars) => {
                    if not_chars.contains(&c) {
                        continue 'word_loop;
                    }
                }
                PositionInfo::Is(is_char) => {
                    if *is_char != c {
                        continue 'word_loop;
                    }
                }
            }
        }
        // check the general info to see if there is enough of each letter
        for c in 'a'..'z' {
            let found_count = word.chars().filter(|ch| *ch == c).count();
            if !info.get_count(c).is_enough(found_count) {
                continue 'word_loop;
            }
        }
        possible_words.push((*word).clone());
    }
    possible_words
}

fn get_scores(words: &Vec<String>, freqs: &LetterFreq, info: &KnownInfo) -> Vec<usize> {
    let mut scores: Vec<usize> = vec![];
    for word in words {
        let mut score_pos: usize = 0;
        for (pos, c) in word.chars().enumerate() {
            score_pos += freqs.get_pos_count(pos, c);
        }

        let mut score_total: usize = 0;
        for c in word.chars() {
            score_total += freqs.get_total_count(c);
        }

        let score = score_pos + score_total * freqs.get_pos_max(0) / freqs.get_total_max() / 3;
        // let score = score_pos;
        scores.push(score as usize);
    }
    scores
}

fn get_best_word(words: &Vec<String>, scores: &Vec<usize>) -> String {
    let mut best_word: &String = &String::new();
    let mut best_score = 0;
    for (word, score) in izip!(words, scores) {
        if *score > best_score {
            best_word = word;
            best_score = *score;
        }
    }
    best_word.clone()
}

// make a guess and get back the known info from that guess
fn make_guess(guess: &String, target: &String, info: &mut KnownInfo) {
    // update positional info
    for (guess_char, target_char, pos_info) in
        izip!(guess.chars(), target.chars(), info.pos_info.iter_mut())
    {
        if guess_char == target_char {
            *pos_info = PositionInfo::Is(guess_char);
        } else {
            match pos_info {
                PositionInfo::None => *pos_info = PositionInfo::Not(vec![guess_char]),
                PositionInfo::Not(not_char) => not_char.push(guess_char),
                _ => panic!("invalid pos_info"),
            }
        }
    }
    // update general info
    for guess_char in guess.chars() {
        let guess_count = guess.chars().filter(|ch| *ch == guess_char).count();
        let target_count = target.chars().filter(|ch| *ch == guess_char).count();
        if guess_count <= target_count {
            info.set_count(guess_char, guess_count, false);
        } else {
            info.set_count(guess_char, target_count, true);
        }
    }
}

fn find_word(target_word: &String, words: &Vec<String>) -> usize {
    if target_word.len() != 5 {
        println!("not 5 letter word, try again");
        return 0;
    }
    if !words.contains(&target_word) {
        println!("word not in dictionary, try again");
        return 0;
    }

    let mut words = words.clone();
    let mut info = KnownInfo::new();
    let mut count = 0;
    loop {
        let freqs = get_letter_freqs(&words);
        let scores = get_scores(&words, &freqs, &info);
        let best_word = get_best_word(&words, &scores);
        println!("guessing: {}", best_word);
        count += 1;
        if best_word == *target_word {
            return count;
        } else {
            make_guess(&best_word, &target_word, &mut info);
            words = get_possible_words(&info, &words);
            if words.len() == 0 {
                println!("huh")
            }
        }
    }
}

fn main() {
    let mut words = load_words("wordle-answers-alphabetical.txt".to_string());

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    find_word(&"angry".to_string(), &words);

    // loop {
    //     let mut target_word = String::new();
    //     print!("word to guess: ");
    //     stdout.flush().unwrap();
    //     stdin.lock().read_line(&mut target_word).unwrap();
    //     target_word = target_word.trim().to_string();
    //     guess(&target_word, &words, &scores);
    // }

    let mut guess_counts = 0;
    let mut bins = vec![0; 10];
    for (i, word) in words.iter().enumerate() {
        // println!("guessing word {}/{}: {}", i, words.len(), word);
        let attempts = find_word(&word, &words);
        guess_counts += attempts;
        bins[attempts] += 1;
    }
    println!(
        "average tries to guess was: {}",
        guess_counts as f64 / words.len() as f64
    );
    println!("bins: {:?}", bins);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_scoring() {
        use crate::*;
    }
    #[test]
    fn get_possible_words() {
        use crate::*;

        let words = load_words("wordle-answers-alphabetical.txt".to_string());
        let mut info = KnownInfo::new();
        info.pos_info = vec![
            PositionInfo::None,
            PositionInfo::Not(vec!['e']),
            PositionInfo::None,
            PositionInfo::Is('e'),
            PositionInfo::Is('r'),
        ];
        info.not_contains = vec![
            't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'f', 'g', 'h', 'c', 'n', 'm',
        ];

        let pos_words = get_possible_words(info, &words);
        assert_eq!(pos_words, vec!["elder"]);
    }
}

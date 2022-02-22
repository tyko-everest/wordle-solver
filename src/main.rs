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

#[derive(Clone, PartialEq)]
enum PosInfo {
    None,
    Not(Vec<char>),
    Is(char),
}

struct KnownInfo {
    pos_info: Vec<PosInfo>,
    not_contains: Vec<char>,
    contains: Vec<char>,
}

impl KnownInfo {
    fn new() -> KnownInfo {
        KnownInfo {
            pos_info: vec![PosInfo::None; 5],
            not_contains: vec![],
            contains: vec![],
        }
    }
}

fn get_possible_words(info: &KnownInfo, words: &Vec<String>) -> Vec<String> {
    let mut possible_words = vec![];
    'word_loop: for word in words {
        for (i, c) in word.chars().enumerate() {
            if info.not_contains.contains(&c) {
                continue 'word_loop;
            }
            match &info.pos_info[i] {
                PosInfo::None => {}
                PosInfo::Not(not_chars) => {
                    if not_chars.contains(&c) {
                        continue 'word_loop;
                    }
                }
                PosInfo::Is(is_char) => {
                    if *is_char != c {
                        continue 'word_loop;
                    }
                }
            }
        }
        if !info.contains.iter().all(|c| word.contains(*c)) {
            continue 'word_loop;
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
            let mut score_temp = freqs.get_total_count(c);
            if info.contains.contains(&c) {
                score_temp /= 2;
            }
            score_total += score_temp;
        }

        // let score = score_pos + score_total * freqs.get_pos_max(0) / freqs.get_total_max() / 2;
        let score = score_pos;
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
    for (guess_char, target_char, pos_info) in
        izip!(guess.chars(), target.chars(), info.pos_info.iter_mut())
    {
        if target.contains(guess_char) {
            // TODO ensure no dups
            info.contains.push(guess_char);
            if guess_char == target_char {
                *pos_info = PosInfo::Is(guess_char);
            } else {
                match pos_info {
                    PosInfo::None => *pos_info = PosInfo::Not(vec![guess_char]),
                    PosInfo::Not(not_char) => not_char.push(guess_char),
                    _ => panic!("invalid pos_info"),
                }
            }
        } else {
            // TODO ensure no dups
            info.not_contains.push(guess_char);
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
        // println!("guessing: {}", best_word);
        count += 1;
        if best_word == *target_word {
            return count;
        } else {
            make_guess(&best_word, &target_word, &mut info);
            words = get_possible_words(&info, &words);
        }
    }
}

fn main() {
    let mut words = load_words("wordle-answers-alphabetical.txt".to_string());

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // loop {
    //     let mut target_word = String::new();
    //     print!("word to guess: ");
    //     stdout.flush().unwrap();
    //     stdin.lock().read_line(&mut target_word).unwrap();
    //     target_word = target_word.trim().to_string();
    //     guess(&target_word, &words, &scores);
    // }

    let mut guess_counts = 0;
    for (i, word) in words.iter().enumerate() {
        // println!("guessing word {}/{}: {}", i, words.len(), word);
        guess_counts += find_word(&word, &words);
    }
    println!(
        "average tries to guess was: {}",
        guess_counts as f64 / words.len() as f64
    );
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
            PosInfo::None,
            PosInfo::Not(vec!['e']),
            PosInfo::None,
            PosInfo::Is('e'),
            PosInfo::Is('r'),
        ];
        info.not_contains = vec![
            't', 'y', 'u', 'i', 'o', 'p', 'a', 's', 'f', 'g', 'h', 'c', 'n', 'm',
        ];

        let pos_words = get_possible_words(info, &words);
        assert_eq!(pos_words, vec!["elder"]);
    }
}

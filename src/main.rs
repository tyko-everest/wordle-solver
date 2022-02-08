use itertools::izip;
use std::io::{self, BufRead, Write};
use std::vec;

fn load_words(file: String) -> (Vec<String>, Vec<f32>) {
    let mut words = vec![];
    let mut freqs = vec![];
    let mut rdr = csv::Reader::from_path(file).unwrap();
    for res in rdr.records() {
        let record = res.unwrap();
        let word = record.get(0).unwrap();
        let freq: f32 = record.get(1).unwrap().parse().unwrap();
        words.push(word.to_string());
        freqs.push(freq);
    }
    (words, freqs)
}

#[derive(Clone)]
struct LetterFreq {
    counts: Vec<usize>,
}

impl LetterFreq {
    fn new() -> LetterFreq {
        LetterFreq {
            counts: vec![0; 26],
        }
    }

    fn inc(&mut self, letter: char) {
        self.counts[letter as usize - 'a' as usize] += 1;
    }

    fn get_count(&self, letter: char) -> usize {
        self.counts[letter as usize - 'a' as usize]
    }

    fn get_highest(&self) -> char {
        let mut max_cnt = 0;
        let mut max_i = 0;
        for (i, cnt) in self.counts.iter().enumerate() {
            if *cnt > max_cnt {
                max_cnt = *cnt;
                max_i = i;
            }
        }
        ((max_i as u8) + 'a' as u8) as char
    }
}

fn get_letter_freqs(words: &Vec<String>) -> Vec<LetterFreq> {
    let mut ret: Vec<LetterFreq> = vec![LetterFreq::new(); 5];
    for word in words {
        for i in 0..5 {
            let c = word.chars().nth(i).unwrap();
            ret[i].inc(c);
        }
    }
    ret
}

#[derive(Clone)]
struct WordInfo {
    word: String,
    score: usize,
}

#[derive(Clone)]
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

fn get_possible_words(info: &KnownInfo, words: &Vec<WordInfo>) -> Vec<WordInfo> {
    let mut pos_word_infos = vec![];
    'word_loop: for word_info in words {
        for (i, c) in word_info.word.chars().enumerate() {
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
        if !info.contains.iter().all(|c| word_info.word.contains(*c)) {
            continue 'word_loop;
        }
        pos_word_infos.push((*word_info).clone());
    }
    pos_word_infos
}

fn get_scores(
    words: &Vec<String>,
    pos_freqs: &Vec<LetterFreq>,
    word_freqs: &Vec<f32>,
) -> Vec<usize> {
    let mut scores: Vec<usize> = vec![];
    for (word, word_freq) in izip!(words, word_freqs) {
        let mut score: usize = 0;
        for (pos, c) in word.chars().enumerate() {
            score += pos_freqs[pos].get_count(c);
        }
        scores.push(score * ((*word_freq * 1000.0) as usize));
    }
    scores
}

fn get_best_word(word_infos: &mut Vec<WordInfo>) -> String {
    let mut best_word = WordInfo {
        word: String::new(),
        score: 0,
    };
    for word_info in word_infos {
        if word_info.score > best_word.score {
            best_word.word = word_info.word.clone();
            best_word.score = word_info.score;
        }
    }
    best_word.word.clone()
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

fn guess(target_word: &String, words: &Vec<String>, scores: &Vec<usize>) {
    let mut word_infos: Vec<WordInfo> = vec![];
    for (word, score) in izip!(words, scores) {
        word_infos.push(WordInfo {
            word: word.clone(),
            score: *score,
        });
    }
    let mut info = KnownInfo::new();
    if target_word.len() != 5 {
        println!("not 5 letter word, try again");
        return;
    }
    if !words.contains(&target_word) {
        println!("word not in dictionary, try again");
        return;
    }
    loop {
        let best_word = get_best_word(&mut word_infos);
        println!("guessing: {}", best_word);
        if best_word == *target_word {
            break;
        } else {
            make_guess(&best_word, &target_word, &mut info);
            word_infos = get_possible_words(&info, &word_infos);
        }
    }
}

fn main() {
    let (words, word_freqs) = load_words("5_letters.csv".to_string());
    let pos_freqs = get_letter_freqs(&words);
    let scores = get_scores(&words, &pos_freqs, &word_freqs);
    let mut full_word_infos: Vec<WordInfo> = vec![];
    for (word, score) in izip!(&words, &scores) {
        full_word_infos.push(WordInfo {
            word: word.clone(),
            score: *score,
        });
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        let mut target_word = String::new();
        print!("word to guess: ");
        stdout.flush().unwrap();
        stdin.lock().read_line(&mut target_word).unwrap();
        target_word = target_word.trim().to_string();
        guess(&target_word, &words, &scores);
    }
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

        let words = load_words("5_letters.csv".to_string());
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

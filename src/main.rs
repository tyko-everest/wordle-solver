use itertools::izip;
use std::io;

fn load_words(file: String) -> Vec<String> {
    let mut words = vec![];
    let mut rdr = csv::Reader::from_path(file).unwrap();
    for res in rdr.records() {
        let record = res.unwrap();
        let word = record.get(0).unwrap();
        words.push(word.to_string());
    }
    words
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

struct WordInfo {
    word: String,
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

fn get_possible_words(info: &KnownInfo, words: &Vec<String>) -> Vec<String> {
    let mut pos_words = vec![];
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
        pos_words.push(word.clone());
    }
    pos_words
}

fn get_best_word(words: &Vec<String>, pos_freqs: &Vec<LetterFreq>) -> String {
    let mut scores: Vec<usize> = vec![];
    for word in words {
        let mut score: usize = 0;
        for (pos, c) in word.chars().enumerate() {
            score += pos_freqs[pos].get_count(c);
        }
        scores.push(score);
    }

    let mut max_i = 0;
    let mut max_val = 0;
    for (i, val) in scores.iter().enumerate() {
        if *val > max_val {
            max_val = *val;
            max_i = i;
        }
    }
    words[max_i].clone()
}

// make a guess and get back the known info from that guess
fn guess(guess: &String, target: &String, info: &mut KnownInfo) {
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

fn main() {
    let mut words = load_words("5_letters.csv".to_string());
    let pos_freqs = get_letter_freqs(&words);

    // let mut info = KnownInfo::new();
    // info.pos_info = vec![
    //     PosInfo::None,
    //     PosInfo::None,
    //     PosInfo::None,
    //     PosInfo::Is('e'),
    //     PosInfo::None,
    // ];
    // info.not_contains = vec!['s', 'a', 'm', 'y'];
    // let words = get_possible_words(info, &words);
    // println!("pos_words: {:?}", words);

    let mut info = KnownInfo::new();
    let target_word = "primp".to_string();
    println!("target is: {}", target_word);
    loop {
        let best_word = get_best_word(&words, &pos_freqs);
        println!("guessing: {}", best_word);
        if best_word == target_word {
            break;
        } else {
            guess(&best_word, &target_word, &mut info);
            words = get_possible_words(&info, &words);
        }
    }

    println!("done");
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

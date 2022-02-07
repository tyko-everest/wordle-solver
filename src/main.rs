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
}

impl KnownInfo {
    fn new() -> KnownInfo {
        KnownInfo {
            pos_info: vec![PosInfo::None; 5],
            not_contains: vec![],
        }
    }
}

fn get_possible_words(info: KnownInfo, words: &Vec<String>) -> Vec<String> {
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
        pos_words.push(word.clone());
    }
    pos_words
}

fn main() {
    let words = load_words("5_letters.csv".to_string());

    println!("done");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_scoring() {
        use crate::*;
        let words = load_words("5_letters.csv".to_string());
        let pos_freqs = get_letter_freqs(&words);
        let mut scores: Vec<usize> = vec![];

        for word in words.iter() {
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
        println!("best score {} with word {}", max_val, words[max_i]);
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

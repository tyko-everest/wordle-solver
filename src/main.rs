fn extract_5_letter() {
    let mut rdr = csv::Reader::from_path("word_list.csv").unwrap();
    let mut wtr = csv::Writer::from_path("5_letters.csv").unwrap();
    for res in rdr.records() {
        let record = res.unwrap();
        let word = record.get(0).unwrap();
        let freq: f32 = record.get(1).unwrap().parse().unwrap();

        if word.len() == 5 {
            wtr.write_record(&record);
        }
    }
}

#[derive(Clone)]
struct LetterFreq {
    data: Vec<usize>,
}

impl LetterFreq {
    fn new() -> LetterFreq {
        LetterFreq { data: vec![0; 26] }
    }

    fn add(&mut self, letter: char) {
        self.data[letter as usize - 'a' as usize] += 1;
    }

    fn get_count(&self, letter: char) -> usize {
        self.data[letter as usize - 'a' as usize]
    }

    fn get_highest(&self) -> char {
        let mut max = 0;
        let mut index = 0;
        for (i, cnt) in self.data.iter().enumerate() {
            if *cnt > max {
                max = *cnt;
                index = i;
            }
        }
        ((index as u8) + 'a' as u8) as char
    }
}

fn get_letter_freqs(words: &Vec<String>) -> Vec<LetterFreq> {
    let mut ret: Vec<LetterFreq> = vec![LetterFreq::new(); 5];
    for word in words {
        for i in 0..5 {
            let c = word.chars().nth(i).unwrap();
            ret[i].add(c);
        }
    }
    ret
}

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

fn main() {
    let words = load_words("5_letters.csv".to_string());
    let freqs = get_letter_freqs(&words);
    let mut scores: Vec<usize> = vec![];

    for word in words.iter() {
        let mut score: usize = 1;

        for (pos, c) in word.chars().enumerate() {
            score *= freqs[pos].get_count(c);
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

    println!("done");
}

use std::io;
use std::io::BufRead;

const STANDARD_FREQUENCIES: [f64; 26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015,
    0.06094, 0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749,
    0.07507, 0.01929, 0.00095, 0.05987, 0.06327, 0.09056, 0.02758,
    0.00978, 0.02360, 0.00150, 0.01974, 0.00074
];

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    let mut buffer = String::new();
    loop {
        let bytes = reader.read_line(&mut buffer)?;
        if bytes == 0 {
            break;
        }
    }

    let (key, plaintext) = crack_vigenere(&buffer);
    println!("{}\n{}", key, plaintext);

    Ok(())
}

fn crack_vigenere(ciphertext: &str) -> (String, String) {
    let key = find_vigenere_key(ciphertext);
    let plaintext = decrypt_vigenere(ciphertext, &key);
    (key, plaintext)
}

fn find_vigenere_key(ciphertext: &str) -> String {
    let text = ciphertext.letters_only();

    let mut best_len = 1;
    let mut best_ioc = 0.0f64;

    for len in 1..=text.len().saturating_div(40) {
        let ioc = average_ioc(&text, len);

        if ioc > best_ioc {
            best_len = len;
            best_ioc = ioc;
        }
    }

    let mut key = String::with_capacity(best_len);

    for i in 0..best_len {
        let slice: String = text.chars()
            .skip(i)
            .step_by(best_len)
            .collect();
        key.push(best_caesar(&slice));
    }

    key.period()
}

fn average_ioc(text: &str, keylen: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..keylen {
        let slice: String = text.chars()
            .skip(i)
            .step_by(keylen)
            .collect();
        sum += index_of_coincidence(&slice);
    }
    sum / keylen as f64
}

fn index_of_coincidence(text: &str) -> f64 {
    let n = text.len();
    if n <= 1 { return 0.0; }

    let mut freq = [0; 26];
    text.chars()
        .for_each(|c| freq[(c as u8 - b'a') as usize] += 1);

    freq.iter()
        .map(|&f| (f * (f - 1)) as f64 / (n * (n - 1)) as f64)
        .sum()
}

fn best_caesar(text: &str) -> char {
    if text.is_empty() {
        return 'a';
    }

    let mut best_pos = 0;
    let mut best_chi = f64::INFINITY;

    for pos in 0..26 {
        let mut freq = [0.0; 26];
        text.chars().for_each(|c| freq[((c as u8 - b'a' + 26 - pos) % 26) as usize] += 1.0);

        let chi = freq.iter()
            .zip(STANDARD_FREQUENCIES.iter())
            .map(|(&x, &y)| (x / text.len() as f64 - y).powi(2) / y)
            .sum();

        if chi < best_chi {
            best_pos = pos;
            best_chi = chi;
        }
    }

    (b'A' + best_pos) as char
}

fn decrypt_vigenere(ciphertext: &str, key: &str) -> String {
    let mut key_iter = key.as_bytes().iter().cycle();
    ciphertext
        .chars()
        .map(|c| match c {
            'A'..='Z' => {
                let k = *key_iter.next().unwrap() - b'A';
                (((c as u8 - b'A' + 26 - k) % 26) + b'A') as char
            }
            'a'..='z' => {
                let k = *key_iter.next().unwrap() - b'A';
                (((c as u8 - b'a' + 26 - k) % 26) + b'a') as char
            }
            _ => c,
        })
        .collect()
}

trait StringExt {
    fn letters_only(&self) -> String;
    fn period(&self) -> String;
}
impl StringExt for str {
    fn letters_only(&self) -> String {
        self.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    }

    fn period(&self) -> String {
        let chars: Vec<char> = self.chars().collect();

        if chars.len() <= 1 {
            return self.to_string();
        }

        let n = chars.len();
        let mut next = vec![0; n];

        let mut i = 1;
        let mut j = 0;

        while i < n {
            if chars[i] == chars[j] {
                j += 1;
                next[i] = j;
                i += 1;
            } else if j > 0 {
                j = next[j - 1];
            } else {
                next[i] = 0;
                i += 1;
            }
        }

        let t = n - next[n - 1];

        if n.is_multiple_of(t) {
            self[..t].to_string()
        } else {
            self.to_string()
        }
    }
}

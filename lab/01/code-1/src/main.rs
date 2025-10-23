use std::io;
use std::io::Read;

fn main() {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    
    let (key, plaintext) = vigenere_cipher::crack(&buffer);
    println!("{}\n{}", key, plaintext);
}

mod vigenere_cipher {
    const STANDARD_FREQUENCIES: [f64; 26] = [
        0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015,
        0.06094, 0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749,
        0.07507, 0.01929, 0.00095, 0.05987, 0.06327, 0.09056, 0.02758,
        0.00978, 0.02360, 0.00150, 0.01974, 0.00074,
    ];

    pub fn crack(ciphertext: &str) -> (String, String) {
        let key = find_key(ciphertext);
        let plaintext = decrypt(ciphertext, &key);
        (key, plaintext)
    }

    fn find_key(ciphertext: &str) -> String {
        let clean_text: String = ciphertext.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_lowercase())
            .collect();

        let best_len = best_key_length(&clean_text);

        let mut key = String::with_capacity(best_len);
        for index in 0..best_len {
            let slice: String = clean_text.chars()
                .skip(index)
                .step_by(best_len)
                .collect();
            key.push(best_caesar(&slice));
        }

        key
    }

    fn best_key_length(text: &str) -> usize {
        let mut best_len = 1;
        let mut best_ioc = 0.0f64;

        for len in 1..=40 {
            let ioc = average_ioc(text, len);
            if ioc > best_ioc + 1e-2 { // add a EPS to avoid choosing peroidic keys
                best_len = len;
                best_ioc = ioc;
            }
        }

        best_len
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
        let mut frequencies = [0; 26];
        text.chars().for_each(|c| frequencies[(c as u8 - b'a') as usize] += 1);

        let n = text.len();
        frequencies.iter().map(|f| (f * (f - 1)) as f64 / (n * (n - 1)) as f64).sum()
    }

    fn best_caesar(text: &str) -> char {
        let mut best_pos = 0;
        let mut best_chi = f64::INFINITY;

        for pos in 0..26 {
            let mut frequencies = [0.0; 26];
            text.chars().for_each(|c| frequencies[((c as u8 - b'a' + 26 - pos) % 26) as usize] += 1.0);

            let chi = frequencies.iter()
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

    fn decrypt(ciphertext: &str, key: &str) -> String {
        let mut key_iter = key.as_bytes().iter().cycle();
        ciphertext.chars()
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
}

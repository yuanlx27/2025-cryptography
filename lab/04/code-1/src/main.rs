use std::io;
use std::io::{Read, Write};

fn main() {
    let mut ctx = SHA256::new();
    loop {
        let mut buf = [0u8; 640000];
        let n = io::stdin().read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
    }
    io::stdout().write_all(&ctx.finalize()).unwrap();
}

struct SHA256 {
    buffer: [u8; 64],
    buffer_len: usize,
    h: [u32; 8],
    len: usize,
    use_sha_ni: bool,
}

impl SHA256 {
    fn new() -> Self {
        SHA256 {
            buffer: [0u8; 64],
            buffer_len: 0,
            h: H,
            len: 0,
            use_sha_ni: detect_sha_ni(),
        }
    }

    fn update(&mut self, text: &[u8]) {
        self.len += text.len();
        text.chunks(64)
            .for_each(|chunk| {
                if chunk.len() < 64 {
                    self.buffer_len = chunk.len();
                    self.buffer[..self.buffer_len].copy_from_slice(chunk);
                    return;
                }
                self.process_chunk(chunk);
            });
    }

    fn finalize(&mut self) -> [u8; 32] {
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        if self.buffer_len > 56 {
            for i in self.buffer_len..64 {
                self.buffer[i] = 0;
            }
            let temp = self.buffer;
            self.process_chunk(&temp);
            self.buffer_len = 0;
        }

        for i in self.buffer_len..56 {
            self.buffer[i] = 0;
        }

        let len = self.len as u64 * 8;
        self.buffer[56..64].copy_from_slice(&len.to_be_bytes());
        let temp = self.buffer;
        self.process_chunk(&temp);

        let mut res = [0u8; 32];
        self.h.iter().enumerate().for_each(|(i, &word)| {
            res[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        });
        res
    }

    fn process_chunk(&mut self, chunk: &[u8]) {
        if self.use_sha_ni && self.try_process_chunk_sha_ni(chunk) {
            return;
        }

        let mut w = [0u32; 64];
        chunk.chunks(4).enumerate().for_each(|(i, bytes)| {
            w[i] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        });

        for i in 16..64 {
            w[i] = w[i - 16]
                .wrapping_add(small_sigma0(w[i - 15]))
                .wrapping_add(w[i - 7])
                .wrapping_add(small_sigma1(w[i - 2]));
        }

        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];
        let mut f = self.h[5];
        let mut g = self.h[6];
        let mut h = self.h[7];

        for i in 0..64 {
            let temp1 = h
                .wrapping_add(big_sigma1(e))
                .wrapping_add(choice(e, f, g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let temp2 = big_sigma0(a)
                .wrapping_add(majority(a, b, c));

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
        self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g);
        self.h[7] = self.h[7].wrapping_add(h);
    }

    fn try_process_chunk_sha_ni(&mut self, chunk: &[u8]) -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::*;

            let mut w = [0i32; 16];
            for (i, bytes) in chunk.chunks(4).enumerate() {
                w[i] = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            }

            let zero = _mm_setzero_si128();
            let mut w128 = [zero; 16];
            for (i, words) in w.chunks(4).enumerate() {
                w128[i] = _mm_set_epi32(words[3], words[2], words[1], words[0]);
            }

            for i in 4..16 {
                let x = {
                    let y = _mm_srli_si128(w128[i - 2], 4);
                    _mm_insert_epi32(y, _mm_cvtsi128_si32(w128[i - 1]), 3);
                    _mm_add_epi32(_mm_sha256msg1_epu32(w128[i - 4], w128[i - 3]), y)
                };
                w128[i] = _mm_sha256msg2_epu32(x, w128[i - 1]);
            }

            true
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = chunk;
            false
        }
    }
}

#[inline(always)]
fn small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline(always)]
fn small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

#[inline(always)]
fn big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline(always)]
fn big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[inline(always)]
fn choice(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline(always)]
fn majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn detect_sha_ni() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        // std::is_x86_feature_detected!("sha")
        false
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

const H: [u32; 8] = [
    0x6a09e667,
    0xbb67ae85,
    0x3c6ef372,
    0xa54ff53a,
    0x510e527f,
    0x9b05688c,
    0x1f83d9ab,
    0x5be0cd19,
];

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

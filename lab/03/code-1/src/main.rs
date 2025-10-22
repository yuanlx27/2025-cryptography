//! AES-128 CBC 实现（不使用外部 crate）
//!
//! 输入格式（从 stdin 读取二进制）:
//! - 1 byte: 模式
//!     0x01 -> CBC 加密
//!     0x81 -> CBC 解密 (最高位 1 表示解密)
//! - 16 bytes: key (AES-128)
//! - 16 bytes: IV
//! - 4 bytes: length (u32 big-endian)
//! - N bytes: 明文（加密时）或密文（解密时）
//!
//! 输出（写到 stdout）: 加密后的密文或解密后的明文（二进制）
//!
//! PKCS#7 padding:
//! - 加密：始终填充，填充长度为 N = 16 - (len % 16)，若正好为 16 的倍数则填充 16 个 0x10。
//! - 解密：移除并验证 PKCS#7 填充（若无效则报错）。

use std::error::Error;
use std::io::{self, Read, Write};

const BLOCK_SIZE: usize = 16;
const NK: usize = 4; // words in key for AES-128
const NR: usize = 10;

const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

const INV_SBOX: [u8; 256] = [
    0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
    0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
    0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
    0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
    0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
    0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
    0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
    0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
    0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
    0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
    0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
    0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
    0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
    0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef,
    0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
    0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d,
];

const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1B, 0x36];

fn xtime(x: u8) -> u8 {
    if x & 0x80 != 0 {
        ((x << 1) ^ 0x1b) & 0xff
    } else {
        (x << 1) & 0xff
    }
}

fn mul(a: u8, b: u8) -> u8 {
    // Russian peasant multiplication in GF(2^8)
    let mut aa = a;
    let mut bb = b;
    let mut res: u8 = 0;
    while bb != 0 {
        if (bb & 1) != 0 {
            res ^= aa;
        }
        aa = xtime(aa);
        bb >>= 1;
    }
    res
}

fn sub_bytes(state: &mut [u8; 16]) {
    for b in state.iter_mut() {
        *b = SBOX[*b as usize];
    }
}

fn inv_sub_bytes(state: &mut [u8; 16]) {
    for b in state.iter_mut() {
        *b = INV_SBOX[*b as usize];
    }
}

fn shift_rows(state: &mut [u8; 16]) {
    // column-major: state[c*4 + r]
    let mut tmp = [0u8; 16];
    tmp.copy_from_slice(state);
    for r in 0..4 {
        for c in 0..4 {
            tmp[c * 4 + r] = state[((c + r) % 4) * 4 + r];
        }
    }
    state.copy_from_slice(&tmp);
}

fn inv_shift_rows(state: &mut [u8; 16]) {
    let mut tmp = [0u8; 16];
    tmp.copy_from_slice(state);
    for r in 0..4 {
        for c in 0..4 {
            tmp[c * 4 + r] = state[((c + 4 - r) % 4) * 4 + r];
        }
    }
    state.copy_from_slice(&tmp);
}

fn mix_columns(state: &mut [u8; 16]) {
    for c in 0..4 {
        let i = c * 4;
        let a0 = state[i];
        let a1 = state[i + 1];
        let a2 = state[i + 2];
        let a3 = state[i + 3];

        let b0 = mul(0x02, a0) ^ mul(0x03, a1) ^ a2 ^ a3;
        let b1 = a0 ^ mul(0x02, a1) ^ mul(0x03, a2) ^ a3;
        let b2 = a0 ^ a1 ^ mul(0x02, a2) ^ mul(0x03, a3);
        let b3 = mul(0x03, a0) ^ a1 ^ a2 ^ mul(0x02, a3);

        state[i] = b0;
        state[i + 1] = b1;
        state[i + 2] = b2;
        state[i + 3] = b3;
    }
}

fn inv_mix_columns(state: &mut [u8; 16]) {
    for c in 0..4 {
        let i = c * 4;
        let a0 = state[i];
        let a1 = state[i + 1];
        let a2 = state[i + 2];
        let a3 = state[i + 3];

        let b0 = mul(0x0e, a0) ^ mul(0x0b, a1) ^ mul(0x0d, a2) ^ mul(0x09, a3);
        let b1 = mul(0x09, a0) ^ mul(0x0e, a1) ^ mul(0x0b, a2) ^ mul(0x0d, a3);
        let b2 = mul(0x0d, a0) ^ mul(0x09, a1) ^ mul(0x0e, a2) ^ mul(0x0b, a3);
        let b3 = mul(0x0b, a0) ^ mul(0x0d, a1) ^ mul(0x09, a2) ^ mul(0x0e, a3);

        state[i] = b0;
        state[i + 1] = b1;
        state[i + 2] = b2;
        state[i + 3] = b3;
    }
}

fn add_round_key(state: &mut [u8; 16], round_key: &[u8; 16]) {
    for i in 0..16 {
        state[i] ^= round_key[i];
    }
}

fn rot_word(x: u32) -> u32 {
    (x << 8) | (x >> 24)
}

fn sub_word(x: u32) -> u32 {
    let b0 = SBOX[((x >> 24) & 0xff) as usize] as u32;
    let b1 = SBOX[((x >> 16) & 0xff) as usize] as u32;
    let b2 = SBOX[((x >> 8) & 0xff) as usize] as u32;
    let b3 = SBOX[(x & 0xff) as usize] as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

fn expand_key(key: &[u8; 16]) -> [[u8; 16]; NR + 1] {
    // produce 11 round keys (0..10), each 16 bytes
    let mut w = [0u32; 4 * (NR + 1)]; // 44 words
    for i in 0..NK {
        w[i] = u32::from_be_bytes([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
    }
    for i in NK..(4 * (NR + 1)) {
        let mut temp = w[i - 1];
        if i % NK == 0 {
            temp = sub_word(rot_word(temp)) ^ ((RCON[i / NK - 1] as u32) << 24);
        }
        w[i] = w[i - NK] ^ temp;
    }

    let mut round_keys = [[0u8; 16]; NR + 1];
    for r in 0..(NR + 1) {
        for c in 0..4 {
            let word = w[r * 4 + c];
            let bytes = word.to_be_bytes();
            round_keys[r][4 * c] = bytes[0];
            round_keys[r][4 * c + 1] = bytes[1];
            round_keys[r][4 * c + 2] = bytes[2];
            round_keys[r][4 * c + 3] = bytes[3];
        }
    }
    round_keys
}

fn encrypt_block(input: &[u8; 16], round_keys: &[[u8; 16]; NR + 1]) -> [u8; 16] {
    let mut state = *input;
    add_round_key(&mut state, &round_keys[0]);
    for round in 1..NR {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &round_keys[round]);
    }
    // final round
    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &round_keys[NR]);
    state
}

fn decrypt_block(input: &[u8; 16], round_keys: &[[u8; 16]; NR + 1]) -> [u8; 16] {
    let mut state = *input;
    add_round_key(&mut state, &round_keys[NR]);
    for round in (1..NR).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(&mut state, &round_keys[round]);
        inv_mix_columns(&mut state);
    }
    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, &round_keys[0]);
    state
}

fn pkcs7_pad(data: &[u8]) -> Vec<u8> {
    let mut out = data.to_vec();
    let pad_len = BLOCK_SIZE - (data.len() % BLOCK_SIZE);
    let pad_byte = pad_len as u8;
    for _ in 0..pad_len {
        out.push(pad_byte);
    }
    out
}

fn pkcs7_unpad(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.is_empty() {
        return Err("invalid padding: empty data");
    }
    let pad = *data.last().unwrap() as usize;
    if pad == 0 || pad > BLOCK_SIZE {
        return Err("invalid padding value");
    }
    let len = data.len();
    for i in (len - pad)..len {
        if data[i] as usize != pad {
            return Err("invalid padding bytes");
        }
    }
    Ok(data[..len - pad].to_vec())
}

fn xor_block(a: &mut [u8; 16], b: &[u8; 16]) {
    for i in 0..16 {
        a[i] ^= b[i];
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut stdin = io::stdin();
    // read mode
    let mut mode_b = [0u8; 1];
    stdin.read_exact(&mut mode_b)?;
    let mode = mode_b[0];
    // check mode validity
    if mode & 0x7f != 0x01 {
        return Err(format!(
            "unsupported mode byte: 0x{:02x}. expected 0x01 (encrypt) or 0x81 (decrypt)",
            mode
        )
        .into());
    }
    let decrypt = (mode & 0x80) != 0;

    // read key and iv
    let mut key = [0u8; 16];
    stdin.read_exact(&mut key)?;
    let mut iv = [0u8; 16];
    stdin.read_exact(&mut iv)?;

    // read 4-byte length (big-endian)
    let mut len_b = [0u8; 4];
    stdin.read_exact(&mut len_b)?;
    let data_len = u32::from_be_bytes(len_b) as usize;

    let mut data = vec![0u8; data_len];
    if data_len > 0 {
        stdin.read_exact(&mut data)?;
    }

    let round_keys = expand_key(&key);

    let mut stdout = io::stdout();

    if decrypt {
        if data.len() % BLOCK_SIZE != 0 || data.is_empty() {
            return Err("ciphertext length must be a non-zero multiple of 16 bytes".into());
        }
        let blocks = data.len() / BLOCK_SIZE;
        let mut prev = iv;
        let mut out = Vec::with_capacity(data.len());
        for i in 0..blocks {
            let mut block = [0u8; 16];
            block.copy_from_slice(&data[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]);
            let decrypted = decrypt_block(&block, &round_keys);
            let mut x = decrypted;
            xor_block(&mut x, &prev);
            out.extend_from_slice(&x);
            prev = block;
        }
        // remove padding
        let unpadded = pkcs7_unpad(&out).map_err(|e| format!("padding error: {}", e))?;
        stdout.write_all(&unpadded)?;
    } else {
        // encrypt
        let padded = pkcs7_pad(&data);
        let blocks = padded.len() / BLOCK_SIZE;
        let mut prev = iv;
        for i in 0..blocks {
            let mut block = [0u8; 16];
            block.copy_from_slice(&padded[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]);
            xor_block(&mut block, &prev);
            let encrypted = encrypt_block(&block, &round_keys);
            stdout.write_all(&encrypted)?;
            prev = encrypted;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

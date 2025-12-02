// Standard Input/Output.
use std::io;
use std::io::{Read, Write};
// Process bytes.
use byteorder::{BigEndian, ByteOrder};

// ==========================================
// 1. Constants
// ==========================================
const KEY_SIZE: usize = 256; // 2048 bits = 256 bytes
const PRIME_SIZE: usize = 128; // 1024 bits = 128 bytes
const HASH_LEN: usize = 32;  // SHA-256 output size

fn main() -> io::Result<()> {
    // We wrap the logic in a helper function.
    // If it returns None (any error/invalid input), main still returns Ok(()).
    // This ensures Exit Code 0 and "prints nothing" on failure.
    if let Some(message) = try_decrypt() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&message)?;
    }
    Ok(())
}

fn try_decrypt() -> Option<Vec<u8>> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // Helper: read exact bytes, return None if EOF or error
    let mut read_exact = |buf: &mut [u8]| -> Option<()> {
        handle.read_exact(buf).ok()
    };

    // ==========================================
    // 2. Input Parsing
    // ==========================================

    // 1. Read p
    let mut p_bytes = [0u8; PRIME_SIZE];
    read_exact(&mut p_bytes)?;
    let p = BigUint::from_be_bytes(&p_bytes);
    if p.is_zero() { return None; } // Invalid modulus

    // 2. Read q
    let mut q_bytes = [0u8; PRIME_SIZE];
    read_exact(&mut q_bytes)?;
    let q = BigUint::from_be_bytes(&q_bytes);
    if q.is_zero() { return None; } // Invalid modulus

    // 3. Read n (skip but read to advance stream)
    let mut n_bytes = [0u8; KEY_SIZE];
    read_exact(&mut n_bytes)?;

    // 4. Read d (skip)
    let mut _d_bytes = [0u8; KEY_SIZE];
    read_exact(&mut _d_bytes)?;

    // 5. Read dP
    let mut dp_bytes = [0u8; PRIME_SIZE];
    read_exact(&mut dp_bytes)?;
    let dp = BigUint::from_be_bytes(&dp_bytes);

    // 6. Read dQ
    let mut dq_bytes = [0u8; PRIME_SIZE];
    read_exact(&mut dq_bytes)?;
    let dq = BigUint::from_be_bytes(&dq_bytes);

    // 7. Read qInv
    let mut qinv_bytes = [0u8; PRIME_SIZE];
    read_exact(&mut qinv_bytes)?;
    let qinv = BigUint::from_be_bytes(&qinv_bytes);

    // 8. Read Ciphertext
    let mut c_bytes = [0u8; KEY_SIZE];
    read_exact(&mut c_bytes)?;
    let c = BigUint::from_be_bytes(&c_bytes);

    // ==========================================
    // 3. CRT Decryption
    // ==========================================

    // m1 = c^dP mod p
    // Reduce c mod p first because c is 2048 bits and p is 1024
    let c_mod_p = c.rem(&p);
    let m1 = c_mod_p.modpow(&dp, &p);

    // m2 = c^dQ mod q
    let c_mod_q = c.rem(&q);
    let m2 = c_mod_q.modpow(&dq, &q);

    // h = (m1 - m2) * qInv mod p
    // Safe subtraction: if m1 < m2, compute (m1 + p - m2)
    // Note: m2 is mod q, so it *could* be larger than p if q > p,
    // so strictly we need m2 % p for the subtraction logic in mod p arithmetic.
    // However, typically Garner's formula uses values reduced by their respective moduli.
    // Let's act strictly in mod p:
    let m2_mod_p = m2.rem(&p);
    
    let diff = if m1.ge(&m2_mod_p) {
        let mut tmp = m1;
        tmp.sub_assign(&m2_mod_p);
        tmp
    } else {
        let mut tmp = m1;
        tmp.add_assign(&p);
        tmp.sub_assign(&m2_mod_p);
        tmp
    };

    // h = diff * qInv mod p
    let h = diff.mul_mod(&qinv, &p);

    // m = m2 + h * q
    let h_q = h.mul(&q);
    let mut m = m2;
    m.add_assign(&h_q);

    // EM (Encoded Message)
    let em = m.to_bytes_be(KEY_SIZE);

    // ==========================================
    // 4. OAEP Decoding
    // ==========================================

    // 1. Check first byte is 0x00
    if em[0] != 0x00 { return None; }

    let masked_seed = &em[1..1 + HASH_LEN];
    let masked_db = &em[1 + HASH_LEN..];

    // 2. Recover Seed
    let seed_mask = mgf1(masked_db, HASH_LEN);
    let seed: Vec<u8> = masked_seed.iter().zip(seed_mask.iter()).map(|(a, b)| a ^ b).collect();

    // 3. Recover DB
    let db_mask = mgf1(&seed, KEY_SIZE - HASH_LEN - 1);
    let db: Vec<u8> = masked_db.iter().zip(db_mask.iter()).map(|(a, b)| a ^ b).collect();

    // 4. Verify lHash
    let l_hash_expected = Sha256::digest(&[]);
    let l_hash_actual = &db[0..HASH_LEN];

    if l_hash_actual != l_hash_expected.as_slice() { return None; }

    // 5. Find 0x01 separator; verify PS is all zeros
    let mut separator_idx = usize::MAX;
    for i in HASH_LEN..db.len() {
        let b = db[i];
        if b == 0x01 {
            separator_idx = i;
            break;
        }
        if b != 0x00 {
            // PS must be all zeros
            return None;
        }
    }

    if separator_idx == usize::MAX { return None; }

    // 6. Extract Message
    Some(db[separator_idx + 1..].to_vec())
}

fn mgf1(seed: &[u8], len: usize) -> Vec<u8> {
    let mut t = Vec::with_capacity(len + HASH_LEN);
    let mut counter = 0u32;
    while t.len() < len {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(&counter.to_be_bytes());
        t.extend_from_slice(&hasher.finalize());
        counter += 1;
    }
    t.truncate(len);
    t
}

// ==========================================
// 5. BigUint Implementation
// ==========================================

const LIMBS: usize = 66; // Safe margin for 2048 bits + operations

#[derive(Debug, Clone, Copy)]
struct BigUint {
    data: [u64; LIMBS],
}

impl BigUint {
    fn new() -> Self {
        Self { data: [0u64; LIMBS] }
    }

    fn one() -> Self {
        let mut s = Self::new();
        s.data[0] = 1;
        s
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut bytes = bytes.to_vec();
        bytes.reverse();
        let mut data = [0u64; LIMBS];
        bytes.chunks(8).enumerate().for_each(|(i, chunk)| {
            if i < LIMBS {
                let mut arr = [0u8; 8];
                arr[..chunk.len()].copy_from_slice(chunk);
                data[i] = u64::from_le_bytes(arr);
            }
        });
        Self { data }
    }

    fn to_bytes_be(&self, width: usize) -> Vec<u8> {
        let mut res = vec![0u8; width];
        let mut temp = *self;
        for i in (0..width).rev() {
            res[i] = (temp.data[0] & 0xFF) as u8;
            temp.shr_8();
        }
        res
    }

    fn modpow(&self, exponent: &BigUint, modulus: &BigUint) -> BigUint {
        if modulus.is_zero() {
            // Fallback or panic, but input checks should prevent this.
            // Return self as dummy to avoid panic if check missed.
            return *self; 
        }

        let n_prime = Self::compute_n_prime(modulus.data[0]);
        let n_limbs = modulus.limbs_used();
        
        let mut r_val = BigUint::new();
        if n_limbs < LIMBS { r_val.data[n_limbs] = 1; } 
        else { return BigUint::new(); } // Error case
        
        r_val = r_val.rem(modulus); 
        let r_sq = r_val.mul_mod(&r_val, modulus); 
        let mut x = self.mont_mul(&r_sq, modulus, n_prime, n_limbs);
        let mut res = r_val; 

        let mut exp_bits = LIMBS * 64; 
        while exp_bits > 0 {
            exp_bits -= 1;
            let limb = exp_bits / 64;
            let bit = exp_bits % 64;
            if (exponent.data[limb] >> bit) & 1 == 1 {
                exp_bits += 1;
                break;
            }
        }

        for i in (0..exp_bits).rev() {
            res = res.mont_mul(&res, modulus, n_prime, n_limbs);
            let limb = i / 64;
            let bit = i % 64;
            if (exponent.data[limb] >> bit) & 1 == 1 {
                res = res.mont_mul(&x, modulus, n_prime, n_limbs);
            }
        }

        res.mont_mul(&BigUint::one(), modulus, n_prime, n_limbs)
    }

    fn mont_mul(&self, other: &BigUint, n: &BigUint, n_prime: u64, n_limbs: usize) -> BigUint {
        let mut t = [0u64; LIMBS * 2]; 
        
        for i in 0..n_limbs {
            let mut carry: u128 = 0;
            let a_i = self.data[i] as u128;
            if a_i == 0 { continue; }
            for j in 0..n_limbs {
                let val = a_i * (other.data[j] as u128) + (t[i + j] as u128) + carry;
                t[i + j] = val as u64;
                carry = val >> 64;
            }
            let mut k = i + n_limbs;
            while carry > 0 {
                let val = (t[k] as u128) + carry;
                t[k] = val as u64;
                carry = val >> 64;
                k += 1;
            }
        }

        for i in 0..n_limbs {
            let m = t[i].wrapping_mul(n_prime);
            let mut carry: u128 = 0;
            let m_u128 = m as u128;
            
            for j in 0..n_limbs {
                let val = (t[i + j] as u128) + m_u128 * (n.data[j] as u128) + carry;
                t[i + j] = val as u64;
                carry = val >> 64;
            }
            
            let mut k = i + n_limbs;
            while carry > 0 {
                let val = (t[k] as u128) + carry;
                t[k] = val as u64;
                carry = val >> 64;
                k += 1;
            }
        }

        let mut res = BigUint::new();
        for i in 0..n_limbs + 1 {
            if i + n_limbs < t.len() {
                res.data[i] = t[i + n_limbs];
            }
        }

        if res.ge(n) {
            res.sub_assign(n);
        }
        res
    }

    fn compute_n_prime(n0: u64) -> u64 {
        let mut x = n0;
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x.wrapping_neg()
    }

    fn limbs_used(&self) -> usize {
        for i in (0..LIMBS).rev() {
            if self.data[i] != 0 {
                return i + 1;
            }
        }
        1
    }

    fn mul_mod(&self, other: &BigUint, modulus: &BigUint) -> BigUint {
        let product = self.mul(other);
        product.rem(modulus)
    }

    fn mul(&self, other: &BigUint) -> BigUint {
        let mut res = BigUint::new();
        for i in 0..LIMBS {
            let mut carry: u128 = 0;
            if self.data[i] == 0 { continue; }
            for j in 0..(LIMBS - i) {
                let val = (self.data[i] as u128) * (other.data[j] as u128) + (res.data[i + j] as u128) + carry;
                res.data[i + j] = val as u64;
                carry = val >> 64;
            }
        }
        res
    }

    fn rem(&self, modulus: &BigUint) -> BigUint {
        if modulus.is_zero() { return BigUint::new(); } // Prevent panic
        let mut remainder = BigUint::new();
        let mut bit_idx = LIMBS * 64;
        while bit_idx > 0 {
            bit_idx -= 1;
            let limb = bit_idx / 64;
            if (self.data[limb] >> (bit_idx % 64)) & 1 == 1 {
                bit_idx += 1; break;
            }
        }
        for i in (0..bit_idx).rev() {
            remainder.shl_1();
            if (self.data[i / 64] >> (i % 64)) & 1 == 1 {
                remainder.data[0] |= 1;
            }
            if remainder.ge(modulus) {
                remainder.sub_assign(modulus);
            }
        }
        remainder
    }

    fn is_zero(&self) -> bool { self.data.iter().all(|&x| x == 0) }

    fn ge(&self, other: &BigUint) -> bool {
        for i in (0..LIMBS).rev() {
            if self.data[i] > other.data[i] { return true; }
            if self.data[i] < other.data[i] { return false; }
        }
        true
    }

    fn sub_assign(&mut self, other: &BigUint) {
        let mut borrow: u64 = 0;
        for i in 0..LIMBS {
            let (diff, b1) = self.data[i].overflowing_sub(other.data[i]);
            let (diff2, b2) = diff.overflowing_sub(borrow);
            self.data[i] = diff2;
            borrow = (if b1 {1} else {0}) + (if b2 {1} else {0});
        }
    }

    fn add_assign(&mut self, other: &BigUint) {
        let mut carry: u64 = 0;
        for i in 0..LIMBS {
            let (sum, c1) = self.data[i].overflowing_add(other.data[i]);
            let (sum2, c2) = sum.overflowing_add(carry);
            self.data[i] = sum2;
            carry = (if c1 {1} else {0}) + (if c2 {1} else {0});
        }
    }

    fn shl_1(&mut self) {
        let mut carry = 0;
        for i in 0..LIMBS {
            let next_carry = self.data[i] >> 63;
            self.data[i] = (self.data[i] << 1) | carry;
            carry = next_carry;
        }
    }
    
    fn shr_8(&mut self) {
        let mut carry = 0;
        for i in (0..LIMBS).rev() {
            let next_carry = (self.data[i] & 0xFF) << 56;
            self.data[i] = (self.data[i] >> 8) | carry;
            carry = next_carry;
        }
    }
}

// ==========================================
// 6. SHA-256 Implementation
// ==========================================

struct Sha256 {
    state: [u32; 8],
    data: [u8; 64],
    datalen: usize,
    bitlen: u64,
}

impl Sha256 {
    fn new() -> Self {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
            ],
            data: [0; 64],
            datalen: 0,
            bitlen: 0,
        }
    }

    fn update(&mut self, input: &[u8]) {
        for &byte in input {
            self.data[self.datalen] = byte;
            self.datalen += 1;
            self.bitlen += 8;
            if self.datalen == 64 {
                self.transform();
                self.datalen = 0;
            }
        }
    }

    fn finalize(mut self) -> Vec<u8> {
        let i = self.datalen;
        self.data[i] = 0x80;
        self.datalen += 1;
        if self.datalen > 56 {
            while self.datalen < 64 { self.data[self.datalen] = 0; self.datalen += 1; }
            self.transform();
            self.datalen = 0;
        }
        while self.datalen < 56 { self.data[self.datalen] = 0; self.datalen += 1; }
        let bits = self.bitlen.to_be_bytes();
        for (idx, &b) in bits.iter().enumerate() { self.data[56 + idx] = b; }
        self.transform();
        let mut out = Vec::with_capacity(32);
        for s in self.state.iter() { out.extend_from_slice(&s.to_be_bytes()); }
        out
    }
    
    fn digest(input: &[u8]) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(input);
        h.finalize()
    }

    fn transform(&mut self) {
        let mut m = [0u32; 64];
        for i in 0..16 {
            m[i] = BigEndian::read_u32(&self.data[i * 4..(i + 1) * 4]);
        }
        for i in 16..64 {
            let s0 = m[i - 15].rotate_right(7) ^ m[i - 15].rotate_right(18) ^ (m[i - 15] >> 3);
            let s1 = m[i - 2].rotate_right(17) ^ m[i - 2].rotate_right(19) ^ (m[i - 2] >> 10);
            m[i] = m[i - 16].wrapping_add(s0).wrapping_add(m[i - 7]).wrapping_add(s1);
        }
        let mut a = self.state[0]; let mut b = self.state[1]; let mut c = self.state[2]; let mut d = self.state[3];
        let mut e = self.state[4]; let mut f = self.state[5]; let mut g = self.state[6]; let mut h = self.state[7];
        let k = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
            0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
            0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
            0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
        ];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(m[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g; g = f; f = e; e = d.wrapping_add(temp1); d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
        }
        self.state[0] = self.state[0].wrapping_add(a); self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c); self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e); self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g); self.state[7] = self.state[7].wrapping_add(h);
    }
}

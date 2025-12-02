// Standard Input/Output.
use std::io;
use std::io::{Read, Write};
// Process bytes.
use byteorder::{BigEndian, ByteOrder};
// CSPRNG.
use rand::{TryRngCore, rngs::OsRng};

// ==========================================
// 1. Constants
// ==========================================
const KEY_SIZE: usize = 256; // 2048 bits = 256 bytes
const HASH_LEN: usize = 32;  // SHA-256 output size

fn main() -> io::Result<()> {
    // Lock stdin.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // ==========================================
    // 2. Input Parsing (Streaming)
    // ==========================================
    
    // 1. Ignore first 16 bytes
    let mut _discard_16 = [0u8; 16];
    handle.read_exact(&mut _discard_16)?;

    // 2. Read N (256 bytes)
    let mut n_bytes = [0u8; KEY_SIZE];
    handle.read_exact(&mut n_bytes)?;
    let n = BigUint::from_be_bytes(&n_bytes);

    // 3. Read E (256 bytes)
    let mut e_bytes = [0u8; KEY_SIZE];
    handle.read_exact(&mut e_bytes)?;
    let e = BigUint::from_be_bytes(&e_bytes);

    // 4. Ignore next 256 bytes (likely D, not needed for encryption)
    let mut _discard_256 = [0u8; KEY_SIZE];
    handle.read_exact(&mut _discard_256)?;

    // 5. Read Message Length (1 byte)
    let mut m_len_buf = [0u8; 1];
    handle.read_exact(&mut m_len_buf)?;
    let m_len = m_len_buf[0] as usize;

    // 6. Read Message Content
    let mut message = vec![0u8; m_len];
    handle.read_exact(&mut message)?;

    // ==========================================
    // 3. OAEP Padding
    // ==========================================
    // L is empty, lHash = Hash(L)
    let l_hash = Sha256::digest(&[]);

    // PS (Padding String) - Zeros
    let ps_len = KEY_SIZE - m_len - 2 * HASH_LEN - 2;
    let ps = vec![0u8; ps_len];

    // DB = lHash || PS || 0x01 || M
    let mut db = Vec::with_capacity(KEY_SIZE - HASH_LEN - 1);
    db.extend_from_slice(&l_hash);
    db.extend_from_slice(&ps);
    db.push(0x01);
    db.extend_from_slice(&message);

    // Generate random Seed using rand crate (CSPRNG)
    let mut seed = [0u8; HASH_LEN];
    OsRng.try_fill_bytes(&mut seed).unwrap();

    // MGF1 Masking
    let db_mask = mgf1(&seed, KEY_SIZE - HASH_LEN - 1);
    let masked_db: Vec<u8> = db.iter().zip(db_mask.iter()).map(|(a, b)| a ^ b).collect();

    let seed_mask = mgf1(&masked_db, HASH_LEN);
    let masked_seed: Vec<u8> = seed.iter().zip(seed_mask.iter()).map(|(a, b)| a ^ b).collect();

    // EM = 0x00 || maskedSeed || maskedDB
    let mut em = Vec::with_capacity(KEY_SIZE);
    em.push(0x00);
    em.extend_from_slice(&masked_seed);
    em.extend_from_slice(&masked_db);

    // ==========================================
    // 4. RSA Encryption with Montgomery Reduction
    // ==========================================
    let m_int = BigUint::from_be_bytes(&em);
    
    // Use the optimized modular exponentiation
    let c_int = m_int.modpow(&e, &n);

    // Output 256 bytes
    let c_bytes = c_int.to_bytes_be(KEY_SIZE);
    io::stdout().write_all(&c_bytes)?;

    Ok(())
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
// 5. BigUint Implementation (Montgomery Optimized)
// ==========================================

// We need 2048 bits. 64 bits * 32 = 2048.
// To handle overflow during multiplication before reduction, we usually need 2x size.
// 256 bytes = 32 u64 limbs.
// For Montgomery mul, we keep the `LIMBS` large enough to hold intermediate product.
// 2048 bits / 64 = 32 limbs.
// Product is 4096 bits = 64 limbs.
// So LIMBS = 66 gives us a safe margin for carries.
const LIMBS: usize = 66; 

#[derive(Debug, Clone, Copy)]
struct BigUint {
    data: [u64; LIMBS],
}

impl BigUint {
    fn new() -> Self {
        Self {
            data: [0u64; LIMBS],
        }
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut bytes = bytes.to_vec();
        bytes.reverse();
        Self::from_le_bytes(&bytes)
    }

    fn from_le_bytes(bytes: &[u8]) -> Self {
        // Ensure we don't buffer overflow if input is huge, though logic says 256 bytes max
        let mut data = [0u64; LIMBS];
        bytes.chunks(8).enumerate().for_each(|(i, chunk)| {
            if i < LIMBS {
                let mut arr = [0u8; 8];
                arr[..chunk.len()].copy_from_slice(chunk);
                data[i] = u64::from_le_bytes(arr);
            }
        });

        Self {
            data,
        }
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

    // Montgomery Modular Exponentiation
    fn modpow(&self, exponent: &BigUint, modulus: &BigUint) -> BigUint {
        if modulus.is_zero() { panic!("Modulus is zero"); }

        // 1. Precompute constants for Montgomery Reduction
        // n_prime = -N^(-1) mod 2^64
        let n_prime = Self::compute_n_prime(modulus.data[0]);
        
        // R = 2^(64 * 32) for 2048 bit modulus (assuming modulus fits in 32 limbs)
        // We need R > N. Since N is 2048 bits, let's define R based on limbs count corresponding to N.
        // A simpler approach for general code:
        // Find number of limbs used by modulus.
        let n_limbs = modulus.limbs_used();
        // R = 2^(64 * n_limbs)
        
        // Calculate R^2 mod N
        // R is represented by a 1 at bit index (64 * n_limbs).
        // We construct R^2 by shifting or using BigUint arithmetic, then taking remainder.
        let mut r_sq = BigUint::new();
        // To be safe, we can compute 2^(2 * 64 * n_limbs) mod N.
        // Or simply: compute 2^(64 * n_limbs) % N, then square it mod N.
        
        // Construct R % N
        let mut r_val = BigUint::new();
        // Set bit at 64 * n_limbs
        // Since our LIMBS is 66 and n_limbs is likely 32, we can handle this.
        if n_limbs < LIMBS {
            r_val.data[n_limbs] = 1; 
        } else {
            // Fallback/Error case, though for 2048 bits LIMBS=66 is fine
             panic!("Modulus too large for configured LIMBS");
        }
        r_val = r_val.rem(modulus); // R mod N

        // Compute R^2 mod N
        // We can use standard mul_mod here just once to initialize.
        r_sq = r_val.mul_mod(&r_val, modulus); 

        // 2. Convert Base to Montgomery Form: A_mont = A * R mod N
        // We can use mont_mul(A, R^2) -> A * R^2 * R^-1 = A * R
        let mut x = self.mont_mul(&r_sq, modulus, n_prime, n_limbs);

        // 3. Initialize result to 1 in Montgomery Form: 1 * R mod N
        // We already have R mod N in r_val.
        let mut res = r_val; // This is 1_mont

        // 4. Binary Exponentiation
        let total_bits = LIMBS * 64; 
        // Find highest set bit in exponent to avoid useless loops
        let mut exp_bits = total_bits;
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
            // Square
            res = res.mont_mul(&res, modulus, n_prime, n_limbs);

            // Multiply if bit is set
            let limb = i / 64;
            let bit = i % 64;
            if (exponent.data[limb] >> bit) & 1 == 1 {
                res = res.mont_mul(&x, modulus, n_prime, n_limbs);
            }
        }

        // 5. Convert back from Montgomery Form: Res = Res_mont * 1 mod N
        // mont_mul(Res_mont, 1) -> Res * R * 1 * R^-1 = Res
        let one = {
            let mut t = BigUint::new();
            t.data[0] = 1;
            t
        };
        
        // Standard mont_reduce is effectively mont_mul(val, 1) but with 1 not shifted.
        // Actually, mont_mul(val, 1) works if 1 is standard '1'.
        res.mont_mul(&one, modulus, n_prime, n_limbs)
    }

    // Standard Montgomery Reduction: T * R^-1 mod N
    // T is usually product of two numbers in Montgomery form (A*R)*(B*R) = AB*R^2
    // Result is AB*R
    fn mont_mul(&self, other: &BigUint, n: &BigUint, n_prime: u64, n_limbs: usize) -> BigUint {
        // 1. Standard Multiplication T = A * B
        let mut t = [0u64; LIMBS * 2]; // Temporary double width buffer
        
        for i in 0..n_limbs {
            let mut carry: u128 = 0;
            let a_i = self.data[i] as u128;
            if a_i == 0 { continue; }
            for j in 0..n_limbs {
                let val = a_i * (other.data[j] as u128) 
                          + (t[i + j] as u128) 
                          + carry;
                t[i + j] = val as u64;
                carry = val >> 64;
            }
            // Propagate carry
            let mut k = i + n_limbs;
            while carry > 0 {
                let val = (t[k] as u128) + carry;
                t[k] = val as u64;
                carry = val >> 64;
                k += 1;
            }
        }

        // 2. Montgomery Reduction
        // T is now in t[]
        // We process limb by limb
        for i in 0..n_limbs {
            // m = (T[i] * n_prime) mod 2^64
            let m = t[i].wrapping_mul(n_prime);
            
            // T = T + m * N * 2^(64*i)
            // Effectively, we add m*N shifted by i limbs to T
            let mut carry: u128 = 0;
            let m_u128 = m as u128;
            
            for j in 0..n_limbs {
                let val = (t[i + j] as u128) + m_u128 * (n.data[j] as u128) + carry;
                t[i + j] = val as u64;
                carry = val >> 64;
            }
            
            // Handle carries up the chain
            let mut k = i + n_limbs;
            while carry > 0 {
                let val = (t[k] as u128) + carry;
                t[k] = val as u64;
                carry = val >> 64;
                k += 1;
            }
        }

        // 3. Result is T / R. Since we added multiples, the lower n_limbs are zero.
        // We shift right by n_limbs words.
        let mut res = BigUint::new();
        for i in 0..n_limbs + 1 {
            if i + n_limbs < t.len() {
                res.data[i] = t[i + n_limbs];
            }
        }

        // 4. Conditional subtraction
        if res.ge(n) {
            res.sub_assign(n);
        }
        
        res
    }

    // Helper to compute -N^(-1) mod 2^64
    fn compute_n_prime(n0: u64) -> u64 {
        let mut x = n0;
        // Newton-Raphson iteration for modular inverse
        // We want x * n0 = 1 mod 2^k
        // 3-bit approx: x * n0 = 1 mod 8 is always x = n0 for odd n0 (RSA modulus is odd)
        // Actually for 64-bit, we iterate: x = x * (2 - n0 * x)
        
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x)));
        x = x.wrapping_mul(2u64.wrapping_sub(n0.wrapping_mul(x))); // 2^64 reached

        // We want -N^(-1), so negate
        x.wrapping_neg()
    }

    fn limbs_used(&self) -> usize {
        for i in (0..LIMBS).rev() {
            if self.data[i] != 0 {
                return i + 1;
            }
        }
        1 // Avoid 0
    }

    // Keep fallback mul_mod for precomputation
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
                let val = (self.data[i] as u128) * (other.data[j] as u128) 
                          + (res.data[i + j] as u128) 
                          + carry;
                res.data[i + j] = val as u64;
                carry = val >> 64;
            }
        }
        res
    }

    fn rem(&self, modulus: &BigUint) -> BigUint {
        if modulus.is_zero() { panic!("Division by zero"); }
        
        let mut remainder = BigUint::new();
        // Simplistic bit-wise reduction
        let total_bits = LIMBS * 64;
        let mut bit_idx = total_bits;
        
        // Find MSB of self
        while bit_idx > 0 {
            bit_idx -= 1;
            let limb = bit_idx / 64;
            let bit = bit_idx % 64;
            if (self.data[limb] >> bit) & 1 == 1 {
                bit_idx += 1;
                break;
            }
        }

        for i in (0..bit_idx).rev() {
            remainder.shl_1();
            let limb = i / 64;
            let bit = i % 64;
            let val = (self.data[limb] >> bit) & 1;
            remainder.data[0] |= val;

            if remainder.ge(modulus) {
                remainder.sub_assign(modulus);
            }
        }

        remainder
    }

    fn is_zero(&self) -> bool {
        self.data.iter().all(|&x| x == 0)
    }

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
            while self.datalen < 64 {
                self.data[self.datalen] = 0;
                self.datalen += 1;
            }
            self.transform();
            self.datalen = 0;
        }

        while self.datalen < 56 {
            self.data[self.datalen] = 0;
            self.datalen += 1;
        }

        let bits = self.bitlen.to_be_bytes();
        for (idx, &b) in bits.iter().enumerate() {
            self.data[56 + idx] = b;
        }
        self.transform();

        let mut out = Vec::with_capacity(32);
        for s in self.state.iter() {
            out.extend_from_slice(&s.to_be_bytes());
        }
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

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

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

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

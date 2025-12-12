use std::cmp::Ordering;
use std::fmt;
use std::io::{self, BufRead};
use std::ops::{Add, Mul, Rem, Sub};
use std::str::FromStr;

fn main() {
    // Read inputs from stdin
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Enter p (modulus):");
    let p_str = lines.next().expect("Expected p").expect("Read error");
    let p = BigUint::from_str(p_str.trim()).expect("Invalid p");

    println!("Enter n (order, 64-bit):");
    let n_str = lines.next().expect("Expected n").expect("Read error");
    let n = n_str.trim().parse::<u64>().expect("Invalid n");

    println!("Enter alpha (generator):");
    let alpha_str = lines.next().expect("Expected alpha").expect("Read error");
    let alpha = BigUint::from_str(alpha_str.trim()).expect("Invalid alpha");

    println!("Enter beta (target):");
    let beta_str = lines.next().expect("Expected beta").expect("Read error");
    let beta = BigUint::from_str(beta_str.trim()).expect("Invalid beta");

    println!("Solving beta = alpha^x (mod p)");
    println!("p = {}", p);
    println!("n = {}", n);
    println!("alpha = {}", alpha);
    println!("beta = {}", beta);

    match pollard_rho(&alpha, &beta, &p, n) {
        Some(x) => println!("Found x: {}", x),
        None => println!("Failed to find x"),
    }
}

// --- Pollard's Rho Implementation ---

#[derive(Clone, Debug)]
struct State {
    x: BigUint,
    a: u64,
    b: u64,
}

fn step(s: &State, alpha: &BigUint, beta: &BigUint, p: &BigUint, n: u64) -> State {
    // Determine the set S0, S1, S2 based on x % 3
    let rem3 = (&s.x % &BigUint::from_u64(3)).to_u64();
    
    if rem3 == 0 {
        // S0: x -> x^2, a -> 2a, b -> 2b
        State {
            x: (&s.x * &s.x) % p.clone(),
            a: (s.a as u128 * 2 % n as u128) as u64,
            b: (s.b as u128 * 2 % n as u128) as u64,
        }
    } else if rem3 == 1 {
        // S1: x -> x * beta, a -> a, b -> b + 1
        State {
            x: (&s.x * beta) % p.clone(),
            a: s.a,
            b: (s.b + 1) % n,
        }
    } else {
        // S2: x -> x * alpha, a -> a + 1, b -> b
        State {
            x: (&s.x * alpha) % p.clone(),
            a: (s.a + 1) % n,
            b: s.b,
        }
    }
}

fn pollard_rho(alpha: &BigUint, beta: &BigUint, p: &BigUint, n: u64) -> Option<u64> {
    let mut tortoise = State {
        x: BigUint::one(),
        a: 0,
        b: 0,
    };
    let mut hare = tortoise.clone();

    // Loop until collision
    loop {
        tortoise = step(&tortoise, alpha, beta, p, n);
        hare = step(&hare, alpha, beta, p, n);
        hare = step(&hare, alpha, beta, p, n);

        if tortoise.x == hare.x {
            break;
        }
    }

    // Collision found:
    // alpha^at * beta^bt = alpha^ah * beta^bh (mod p)
    // alpha^(at - ah) = beta^(bh - bt) (mod p)
    // Let x = log_alpha(beta)
    // alpha^(at - ah) = alpha^(x * (bh - bt)) (mod p)
    // at - ah = x * (bh - bt) (mod n)
    // x * (bt - bh) = (ah - at) (mod n)   <-- Flipping signs for convenience
    
    // Note: Use u128 for intermediate calculations to handle modular arithmetic safely
    // (a - b) mod n ==> (a + n - b) % n
    
    let diff_b = (tortoise.b + n - hare.b) % n;
    let diff_a = (hare.a + n - tortoise.a) % n;
    
    if diff_b == 0 {
        return None;
    }

    let (g, inv_b, _) = extended_gcd(diff_b as i64, n as i64);
    if g != 1 {
        // If gcd is not 1, we might still solve it, but simple inverse won't work directly
        // usually implies we need to restart or handle multiple solutions.
        // For this specific problem (n is prime), g should be 1 unless diff_b is 0.
        return None; 
    }

    let inv_b = (inv_b % n as i64 + n as i64) as u64 % n;
    let x = (diff_a as u128 * inv_b as u128 % n as u128) as u64;
    
    Some(x)
}

fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        (b, 0, 1)
    } else {
        let (g, x1, y1) = extended_gcd(b % a, a);
        let x = y1 - (b / a) * x1;
        let y = x1;
        (g, x, y)
    }
}

// --- Minimal BigUint Implementation ---

#[derive(Clone, Debug, Eq)]
pub struct BigUint {
    data: Vec<u64>,
}

impl BigUint {
    pub fn new(data: Vec<u64>) -> Self {
        let mut res = BigUint { data };
        res.trim();
        res
    }

    pub fn zero() -> Self {
        BigUint { data: vec![0] }
    }

    pub fn one() -> Self {
        BigUint { data: vec![1] }
    }

    pub fn from_u64(v: u64) -> Self {
        BigUint { data: vec![v] }
    }
    
    pub fn to_u64(&self) -> u64 {
        if self.data.is_empty() { 0 } else { self.data[0] }
    }

    fn trim(&mut self) {
        while self.data.len() > 1 && self.data.last() == Some(&0) {
            self.data.pop();
        }
    }

    pub fn is_zero(&self) -> bool {
        self.data.len() == 1 && self.data[0] == 0
    }
    
    fn get_bit(&self, index: usize) -> bool {
        let word_idx = index / 64;
        let bit_idx = index % 64;
        if word_idx >= self.data.len() {
            return false;
        }
        (self.data[word_idx] & (1 << bit_idx)) != 0
    }

    fn set_bit(&mut self, index: usize) {
        let word_idx = index / 64;
        let bit_idx = index % 64;
        while self.data.len() <= word_idx {
            self.data.push(0);
        }
        self.data[word_idx] |= 1 << bit_idx;
    }
    
    fn shl1(&mut self) {
        let mut carry = 0;
        for word in self.data.iter_mut() {
            let next_carry = *word >> 63;
            *word = (*word << 1) | carry;
            carry = next_carry;
        }
        if carry != 0 {
            self.data.push(carry);
        }
    }

    fn set_bit0(&mut self) {
        if self.data.is_empty() {
            self.data.push(1);
        } else {
            self.data[0] |= 1;
        }
    }

    fn bit_len(&self) -> usize {
        if self.is_zero() { return 0; }
        let last_idx = self.data.len() - 1;
        let last_word = self.data[last_idx];
        let bits_in_last = 64 - last_word.leading_zeros() as usize;
        last_idx * 64 + bits_in_last
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if other.is_zero() {
            panic!("Division by zero");
        }
        if self < other {
            return (Self::zero(), self.clone());
        }
        
        let mut quotient = BigUint::zero();
        let mut remainder = BigUint::zero();
        let bit_len = self.bit_len();
        
        for i in (0..bit_len).rev() {
            remainder.shl1();
            if self.get_bit(i) {
                remainder.set_bit0();
            }
            if &remainder >= other {
                remainder = &remainder - other;
                quotient.set_bit(i);
            }
        }
        (quotient, remainder)
    }
}

impl PartialEq for BigUint {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.data.len() != other.data.len() {
            return self.data.len().cmp(&other.data.len());
        }
        for (a, b) in self.data.iter().rev().zip(other.data.iter().rev()) {
            match a.cmp(b) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }
}

impl Add for &BigUint {
    type Output = BigUint;

    fn add(self, other: Self) -> BigUint {
        let max_len = std::cmp::max(self.data.len(), other.data.len());
        let mut result = Vec::with_capacity(max_len + 1);
        let mut carry = 0;

        for i in 0..max_len {
            let a = if i < self.data.len() { self.data[i] } else { 0 };
            let b = if i < other.data.len() { other.data[i] } else { 0 };
            let (sum, c1) = a.overflowing_add(b);
            let (sum, c2) = sum.overflowing_add(carry);
            result.push(sum);
            carry = if c1 || c2 { 1 } else { 0 };
        }
        if carry > 0 {
            result.push(carry);
        }
        BigUint::new(result)
    }
}

impl Sub for &BigUint {
    type Output = BigUint;

    fn sub(self, other: Self) -> BigUint {
        if self < other {
            panic!("Subtraction underflow");
        }
        let mut result = Vec::with_capacity(self.data.len());
        let mut borrow = 0;

        for i in 0..self.data.len() {
            let a = self.data[i];
            let b = if i < other.data.len() { other.data[i] } else { 0 };
            let (diff, b1) = a.overflowing_sub(b);
            let (diff, b2) = diff.overflowing_sub(borrow);
            result.push(diff);
            borrow = if b1 || b2 { 1 } else { 0 };
        }
        BigUint::new(result)
    }
}

impl Mul for &BigUint {
    type Output = BigUint;

    fn mul(self, other: Self) -> BigUint {
        if self.is_zero() || other.is_zero() {
            return BigUint::zero();
        }
        let n = self.data.len();
        let m = other.data.len();
        let mut result = vec![0u64; n + m];

        for i in 0..n {
            let mut carry = 0u128;
            for j in 0..m {
                let product = (self.data[i] as u128) * (other.data[j] as u128) + (result[i + j] as u128) + carry;
                result[i + j] = product as u64;
                carry = product >> 64;
            }
            if carry > 0 {
                result[i + m] += carry as u64;
            }
        }
        BigUint::new(result)
    }
}

impl Rem for BigUint {
    type Output = BigUint;
    fn rem(self, rhs: Self) -> Self::Output {
        let (_, rem) = self.div_rem(&rhs);
        rem
    }
}
impl Rem for &BigUint {
    type Output = BigUint;
    fn rem(self, other: &BigUint) -> BigUint {
        let (_, rem) = self.div_rem(other);
        rem
    }
}

impl FromStr for BigUint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = BigUint::zero();
        let ten = BigUint::from_u64(10);
        
        for c in s.chars() {
            if let Some(digit) = c.to_digit(10) {
                res = &(&res * &ten) + &BigUint::from_u64(digit as u64);
            } else {
                return Err(());
            }
        }
        Ok(res)
    }
}

impl fmt::Display for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        // Very inefficient display but works without big dependencies
        let mut temp = self.clone();
        let ten = BigUint::from_u64(10);
        let mut digits = Vec::new();
        
        while !temp.is_zero() {
            let (q, r) = temp.div_rem(&ten);
            digits.push(r.to_u64());
            temp = q;
        }
        
        for d in digits.iter().rev() {
            write!(f, "{}", d)?;
        }
        Ok(())
    }
}

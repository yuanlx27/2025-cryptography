use std::env::var;
use std::fs::File;
use std::io::{Read, Write, stdin, stdout};

const N: usize = 131;

const I: [u8; 24] = {
    let mut arr = [0u8; 24];
    arr[0] = 0b00000001;
    arr
};

fn main() {
    let mut reader: Box<dyn Read> = if cfg!(feature = "online_judge") {
        Box::new(stdin())
    } else {
        let tc = var("TESTCASE").unwrap();
        Box::new(File::open(format!("samples/sample{tc}_in.bin")).unwrap())
    };
    let mut writer: Box<dyn Write> = if cfg!(feature = "online_judge") {
        Box::new(stdout())
    } else {
        let tc = var("TESTCASE").unwrap();
        Box::new(File::create(format!("samples/sample{tc}_out.bin")).unwrap())
    };

    let n = {
        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer).unwrap();
        u32::from_le_bytes(buffer) as usize
    };

    for _ in 0..n {
        let op_type = {
            let mut buffer = [0u8; 1];
            reader.read_exact(&mut buffer).unwrap();
            buffer[0] as usize
        };

        let a = {
            let mut buffer = [0u8; 24];
            reader.read_exact(&mut buffer).unwrap();
            buffer
        };

        let b = {
            let mut buffer = [0u8; 24];
            reader.read_exact(&mut buffer).unwrap();
            buffer
        };

        let result = match op_type {
            0 => add(a, b),
            1 => mul(a, b),
            2 => square(a),
            3 => invert(a),
            _ => panic!("Invalid operation type."),
        };
        writer.write_all(&result).unwrap();
    }
}

fn get(a: &[u8], i: usize) -> u8 {
    (a[i / 8] >> (i % 8)) & 1
}
fn xor(a: &mut [u8], i: usize, v: u8) {
    a[i / 8] ^= (v & 1) << (i % 8);
}

fn add(a: [u8; 24], b: [u8; 24]) -> [u8; 24] {
    (0..24)
        .map(|i| a[i] ^ b[i])
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

fn mul(a: [u8; 24], b: [u8; 24]) -> [u8; 24] {
    let mut res = [0u8; 48];
    for i in 0..N {
        for j in 0..N {
            xor(&mut res, i + j, get(&a, i) & get(&b, j));
        }
    }
    for i in (N..(N + N - 1)).rev() {
        if get(&res, i) != 0 {
            xor(&mut res, i - N, 1);
            xor(&mut res, i - N + 1, 1);
            xor(&mut res, i - N + 2, 1);
            xor(&mut res, i - N + 13, 1);
        }
    }

    res[0..24].try_into().unwrap()
}

fn square(a: [u8; 24]) -> [u8; 24] {
    let mut res = [0u8; 24];
    for i in 0..N.div_ceil(2) {
        xor(&mut res, i * 2, get(&a, i));
    }
    res
}

fn invert(a: [u8; 24]) -> [u8; 24] {
    todo!()
}

fn exgcd(a: [u8; 24], b: [u8; 24]) -> ([u8; 24], [u8; 24]) {
}

mod finite_field {
    use core::ops::{Add, Mul, Div};

    const N: usize = 131;

    #[allow(non_camel_case_types)]
    struct u131([u128; 4]);

    impl u131 {
        fn new() -> Self {
            Self([0u128; 4])
        }

        fn from_le_bytes(bytes: [u8; 24]) -> Self {
            let mut arr = [0u128; 4];
            arr[0] = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
            arr[1] = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as u128;
            Self(arr)
        }

        fn to_le_bytes(&self) -> [u8; 24] {
            let mut bytes = [0u8; 24];
            bytes[0..16].copy_from_slice(&self.0[0].to_le_bytes());
            bytes[16..24].copy_from_slice(&(self.0[1] as u64).to_le_bytes());
            bytes
        }
    }

    impl Add for u131 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self([
                self.0[0] ^ rhs.0[0],
                self.0[1] ^ rhs.0[1],
                self.0[2] ^ rhs.0[2],
                self.0[3] ^ rhs.0[3],
            ])
        }
    }

    impl Mul for u131 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            let mut a = [0u8; N];
            (0..N).for_each(|i| a[i] = ((self.0[i / 128] >> (i % 128)) & 1) as u8);

            let mut b = [0u8; N];
            (0..N).for_each(|i| b[i] = ((rhs.0[i / 128] >> (i % 128)) & 1) as u8);

            let mut c = [0u8; N + N - 1];
            for i in 0..N {
                for j in 0..N {
                    c[i + j] ^= a[i] & b[j];
                }
            }

            let mut res = Self::new();
            (0..(N + N - 1)).rev().for_each(|i| res.0[i / 128] ^= (c[i] as u128) << (i % 128));

            res
        }
    }

    impl Div for u131 {

    }
}

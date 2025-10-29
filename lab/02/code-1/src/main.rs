use std::fs;
use std::io;
use std::mem::swap;
use std::cmp::{PartialOrd, Ordering};
use std::ops::{Add, Mul, Shl};

const N: usize = 131;

fn main() -> io::Result<()> {
    let mut reader: Box<dyn io::Read> = if cfg!(feature = "online_judge") {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open("sample/input.bin")?)
    };
    let mut writer: Box<dyn io::Write> = if cfg!(feature = "online_judge") {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create("sample/output.bin")?)
    };

    let n = {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        u32::from_le_bytes(buf) as usize
    };

    for _ in 0..n {
        let op_type = {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0]
        };

        let a = {
            let mut buf = [0u8; 24];
            reader.read_exact(&mut buf)?;
            u131::from_le_bytes(buf)
        };

        let b = {
            let mut buf = [0u8; 24];
            reader.read_exact(&mut buf)?;
            u131::from_le_bytes(buf)
        };

        let result = match op_type {
            0 => a + b,
            1 => a * b,
            2 => a.sqr(),
            3 => a.inv(),
            _ => panic!("Invalid operation type."),
        };
        writer.write_all(&result.into_le_bytes())?;
    }

    Ok(())
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default, PartialEq)]
struct u131 {
    data: [u64; 5],
}

const P: u131 = u131 {
    data: [
        0x00000000_00002007,
        0,
        0x00000000_00000008,
        0,
        0,
    ],
};

const ZERO: u131 = u131 { data: [0; 5] };
const ONE: u131 = u131 { data: [1, 0, 0, 0, 0] };

impl PartialOrd for u131 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in (0..5).rev() {
            if self.data[i] < other.data[i] {
                return Some(Ordering::Less);
            } else if self.data[i] > other.data[i] {
                return Some(Ordering::Greater);
            }
        }
        Some(Ordering::Equal)
    }
}

impl Add for u131 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            data: [
                self.data[0] ^ rhs.data[0],
                self.data[1] ^ rhs.data[1],
                self.data[2] ^ rhs.data[2],
                self.data[3] ^ rhs.data[3],
                self.data[4] ^ rhs.data[4],
            ],
        }
    }
}

impl Mul for u131 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut res = Self::default();
        for i in 0..N {
            if (self.data[i / 64] >> (i % 64)) & 1 != 0 {
                res = res + (rhs << i);
            }
        }
        res.rem()
    }
}

// impl Div for u131 {
//     type Output = Self;

//     fn div(self, rhs: Self) -> Self {
//         let mut rem = self;
//         let mut res = Self::default();
//         for i in (0..N).rev() {
//             if rem.len() == rhs.len() + i {
//                 rem = rem + (rhs << i);
//                 res.data[i / 64] |= 1 << (i % 64);
//             }
//         }
//         res
//     }
// }

impl Shl<usize> for u131 {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self {
        let mut res = Self::default();
        let (x, y) = (rhs / 64, rhs % 64);

        for i in (x..5).rev() {
            res.data[i] = self.data[i - x];
        }

        if y == 0 {
            return res;
        }

        for i in (x..5).rev() {
            res.data[i] <<= y;
            if i > 0 {
                res.data[i] ^= res.data[i - 1] >> (64 - y);
            }
        }

        res
    }
}

impl u131 {
    fn from_le_bytes(bytes: [u8; 24]) -> Self {
        Self {
            data: [
                u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
                u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
                u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
                0,
                0,
            ],
        }
    }

    fn into_le_bytes(self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        bytes[0..8].copy_from_slice(&self.data[0].to_le_bytes());
        bytes[8..16].copy_from_slice(&self.data[1].to_le_bytes());
        bytes[16..24].copy_from_slice(&self.data[2].to_le_bytes());
        bytes
    }

    fn deg(&self) -> usize {
        for i in (0..5).rev() {
            if self.data[i] != 0 {
                return i * 64 + self.data[i].ilog2() as usize;
            }
        }
        panic!("Degree of zero polynomial is undefined.");
    }

    fn rem(&self) -> Self {
        let mut res = *self;

        for i in (3..5).rev() {
            res.data[i - 3] ^= res.data[i] << 61 ^ res.data[i] << 62 ^ res.data[i] << 63;
            res.data[i - 2] ^= res.data[i] >> 3 ^ res.data[i] >> 2 ^ res.data[i] >> 1 ^ res.data[i] << 10;
            res.data[i - 1] ^= res.data[i] >> 54;
        }

        res.data[4] = 0;
        res.data[3] = 0;

        let temp = res.data[2] >> 3;
        res.data[0] ^= temp ^ temp << 1 ^ temp << 2 ^ temp << 13;
        res.data[1] ^= temp >> 51;
        res.data[2] &= 0x7;

        res
    }

    fn sqr(&self) -> Self {
        let mut res = Self::default();
        for i in 0..N {
            let j = i * 2;
            res.data[j / 64] |= ((self.data[i / 64] >> (i % 64)) & 1) << (j % 64);
        }
        res.rem()
    }

    fn inv(&self) -> Self {
        self.inv_gcd()
    }

    #[allow(unused)]
    fn inv_gcd(&self) -> Self {
        let (mut u, mut v) = (*self, P);
        let (mut g1, mut g2) = (ONE, ZERO);
        while u != ONE {
            let j = u.deg().checked_sub(v.deg());
            if j == None {
                swap(&mut u, &mut v);
                swap(&mut g1, &mut g2);
                continue;
            }
            let j = j.unwrap();
            u = (u + (v << j)).rem();
            g1 = (g1 + (g2 << j)).rem();
        }
        g1
    }

    #[allow(unused)]
    fn inv_pow(&self) -> Self {
        let mut a = *self;
        let mut res = Self { data: [1, 0, 0, 0, 0] };
        for _ in 0..(N - 1) {
            res = (res * a).rem();
            a = a.sqr().rem();
        }
        res.sqr().rem()
    }
}
use std::fs;
use std::io;
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
#[derive(Clone, Copy, Default)]
struct u131 {
    data: [u128; 3],
}

impl Add for u131 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            data: [
                self.data[0] ^ rhs.data[0],
                self.data[1] ^ rhs.data[1],
                0,
            ],
        }
    }
}

impl Mul for u131 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut res = Self::default();
        for i in 0..N {
            if self.data[i / 128] >> (i % 128) & 1 != 0 {
                res = res + (rhs << i);
            }
        }
        res.rem()
    }
}

impl Shl<usize> for u131 {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self {
        match rhs {
            0 => self,
            1..128 => Self {
                data: [
                    self.data[0] << rhs,
                    self.data[1] << rhs ^ self.data[0] >> (128 - rhs),
                    self.data[2] << rhs ^ self.data[1] >> (128 - rhs),
                ],
            },
            128 => Self {
                data: [
                    0,
                    self.data[0],
                    self.data[1],
                ],
            },
            129..N => Self {
                data: [
                    0,
                    self.data[0] << (rhs - 128),
                    self.data[1] << (rhs - 128) ^ self.data[0] >> (256 - rhs),
                ],
            },
            _ => panic!("Shift amount out of range."),
        }
    }
}

impl u131 {
    fn from_le_bytes(bytes: [u8; 24]) -> Self {
        Self {
            data: [
                u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
                u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as u128,
                0,
            ],
        }
    }

    fn into_le_bytes(self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        bytes[0..16].copy_from_slice(&self.data[0].to_le_bytes());
        bytes[16..24].copy_from_slice(&(self.data[1] as u64).to_le_bytes());
        bytes
    }

    fn rem(&self) -> Self {
        let mut res = *self;

        let x = res.data[2] << 125;
        res.data[0] ^= x ^ (x << 1) ^ (x << 2);

        let x = res.data[2] >> 3;
        res.data[1] ^= x ^ (x << 1) ^ (x << 2) ^ (x << 13);

        let x = res.data[1] >> 3;
        res.data[0] ^= x ^ (x << 1) ^ (x << 2) ^ (x << 13);

        res.data[2] &= 0;
        res.data[1] &= 0x07;

        res
    }

    fn sqr(&self) -> Self {
        todo!()
    }

    fn inv(&self) -> Self {
        todo!()
    }
}

use std::env::var;
use std::fs::File;
use std::io::{Read, Write, stdin, stdout};
use finite_field::u131;

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
            u131::from_le_bytes(buffer)
        };

        let b = {
            let mut buffer = [0u8; 24];
            reader.read_exact(&mut buffer).unwrap();
            u131::from_le_bytes(buffer)
        };

        let result = match op_type {
            0 => a + b,
            1 => a * b,
            _ => panic!("Invalid operation type."),
        };
        writer.write_all(&result.into_le_bytes()).unwrap();
    }
}

mod finite_field {
    use core::cmp::{Ordering, PartialEq, PartialOrd};
    use core::ops::{Add, Mul, Shl};

    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct u131 {
        data: [u128; 3],
    }

    const N: usize = 131;

    const P: u131 = u131 {
        data: [
            0x00000000_00000000_00000000_00002007,
            0x00000000_00000000_00000000_00000008,
            0x00000000_00000000_00000000_00000000,
        ],
    };

    impl Add for u131 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                data: [
                    self.data[0] ^ rhs.data[0],
                    self.data[1] ^ rhs.data[1],
                    self.data[2] ^ rhs.data[2],
                ],
            }
        }
    }

    impl Mul for u131 {
        type Output = Self;

        // fn mul(self, rhs: Self) -> Self::Output {
        //     let mut result = Self::default();
        //     for i in 0..N {
        //         for j in 0..N {
        //             let x = (i / 128, i % 128);
        //             let y = (j / 128, j % 128);
        //             let z = ((i + j) / 128, (i + j) % 128);
        //             result.data[z.0] ^= ((self.data[x.0] >> x.1) & 1) & ((rhs.data[y.0] >> y.1) & 1) << z.1;
        //         }
        //     }
        //     result.rem()
        // }

        fn mul(self, rhs: Self) -> Self::Output {
            let mut result = Self::default();
            for i in 0..N {
                let x = (i / 128, i % 128);
                if (self.data[x.0] >> x.1) & 1 != 0 {
                    result = result + (rhs << i);
                }
            }
            result.rem()
        }
    }

    impl PartialOrd for u131 {
        fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
            for i in (0..3).rev() {
                if self.data[i] < rhs.data[i] {
                    return Some(Ordering::Less);
                } else if self.data[i] > rhs.data[i] {
                    return Some(Ordering::Greater);
                }
            }
            Some(Ordering::Equal)
        }
    }

    impl Shl<usize> for u131 {
        type Output = Self;

        fn shl(self, rhs: usize) -> Self::Output {
            let mut result = Self::default();
            for i in 0..3 {
                let x = (rhs / 128, (rhs % 128) as u32);
                if i >= x.0 {
                    result.data[i] |= self.data[i - x.0].wrapping_shl(x.1);
                    if i > x.0 {
                        result.data[i] |= self.data[i - x.0 - 1].wrapping_shr(128 - x.1);
                    }
                }
            }
            result
        }
    }

    impl u131 {
        pub fn from_le_bytes(bytes: [u8; 24]) -> Self {
            Self {
                data: [
                    u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
                    u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as u128,
                    0,
                ],
            }
        }

        pub fn into_le_bytes(self) -> [u8; 24] {
            let mut bytes = [0u8; 24];
            bytes[0..16].copy_from_slice(&self.data[0].to_le_bytes());
            bytes[16..24].copy_from_slice(&(self.data[1] as u64).to_le_bytes());
            bytes
        }

        // fn rem(self) -> Self {
        //     let mut result = self;
        //     result.data[0] ^= result.data[1].wrapping_shr(3) ^ result.data[1].wrapping_shr(2) ^ result.data[1].wrapping_shr(1) ^ result.data[1].wrapping_shl(10);
        //     result.data[0] ^= result.data[2].wrapping_shl(125) ^ result.data[2].wrapping_shl(126) ^ result.data[2].wrapping_shl(127);
        //     result.data[1] ^= result.data[2].wrapping_shr(3) ^ result.data[2].wrapping_shr(2) ^ result.data[2].wrapping_shr(1) ^ result.data[2].wrapping_shl(10);
        //     result.data[1] &= 0x7;
        //     result.data[2] &= 0;
        //     result
        // }

        fn rem(self) -> Self {
            let mut result = self;
            for i in (0..N).rev() {
                if result >= (P << i) {
                    result = result + (P << i);
                }
            }
            result
        }
    }
}

use std::{
    env::var,
    fs::File,
    io::{Read, Write, stdin, stdout},
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

    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer).unwrap();
    let op_count = u32::from_le_bytes(buffer) as usize;

    for _ in 0..op_count {
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer).unwrap();
        let op_type = buffer[0];
    }
}

#[allow(non_camel_case_types)]
type u131 = [u64; 3];

trait IntExt {
    fn add(&self, rhs: &Self) -> Self;
    fn mul(&self, rhs: &Self) -> Self;
    fn square(&self) -> Self;
}

impl IntExt for u131 {
    fn add(&self, rhs: &Self) -> Self {
        [
            self[0] ^ rhs[0],
            self[1] ^ rhs[1],
            self[2] ^ rhs[2],
        ]
    }

    fn mul(&self, rhs: &Self) -> Self {
        let mut a = [0u8; 131];
        for i in 0..131 {
            a[i] = (self[i / 64] >> (i % 64) & 1) as u8;
        }

        let mut b = [0u8; 131];
        for i in 0..131 {
            b[i] = (rhs[i / 64] >> (i % 64) & 1) as u8;
        }

        let mut c = [0u8; 262];
        for i in 0..131 {
            for j in 0..131 {
                c[i + j] ^= a[i] & b[j];
            }
        }

        todo!()
    }

    fn square(&self) -> Self {
        todo!()
    }
}

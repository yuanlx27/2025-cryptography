use std::{env::var, fs::File, io::{Read, Write, stdin, stdout}, iter::once};

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

        let mut buffer = [0u8; 24];
        reader.read_exact(&mut buffer).unwrap();
        let a = u131::from_le_bytes(buffer);

        let mut buffer = [0u8; 24];
        reader.read_exact(&mut buffer).unwrap();
        let b = u131::from_le_bytes(buffer);
    }
}

#[allow(non_camel_case_types)]
struct u131(u128, u8);

impl u131 {
    fn from_le_bytes(bytes: [u8; 24]) -> Self {
        Self(u128::from_le_bytes(bytes[0..16].try_into().unwrap()), bytes[16])
    }

    fn to_le_bytes(&self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        bytes[0..16].copy_from_slice(&self.0.to_le_bytes());
        bytes[16] = self.1;
        bytes
    }
}

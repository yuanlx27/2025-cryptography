use std::fs;
use std::io;

fn main() {
    let mut reader: Box<dyn io::Read> = if cfg!(feature = "online_judge") {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open("samples/input.bin").unwrap())
    };
    let mut writer: Box<dyn io::Write> = if cfg!(feature = "online_judge") {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create("samples/output.bin").unwrap())
    };

    let mut buffer = Vec::<u8>::new();
    reader.read_to_end(&mut buffer).unwrap();
    writer.write_all(&sha256::digest(&buffer)).unwrap();
}

mod sha256 {
    struct SHA256 {
        h: [u32; 8],
    }

    impl SHA256 {
        fn new() -> Self {
            SHA256 {
                h: H0,
            }
        }

        fn update(&mut self, text: &[u8]) {
        }

        fn finalize(&self) -> [u8; 32] {
        }
    }

    pub fn digest(text: &[u8]) -> [u8; 32] {
        let mut ctx = SHA256::new();
        ctx.update(text);
        ctx.finalize()
    }

    const H0: [u32; 8] = [
        0x6a09e667,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
}

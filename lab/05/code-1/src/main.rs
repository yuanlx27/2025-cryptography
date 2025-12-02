use std::io;
use std::io::Read;

fn main() -> io::Result<()> {
    // Lock stdin.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // Discard first 16 bytes.
    let mut _discard = [0u8; 16];
    handle.read_exact(&mut _discard)?;

    // Parse n.
    let n = {
        let mut buf = [0u8; 256];
        handle.read_exact(&mut buf)?;
        BigUint::from_be_bytes(&buf)
    };

    // Parse e.
    let e = {
        let mut buf = [0u8; 256];
        handle.read_exact(&mut buf)?;
        BigUint::from_be_bytes(&buf)
    };

    // Discard following 256 bytes.
    let mut _discard = [0u8; 256];
    handle.read_exact(&mut _discard)?;

    // Parse message length l.
    let l = {
        let mut buf = [0u8; 1];
        handle.read_exact(&mut buf)?;
        buf[0] as usize
    }

    // Parse message m.
    let m = {
        let mut buf = vec![0u8; l];
        handle.read_exact(&mut buf)?;
        buf
    }

    Ok(())
}

struct BigUint {
    data: [u64; 65],
}

impl BigUint {
    fn new() -> Self {
        BigUint { data: [0u64; 65] }
    }

    fn from_be_bytes(bytes: &[u8]) -> Self {
        // Input must be exactly 256 bytes.
        assert_eq!(bytes.len(), 256);

        let mut data = [0u64; 65];
        bytes.chunks(8).enumerate().for_each(|(i, chunk)| {
            data[i] = u64::from_be_bytes(chunk.try_into().unwrap());
        });

        Self { data }
    }
}

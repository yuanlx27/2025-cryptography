use std::fs;
use std::io;

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

    let x = {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        buf[0]
    };

    let n = {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        u32::from_le_bytes(buf) as usize
    };

    let y = {
        let mut buf = vec![0u8; n];
        reader.read_exact(&mut buf)?;
        buf
    };

    let result = y.iter().map(|b| b ^ x).collect::<Vec<u8>>();

    writer.write_all(&result)?;

    Ok(())
}

use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let mut reader: Box<dyn io::Read> = if cfg!(feature = "online_judge") {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open("samples/sample_input.bin")?)
    };

    let mut x_buf = [0u8; 1];
    reader.read_exact(&mut x_buf)?;
    let x = x_buf[0];

    let mut n_buf = [0u8; 4];
    reader.read_exact(&mut n_buf)?;
    let n = u32::from_le_bytes(n_buf) as usize;

    let mut y = vec![0u8; n];
    reader.read_exact(&mut y)?;
    let result: Vec<u8> = y.iter().map(|b| b ^ x).collect();

    let mut writer: Box<dyn io::Write> = if cfg!(feature = "online_judge") {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create("samples/sample_output.bin")?)
    };

    writer.write_all(&result)?;

    Ok(())
}

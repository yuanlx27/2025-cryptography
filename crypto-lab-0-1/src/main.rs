use std::io;

fn rin() -> usize {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input.trim().parse().unwrap();
}

fn main() {

    let (a, b, c) = (rin(), rin(), rin());
    println!("{} {}", (a + b) % c, a * b % c);
}

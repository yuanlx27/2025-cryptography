use std::io;

fn rin() -> i32 {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap()
}

fn main() {
    let (a, b, c) = (rin(), rin(), rin());
    println!("{} {}", (a + b) % c, a * b % c);
}

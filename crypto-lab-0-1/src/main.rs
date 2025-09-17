use std::io;
use std::io::Read;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let args: Vec<i32> = input.split_whitespace().map(|s| s.parse().unwrap()).collect();

    let (a, b, c) = (args[0], args[1], args[2]);
    println!("{} {}", (a + b) % c, a * b % c);
}

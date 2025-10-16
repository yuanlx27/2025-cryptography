fn exgcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x1, y1) = exgcd(b, a % b);
        (g, y1, x1 - (a / b) * y1)
    }
}

fn inv_exgcd(a: i64, m: i64) -> Option<i64> {
    let (g, x, _) = exgcd(a, m);
    if g != 1 { None } else { Some((x % m + m) % m) }
}

fn binexp(mut a: i64, mut b: i64, m: i64) -> i64 {
    let mut c = 1;
    while b > 0 {
        if b % 2 == 1 { c = c * a % m; }
        a = a * a % m; b /= 2;
    }
    c
}

fn inv_fermat(a: i64, p: i64) -> Option<i64> {
    if a % p == 0 { None } else { Some(binexp(a, p - 2, p)) }
}

fn main() {
    println!("Hello, world!");
}

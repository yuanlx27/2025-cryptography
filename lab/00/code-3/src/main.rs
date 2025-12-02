fn main() {
}

mod num {
    use std::cmp::{PartialOrd, Ordering};
    use std::ops::{Add, AddAssign};

    pub struct BigUint {
        digits: Vec<u64>,
    }

    impl BigUint {
    }

    impl Add for BigUint {
        type Output = BigUint;

        fn add(mut self, other: BigUint) -> BigUint {
            self += other;
            self
        }
    }
    impl AddAssign for BigUint {
        fn add_assign(&mut self, other: BigUint) {
            // Implementation goes here
        }
    }

    pub struct BigInt {
        sign: bool,
        digits: BigUint,
    }

    impl BigInt {
    }

    impl Add for BigInt {
        type Output = BigInt;

        fn add(mut self, other: BigInt) -> BigInt {
            self += other;
            self
        }
    }
    impl AddAssign for BigInt {
        fn add_assign(&mut self, other: BigInt) {
            if self.sign == other.sign {
                self.digits += other.digits;
            } else {
                if self.digits < other.digits {
                    self.sign = other.sign;
                }
                self.digits -= other.digits;
            }
        }
    }
}

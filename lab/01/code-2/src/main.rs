use crate::matrix::Matrix;

use std::io;

fn main() {
    let (key_dimension, plaintext, ciphertext) = read_inputs();

    let (matrix, vector) = hill_cipher::crack(key_dimension, &plaintext, &ciphertext);

    write_outputs(&matrix, &vector);
}

fn read_inputs() -> (usize, String, String) {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    let key_dimension: usize = line.trim().parse().unwrap();

    line.clear();
    io::stdin().read_line(&mut line).unwrap();

    let plaintext: String = line.trim().to_string();

    line.clear();
    io::stdin().read_line(&mut line).unwrap();

    let ciphertext: String = line.trim().to_string();

    (key_dimension, plaintext, ciphertext)
}

fn write_outputs(matrix: &Matrix, vector: &[i32]) {
    print!("{}", matrix);

    vector.iter().for_each(|val| print!("{} ", val));
    println!();
}

mod hill_cipher {
    use crate::matrix::Matrix;

    /// Cracks the Hill cipher with given key dimension, plaintext, and ciphertext.
    pub fn crack(key_dim: usize, plaintext: &str, ciphertext: &str) -> (Matrix, Vec<i32>) {
        let mut x = Matrix::new(key_dim, key_dim);
        let mut y = Matrix::new(key_dim, key_dim);

        let p_nums = plaintext.as_bytes();
        let c_nums = ciphertext.as_bytes();
        p_nums.chunks(key_dim)
            .zip(c_nums.chunks(key_dim))
            .skip(1)
            .take(key_dim)
            .enumerate()
            .for_each(|(i, (p_chunk, c_chunk))| {
                p_chunk.iter().enumerate().for_each(|(j, p)| {
                    x[(i, j)] = ((p + 26 - p_nums[j]) % 26) as i32;
                });
                c_chunk.iter().enumerate().for_each(|(j, c)| {
                    y[(i, j)] = ((c + 26 - c_nums[j]) % 26) as i32;
                });
            });
        let mut pos = key_dim * key_dim;

        let key_matrix = loop {
            if let Some(x_inv) = x.inv() {
                break x_inv * y;
            }

            pos += key_dim;
            p_nums[pos..pos + key_dim].iter()
                .zip(c_nums[pos..pos + key_dim].iter())
                .enumerate()
                .for_each(|(j, (p, c))| {
                    x[(0, j)] = ((p + 26 - p_nums[j]) % 26) as i32;
                    y[(0, j)] = ((c + 26 - c_nums[j]) % 26) as i32;
                });
        };

        let p_nums: Vec<u8> = p_nums.iter().map(|b| b - b'A').collect();
        let c_nums: Vec<u8> = c_nums.iter().map(|b| b - b'A').collect();

        let mut key_vector = vec![0; key_dim];
        for i in 0..key_dim {
            key_vector[i] = c_nums[i] as i32;
            for j in 0..key_dim {
                key_vector[i] -= p_nums[j] as i32 * key_matrix[(j, i)];
            }
            key_vector[i] = ((key_vector[i] % 26) + 26) % 26;
        }

        (key_matrix, key_vector)
    }
}

mod matrix {
    use std::cmp::min;
    use std::fmt::{Display, Formatter, Result};
    use std::ops::{Index, IndexMut, Mul};

    const INV: [i32; 26] = [
        0, 1, 0, 9, 0, 21, 0, 15, 3, 3, 0, 19, 0, 0, 0, 7, 0, 23, 0, 11, 0, 5, 0, 17, 0, 25,
    ];

    #[derive(Clone, Default)]
    pub struct Matrix {
        rows: usize,
        cols: usize,
        data: Vec<i32>,
    }

    impl Matrix {
        pub fn new(rows: usize, cols: usize) -> Self {
            Self {
                rows,
                cols,
                data: vec![0i32; rows * cols],
            }
        }

        pub fn inv(&self) -> Option<Self> {
            if self.rows != self.cols {
                return None;
            }

            let mut augmented = Matrix::new(self.rows, self.cols * 2);
            for i in 0..self.rows {
                for j in 0..self.cols {
                    augmented[(i, j)] = self[(i, j)];
                }
                augmented[(i, i + self.cols)] = 1;
            }
            let augmented = augmented.gaussian_elimination()?;

            let mut result = Matrix::new(self.rows, self.cols);
            for i in 0..self.rows {
                for j in 0..self.cols {
                    result[(i, j)] = augmented[(i, j + self.cols)];
                }
            }
            Some(result)
        }

        fn gaussian_elimination(&self) -> Option<Self> {
            let mut result = self.clone();
            for col in 0..min(result.rows, result.cols) {
                let mut pivot_row = None;
                for row in col..result.rows {
                    if INV[result[(row, col)] as usize] != 0 {
                        pivot_row = Some(row);
                        break;
                    }
                }
                let pivot_row = pivot_row?;

                if pivot_row != col {
                    for pos in 0..result.cols {
                        let temp = result[(pivot_row, pos)];
                        result[(pivot_row, pos)] = result[(col, pos)];
                        result[(col, pos)] = temp;
                    }
                }

                let factor = INV[result[(col, col)] as usize];
                for pos in col..result.cols {
                    result[(col, pos)] *= factor;
                    result[(col, pos)] %= 26;
                }

                for row in 0..result.rows {
                    if row == col {
                        continue;
                    }

                    let factor = result[(row, col)];
                    if factor == 0 {
                        continue;
                    }

                    for pos in col..result.cols {
                        result[(row, pos)] -= factor * result[(col, pos)];
                        result[(row, pos)] = (result[(row, pos)] % 26 + 26) % 26;
                    }
                }
            }
            Some(result)
        }
    }

    impl Display for Matrix {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            for i in 0..self.rows {
                for j in 0..self.cols {
                    write!(f, "{} ", self[(i, j)])?;
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }

    impl Index<(usize, usize)> for Matrix {
        type Output = i32;

        fn index(&self, index: (usize, usize)) -> &Self::Output {
            &self.data[index.0 * self.cols + index.1]
        }
    }

    impl IndexMut<(usize, usize)> for Matrix {
        fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
            &mut self.data[index.0 * self.cols + index.1]
        }
    }

    impl Mul for Matrix {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            let mut result = Matrix::new(self.rows, rhs.cols);
            for i in 0..self.rows {
                for j in 0..rhs.cols {
                    for k in 0..self.cols {
                        result[(i, j)] += self[(i, k)] * rhs[(k, j)];
                    }
                    result[(i, j)] %= 26;
                }
            }
            result
        }
    }
}

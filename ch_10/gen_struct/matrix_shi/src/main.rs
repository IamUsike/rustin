/* Const generics: fixed size matrix
- Define Matrix<T, const R: usize, const C: usize> backed by [[T;C];R].
- implement:
    - new() where T: Default + Copy,
    - get(row, col) -> &T
    - set(row, col, val: T)
    - transpose(&self) -> Matrix<T, C, R> where T: Copy.
- write a fn Multiply<T, const N: usize, M: usize, const P: usize> (a: &Matrix<T, N, M>, b: &Matrix<T, M, P>)
-> Matrix<T, N, P> where T: Default + Copy + Add<Output = T> + Mul<Output = T>
*/

use num_traits::identities::Zero;
use std::fmt::Display;
use std::ops::{Add, Mul};

struct Matrix<T, const R: usize, const C: usize> {
    mat: Vec<Vec<T>>,
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C> {
    fn new() -> Self
    where
        T: Default + Copy,
    {
        //T default is required here cos when building a vec<T>, rust needs some type of
        //value to put in each slot. For a generic T, the compiler doesnt guess what T will be
        //so either use default of have the caller provide the value
        let mat = vec![vec![T::default(); C]; R];
        Self { mat }
    }

    fn get(&self, row: usize, col: usize) -> &T {
        &self.mat[row][col]
    }

    fn set(&mut self, row: usize, col: usize, val: T) {
        self.mat[row][col] = val;
    }

    fn transpose(&self) -> Matrix<T, C, R>
    where
        T: Copy + Default,
    {
        let mut mat = vec![vec![T::default(); R]; C]; // [C][R]
        for i in 0..R {
            for j in 0..C {
                mat[j][i] = self.mat[i][j];
            }
        }
        Matrix { mat }
    }

    fn display(&self)
    where
        T: Display,
    {
        for row in &self.mat {
            for cell in row {
                print!("{} ", cell);
            }
            println!();
        }
    }
}

fn multiply<T, const N: usize, const P: usize, const M: usize>(
    a: &Matrix<T, N, M>,
    b: &Matrix<T, M, P>,
) -> Matrix<T, N, P>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T> + Zero,
{
    // (N x M) * (M x P) = (N x P)
    let mut res = vec![vec![T::default(); P]; N];

    for i in 0..N {
        for j in 0..P {
            let mut dot_row: T = T::zero(); //Ideally should be 'None' | Option<i32>
            for k in 0..M {
                let prod = a.mat[i][k] * b.mat[k][j];
                dot_row = dot_row + prod;
            }
            res[i][j] = dot_row;
        }
    }

    return Matrix { mat: res };
}

fn main() {
    const R: usize = 4;
    const C: usize = 4;
    //I need to be mut when set method is called ?
    let mut matrix = Matrix::<i32, R, C>::new();
    //Iput values in the matrix
    for i in 0..R {
        for j in 0..C {
            let v = (i * C + j) as i32; // 0,1,2,...
            matrix.set(i, j, v);
        }
    }

    println!("get some shi, {}", matrix.get(0, 0));
    let transpose = matrix.transpose();
    matrix.display();
    println!("w");
    transpose.display();

    ///mult
    const N: usize = 2;
    const M: usize = 3;
    const P: usize = 2;

    let mut a = Matrix::<i32, N, M>::new();
    a.set(0, 0, 1);
    a.set(0, 1, 2);
    a.set(0, 2, 3);
    a.set(1, 0, 4);
    a.set(1, 1, 5);
    a.set(1, 2, 6);

    let mut b = Matrix::<i32, M, P>::new();
    b.set(0, 0, 7);
    b.set(0, 1, 8);
    b.set(1, 0, 9);
    b.set(1, 1, 10);
    b.set(2, 0, 11);
    b.set(2, 1, 12);

    let res = multiply::<i32, N, P, M>(&a, &b);
    println!("mult stuff start");
    a.display();
    println!();
    b.display();
    println!("mult stuff resssss");
    res.display();
}

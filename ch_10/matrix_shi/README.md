# Const generics: fixed-size matrix

Define Matrix<T, const R: usize, const C: usize> backed by [[T; C]; R]. Implement: new() where T: Default + Copy, get(row, col) -> &T, set(row, col, val: T), and fn transpose(&self) -> Matrix<T, C, R> where T: Copy. Write a fn multiply<T, const N: usize, const M: usize, const P: usize>(a: &Matrix<T,N,M>, b: &Matrix<T,M,P>) -> Matrix<T,N,P> where T: Default + Copy + Add<Output=T> + Mul<Output=T>.

> What this cements: Const generics encode size at the type level — Matrix<f64,3,3> and Matrix<f64,3,4> are different types. Multiplying wrong-dimension matrices is a compile error, not a runtime panic.

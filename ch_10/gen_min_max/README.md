# Generic min/max without std

Write two functions: fn min_of(a: T, b: T) -> T and fn max_of(a: T, b: T) -> T. Then write fn clamp(val: T, lo: T, hi: T) -> T. Test with i32, f64, and char. Then deliberately try calling clamp with a type that doesn't implement PartialOrd — read and explain the compiler error in a comment.

> What this cements: T: Bound syntax, why PartialOrd not Ord (f64 has NaN), monomorphization means three separate compiled versions for i32/f64/char.

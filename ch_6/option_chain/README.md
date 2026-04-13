**medium**

**Option chain**

Write `fn safe_divide(a: f64, b: f64) -> Option` and `fn safe_sqrt(x: f64) -> Option`. Then write `fn compute(a: f64, b: f64) -> Option` that divides then takes the square root, using `if let` or `?` operator (once you’ve seen it).

_Hint: safe_sqrt returns None if x < 0. Use if let Some(x) = safe_divide(...) to chain._

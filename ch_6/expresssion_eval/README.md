**hard**

**Tiny expression evaluator**

Define an `Expr` enum: `Num(f64)`, `Add(Box, Box)`, `Mul(Box, Box)`. Write `fn eval(e: &Expr) -> f64` using recursive match. Manually construct and evaluate: `(2 + 3) * (4 + 1)`.

_Hint: Box is needed for recursive types. eval calls itself recursively on the inner expressions._

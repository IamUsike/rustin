# Operator overloading via std traits

Build a Vec2 struct (x: f64, y: f64). Implement: Add (v1 + v2), Sub, Neg (-v), Mul<f64> (scalar multiply: v * 2.0), PartialEq, Display (format as "(x, y)"), and a dot() method. Then impl From<(f64,f64)> for Vec2 and Into automatically. Write a fn normalize(v: Vec2) -> Vec2 that divides by magnitude — use your own Div impl. Test: (Vec2::from((3.0,4.0)) * 2.0 + Vec2::from((1.0,0.0))).dot(&Vec2::from((1.0,0.0))).

> What this cements: std::ops traits are just regular traits with special syntax sugar. From/Into are a pair — impl From and you get Into for free. This pattern is everywhere in real Rust codebases.

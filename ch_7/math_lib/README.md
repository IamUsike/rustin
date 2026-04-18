**Ch 7 · Modules**
**medium**

**Modular math library**

Create a library crate with modules: `math::stats` (mean, median, mode) and `math::convert` (celsius_to_fahrenheit, km_to_miles). Use `pub` correctly. Write a binary that uses it with `use` paths.

_Hint: mod stats; inside lib.rs, then pub fn mean(...) inside stats.rs. Use `use mylib::math::stats::mean;` in main._

---

`cargo new math_lib --lib`

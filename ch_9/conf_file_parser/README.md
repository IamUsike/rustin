**Config file parser**

Parse a config file format (KEY=VALUE, one per line, # comments). Return a HashMap. Return a custom error enum `ConfigError` with variants: `MalformedLine(usize)`, `EmptyKey(usize)`, `DuplicateKey(String)`.

_Hint: Use `enumerate()` to track line numbers. Define enum `ConfigError` and implement `std::fmt::Display` for it._

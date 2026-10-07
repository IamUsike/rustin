- rust doesnt have exceptions but rather error values and the rust type system forces you to handle them

## Undrecoverable Errors: `panic`

- A panic means "this is a bug or an impossible state, stop"
- It unwinds the stack or aborts, depending on the config.

eg: out of bounds access.

## Recoverable: `Result<T,E>`

- any fn that can fail returns one, so the caller cant ignore failure by accident. The compiler warns on an unused `Result`

```rust
use std::fs::File;

match File::open("config.toml") {
  Ok(file) => ...,
  Err(e) => ...
}
```

### Handling `Result`: The toolbox

```rust
let r: Result<i32, _> = "42".parse::<i32>();

r.unwrap();                    // panics on Err (fine in tests/prototypes)
r.expect("must be a number");  // panics with your message
r.unwrap_or(0);                // default value
r.unwrap_or_else(|e| { log(e); 0 });
r.map(|n| n * 2);              // transform Ok, pass Err through
r.and_then(|n| checked(n));    // chain fallible steps
r.ok();                        // Result -> Option, discarding the error
```

#### the `?` operator

- this is the idiomatic way to propagate errors
- means: if `Err`, return it from this function early; if `Ok`, unwrap the value.

```rust
use std::{fs, io};

fn read_username() -> Result<String, io::Error> {
    let s = fs::read_to_string("user.txt")?;
    Ok(s.trim().to_string())
}
```

`?` also calls `From::from` on the error, which is what lets you convert between error types automatically. It works on `Option` too, in functions returning `Option`.

#### Custom error types.

Real programs combine several failure sources, so you define an enum:

```rust
use std::{fmt, io, num::ParseIntError};

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    Parse(ParseIntError),
    Missing(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "io error: {e}"),
            ConfigError::Parse(e) => write!(f, "parse error: {e}"),
            ConfigError::Missing(k) => write!(f, "missing key: {k}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self { ConfigError::Io(e) }
}
impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self { ConfigError::Parse(e) }
}

fn load_port() -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string("port.txt")?; // io::Error -> ConfigError
    let port = text.trim().parse::<u16>()?;          // ParseIntError -> ConfigError
    Ok(port)
}
```

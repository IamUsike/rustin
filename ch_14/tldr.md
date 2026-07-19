# 14. More about Cargo and Crates.io

14.1 Customizing Builds with Release Profiles
14.2 Publishing a Crate to Crates.io
14.3 Installing Binaries with cargo install
14.4 Extending Cargo with Custom Commands

---

## Customizing Builds with Release Profiles

In Rust, _release profiles_ are predefined, customizable profiles with different configurations that allow a programmer to have more control over various options for compiling code. Each profile is configured independently of the others.

Cargo has 2 profiles

- `dev` profile : Cargo uses this when you run `cargo build`. This profile is defined with good defaults for development.
- `release` profile: Cargo uses this when you run `cargo build --release`. This has good defaults for release builds.

Cargo has default settings for each of the profiles that apply when you haven’t explicitly added any `[profile.*]` sections in the project’s Cargo.toml file. By adding `[profile.*]` sections for any profile you want to customize, you override any subset of the default settings. For example, here are the default values for the opt-level setting for the dev and release profiles:

```toml
[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3
```

For the full list of configuration options and defaults for each profile, see Cargo’s [documentation](https://doc.rust-lang.org/cargo/reference/profiles.html).

---

## Publishing a Crate to Creates.io

read the book for this broda

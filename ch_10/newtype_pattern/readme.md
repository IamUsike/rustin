# Newtype pattern + generic wrapper

Build Validated<T> — a wrapper that holds a T only if it passed a validation check. Define a trait Validate { fn validate(&self) -> Result<(), String>; }. Impl Validated<T: Validate> with fn new(val: T) -> Result<Self, String> (returns Err if validate fails) and fn inner(&self) -> &T. Implement Validate for: Email (must contain @), Port (must be 1–65535 as u16), NonEmptyString. Show that Validated<Email> and Validated<Port> are completely separate types at compile time.

> What this cements: Newtypes + generics + trait bounds create domain-specific types. A Validated<Email> can never hold an invalid email — the invariant is in the type, not scattered across if-checks.

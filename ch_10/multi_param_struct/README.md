# Struct with multiple lifetime params

Build a StrParser<'input, 'schema> struct that holds a reference to an input string ('input) and a reference to a schema/pattern string ('schema) — these may have different lifetimes. Add: fn new(input: &'input str, schema: &'schema str) -> Self, fn matches(&self) -> bool (simple: check if input starts with schema), fn remaining(&self) -> &'input str (returns the rest of input after the schema — note: lifetime is 'input not 'schema). Write a main that proves 'input and 'schema can have genuinely different lifetimes by letting the schema go out of scope while remaining() is still valid.

> What this cements: Multiple lifetime params let you express that outputs borrow from ONE specific input, not all inputs. remaining() borrows from input but NOT from schema — if both were 'a you couldn't express this.

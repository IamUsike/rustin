# Blanket impl + extension trait

Define a trait Describe { fn describe(&self) -> String; }. Write a blanket impl: impl<T: Display + Debug> Describe for T — so every type that is both Display and Debug gets a free describe() that returns "display: {self}, debug: {self:?}". Verify it works on i32, String, Vec<i32>, your Vec2 from challenge 6. Then define a second trait IterDescribe for anything that is IntoIterator where Item: Describe — giving it a fn describe_all(&self) -> Vec<String>. Impl it as a blanket.

> What this cements: Blanket impls are how std works internally — impl<T: Display> ToString for T is why every Display type has .to_string() for free. This is one of Rust's most powerful patterns.

# Builder pattern with typestate

Build a RequestBuilder using typestate — phantom type params encode what fields have been set. Define marker types: struct NoUrl; struct HasUrl; struct NoMethod; struct HasMethod. RequestBuilder<U, M> starts as RequestBuilder<NoUrl, NoMethod>. fn set_url(self, url: &str) -> RequestBuilder<HasUrl, M>. fn set_method(self, m: &str) -> RequestBuilder<U, HasMethod>. fn build(self) -> Request only exists on RequestBuilder<HasUrl, HasMethod>. Calling build() before setting both URL and method is a compile error — not a runtime error.

> What this cements: Type parameters can encode state. This is the typestate pattern — widely used in embedded Rust (HAL crates), protocol implementations, and builder APIs. You've turned runtime validation into compile-time enforcement.

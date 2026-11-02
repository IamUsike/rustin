## Routing Fundamentals

Axum's `Router` maps HTTP methods to handler functions.

```rust
use axum::{routing::{get, post}, Router};

async fn list_users() -> &'static str { "all users" }
async fn create_user() -> &'static str { "created" }
async fn get_user() -> &'static str { "one user" }

let app = Router::new()
    .route("/users", get(list_users).post(create_user))
    .route("/users/{id}", get(get_user));
```

- nested routers
- merging routers

---

## Handler Functions and Responses

A handler is any async function that:

1. takes zero or more **extractors** as args.
2. Returns something that implements `IntoResponse`

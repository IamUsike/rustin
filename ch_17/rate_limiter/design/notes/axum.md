1. HTTP basics (half a day)

- What a request and a response look like: method, path, headers, body; status code, headers, body.
- Status codes you'll return: 200, 401 (who are you?), 429 (slow down), 503 (overloaded).
- The headers you'll use:
  - Cookie, and how cookie: a=1; b=2 is formatted.
  - Authorization: Bearer <token>
  - Retry-After
  - X-Forwarded-For: understand why it's a comma-separated list.
- curl, which you'll test everything with:
  - -i shows the response headers.
  - -H 'Authorization: Bearer x' sets a header.
  - -b 'session=x' sends a cookie.
  - -X POST changes the method.
  - Practice against any public API first.

2. axum (1–2 days)

Same list as before:

- the crate root docs: routing, handlers, extractors, responses, State
- middleware::from_fn_with_state
- ConnectInfo
- the hello-world, key-value-store and jwt examples

Then build the warm-up toy server.

3. Rust that axum leans on

- impl IntoResponse as a return type: "returns some type that implements this trait". TRPL ch 10.
- Returning Result<T, StatusCode> from handlers, so ? turns errors into responses.
- Extractor order: extractors that read the request body must come last among the handler's arguments, because the body can only be read once.
- Arc in State: axum clones your state for every request, so state is usually Arc<Something>. Your RateLimiter goes behind one.

4. JWT (2 hours)

- The structure: header.payload.signature, each part base64url-encoded. Paste any token into jwt.io and look inside.
- The claims you'll use: sub (the user ID, your rate limit key) and exp (expiry).
- HS256 vs RS256: a shared secret versus a public/private key pair. Use HS256 locally.
- The key insight: the payload is readable by anyone. It's signed, not encrypted. Verifying the signature is what stops forgery.
- Then the jsonwebtoken crate: encode, decode, Validation.

5. Middleware ordering (30 min, easy to get wrong)

- Each .layer() call wraps everything added before it. The layer added last runs first on the incoming request. Your IP tier must run before the user tier, so think about which order to add them in.
- layer vs route_layer: the first applies to everything, including unmatched routes (404s). The second applies only to routes that match. Read the docs and decide which your limiter wants.

A tip for axum's error messages

axum's compile errors are famously cryptic, like "the trait Handler<_, _> is not implemented". Add the #[axum::debug_handler] attribute (it needs the macros feature) to a handler, and you'll get a readable error instead.

# forge roadmap

A NestJS-style backend framework for Rust, built **from scratch with zero external crates**. Everything is written on top of Rust's standard library (`std`).

Legend: `[x]` done · `[ ]` to do · ⭐ = do this next

---

## The zero-dependency rule

- No crates from crates.io in any `Cargo.toml`. Check with `cargo tree`: it should list only `forge-*` crates.
- An exception is allowed only when building it yourself would be **unsafe** (security code) or **impossible with `std` alone**. Every exception goes in the table below with a reason.

### Allowed exceptions (candidates — none used yet)

| Need | Why not build it | What to do |
|---|---|---|
| HTTPS / TLS | Rolling your own TLS is a classic source of serious vulnerabilities | Run forge behind a reverse proxy (Caddy, nginx) that handles HTTPS. forge stays plain HTTP, so no crate is needed. |
| Password hashing | Must be slow, salted and resistant to timing attacks; easy to get subtly wrong | Use a vetted Argon2 crate when `forge-auth` arrives |
| JWT signing (HMAC-SHA256) | Same as above | Vetted crate, or implement SHA-256/HMAC from the spec **only** with the official test vectors and a constant-time comparison |
| Secure random numbers | `std` has no RNG | On Linux/macOS read `/dev/urandom` with `std::fs` (no crate needed). Windows needs an OS call or a crate. |
| SQLite | It's a C library | Use it through a crate, or write a Postgres client from scratch (it's a documented TCP protocol) |

Non-security randomness (e.g. picking a random item, like `Math.random`) can be a tiny generator you write yourself (xorshift or PCG), seeded from the clock or `/dev/urandom`.

---

## Phase 0 — Foundation ✅

- [x] Cargo workspace, no external crates
- [x] Crates: `core`, `macros`, `di`, `http`, `json`, `validation`, `config`
- [x] `forge-http`: HTTP/1.1 parser, `StatusCode`, `Response`, router with `:params`, `nest`, 404/405, thread pool, read timeout, size limits, panic → 500
- [x] `forge-json`: `Json` value, string escaping, `ToJson` trait for common types
- [x] `forge-di`: type-keyed container (`register` / `resolve`)
- [x] `examples/todo-api` with `GET /todos` and `GET /todos/:id`

---

## Phase 1 — Make the core usable

Goal: write a full CRUD todo app by hand, with no macros yet.

### `forge-http`
- [ ] ⭐ `HttpError` type with shorthand constructors (`not_found`, `bad_request`, …), built on the existing `Response::error`
- [ ] ⭐ `IntoResponse` trait, so handlers can return `Response`, `Json`, `String`, or `Result<T, HttpError>`
- [ ] Query-string parsing (`req.query_param("page")`), including `+` and `%20` decoding
- [ ] Percent-decoding of path segments (`/users/john%20doe`)
- [ ] Keep-alive: handle several requests per connection (currently `Connection: close`)
- [ ] Chunked request bodies (`Transfer-Encoding: chunked`); currently returns 501
- [ ] `HEAD` support (same as GET, but without the body) and automatic `OPTIONS`
- [ ] Graceful shutdown: stop accepting, finish in-flight requests (Ctrl+C handling needs care in `std`; a shutdown flag + timeout works)
- [ ] Middleware: a `fn(Request, Next) -> Response` chain for logging, CORS, request IDs
- [ ] Configurable limits (body size, header size, timeout, worker count)
- [ ] Global route prefix (`/api/v1`)
- [ ] Wildcard routes (`/files/*path`) and static file serving

### `forge-json`
- [x] ⭐ JSON **parser**: text → `Json` (needed for POST/PUT bodies). Handle escapes, `\uXXXX` incl. surrogate pairs, numbers, nesting depth limit
- [ ] `FromJson` trait (the opposite of `ToJson`) with helpful errors (`"title" must be a string`)
- [ ] `Json` accessors: `get("key")`, `as_str()`, `as_u64()`, indexing
- [ ] Pretty-printing
- [ ] `ToJson` for `HashMap`, `BTreeMap`, tuples
- [ ] Optional: integers stored without going through `f64` (no precision loss above 2^53)

### `forge-di`
- [ ] ⭐ Factories: `register_with(|c| UserService::new(c.resolve()))`, so providers can depend on other providers
- [ ] Trait objects (`Arc<dyn UserRepo>`) so implementations can be swapped (e.g. in tests)
- [ ] Missing-dependency errors at startup, not on first request
- [ ] Circular dependency detection
- [ ] Scopes: singleton (default), transient, per-request

### `forge-core`
- [ ] ⭐ Module imports: `UserModule` imports `DatabaseModule`
- [ ] Exported vs private providers per module
- [ ] Global modules
- [ ] Lifecycle hooks: `on_module_init`, `on_app_bootstrap`, `on_shutdown`
- [ ] Startup log listing every route (`GET /todos/:id`)

### `forge-config`
- [ ] `.env` file parser (written by hand: `KEY=value`, quotes, comments, `export`)
- [ ] Typed config structs, failing fast if a required value is missing
- [ ] Per-environment files (`.env.development`, `.env.production`)
- [ ] Register config as a provider so services can inject it

### `forge-validation`
- [ ] Built-in rules: `required`, `min_length`, `max_length`, `email`, `range`, `one_of`
- [ ] Hand-written pattern matching for email/URL-style checks (no regex crate; optionally write a small regex engine later)
- [ ] `Valid<T>` that parses a JSON body and validates it, returning 400 with field errors
- [ ] Nested struct and list validation

### New: `forge-time`
- [ ] Timestamps from `std::time::SystemTime`, formatted as ISO 8601 / RFC 3339 (you'll write the date maths for days → year/month/day)
- [ ] HTTP `Date` header

---

## Phase 2 — The decorators (`forge-macros`)

No `syn` or `quote`: you parse `proc_macro::TokenStream` yourself and generate code by building a `String` and calling `.parse::<TokenStream>()`.

- [ ] ⭐ A tiny internal token parser: find the struct/impl name, fields, methods and attributes
- [ ] ⭐ `#[derive(ToJson)]`: the simplest useful macro, a good first one
- [ ] `#[controller("/prefix")]` on an `impl` block: collects `#[get]`/`#[post]`/... methods into a `fn router()`
- [ ] `#[get]`, `#[post]`, `#[put]`, `#[patch]`, `#[delete]`
- [ ] `#[derive(FromJson)]`
- [ ] `#[injectable]`: a constructor that resolves each field from the container
- [ ] `#[module(imports = [..], controllers = [..], providers = [..], exports = [..])]`
- [ ] `#[derive(Validate)]` with `#[validate(email, length(min = 3))]`
- [ ] Clear errors via `compile_error!("...")` in the generated code
- [ ] Tests: expand macros in example crates and check that they compile and behave correctly

---

## Phase 3 — Request pipeline

Like Nest's lifecycle: middleware → guards → interceptors (before) → pipes → handler → interceptors (after) → exception filters.

- [ ] `Guard` trait: `fn can_activate(&self, req: &Request) -> bool`
- [ ] `#[use_guards(AuthGuard)]` on a controller or route
- [ ] `Interceptor` trait (timing, caching, response mapping)
- [ ] `Pipe` trait (`ParseIntPipe`, `ValidationPipe`)
- [ ] Exception filters
- [ ] Global, controller and route-level registration
- [ ] Per-request context (current user, request ID) passed through the pipeline

---

## Phase 4 — Batteries (new crates)

| Crate | What it does | Built from scratch? |
|---|---|---|
| `forge-log` | Levelled, structured logging with timestamps and request IDs | Yes |
| `forge-testing` | `TestApp` with overridable providers, and a client that calls the router directly (no network) | Yes |
| `forge-openapi` | Generate an OpenAPI JSON spec from controllers and DTOs, serve it at `/docs/openapi.json` | Yes (Swagger UI HTML can be served as a static file) |
| `forge-ws` | WebSocket gateways: handshake, frame parsing, `#[subscribe_message]` | Mostly. The handshake needs SHA-1 + base64; SHA-1 here isn't for security, so writing it yourself is fine. |
| `forge-cache` | In-memory cache with TTL, `#[cache(ttl = 60)]` interceptor | Yes |
| `forge-schedule` | Cron-expression parser + scheduler thread: `#[cron("0 * * * *")]` | Yes |
| `forge-events` | In-process event bus: `#[on_event("user.created")]` | Yes |
| `forge-health` | `/health` endpoint with pluggable checks | Yes |
| `forge-rate-limit` | Token-bucket rate limiting per IP or user | Yes |
| `forge-store` | Simple file-backed storage (JSON lines / append-only log), so the examples persist without a database | Yes |
| `forge-db` | Postgres client over raw TCP (wire protocol, prepared statements, connection pool) | Yes, but big. Auth needs MD5/SCRAM-SHA-256 → see exceptions. |
| `forge-auth` | Sessions/JWT, `AuthGuard`, `#[roles("admin")]` | Partly: hashing and signing are listed exceptions |
| `forge-runtime` | Your own async runtime (executor, waker, `epoll`/`kqueue` reactor) to replace the thread pool | Yes. Advanced: a project of its own, so do it last. |

---

## Phase 5 — Developer experience

- [ ] `forge` facade crate that re-exports everything (`use forge::prelude::*;`)
- [ ] `forge-cli` (arguments parsed by hand): `forge new my-app`, `forge generate module users`, `forge generate controller users`
- [ ] More examples: `auth-api`, `blog-api` (with `forge-store`), `chat` (with WebSockets)
- [ ] Docs: a guide per concept and a "coming from NestJS" page

---

## Phase 6 — Production ready

- [ ] Unit tests for every crate, integration tests for the examples
- [ ] Fuzz-style tests for the HTTP and JSON parsers (feed in random/broken input and make sure nothing panics)
- [ ] CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and a check that `cargo tree` shows no external crates
- [ ] Load testing with a tool like `wrk` or `oha` (external tools are fine; they aren't part of forge)
- [ ] Dockerfile + guide for running behind Caddy/nginx for HTTPS
- [ ] Docs on every public item (`#![warn(missing_docs)]`)
- [ ] Versioning, CHANGELOG, publish to crates.io

---

## Suggested order

1. `HttpError` + `IntoResponse`
2. JSON parser + `FromJson` → add `POST /todos`, `PUT /todos/:id`, `DELETE /todos/:id` (store todos behind a `Mutex`)
3. Query-string parsing + DI factories + module imports
4. Token parser in `forge-macros` → `#[derive(ToJson)]` → `#[derive(FromJson)]`
5. `#[controller]` / `#[get]` → `#[injectable]` / `#[module]`
6. Validation + `#[derive(Validate)]`
7. Keep-alive, middleware, logging, graceful shutdown
8. Guards → `forge-testing` → `forge-store`
9. Everything else as you need it, with `forge-runtime` last
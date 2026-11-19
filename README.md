# forge

A NestJS-style backend framework for Rust, built from scratch with **zero external crates**.

> ⚠️ Early development: APIs will change.

```rust
App::new().module(TodoModule).listen(3000)
```

## Crates

| Crate | What it does |
|---|---|
| `forge-core` | Modules and the `App` builder |
| `forge-http` | HTTP/1.1 server, router, request and response types |
| `forge-json` | JSON values and serialization |
| `forge-di` | Dependency injection container |
| `forge-macros` | `#[controller]`, `#[get]`, `#[injectable]`, `#[module]` |
| `forge-validation` | Request validation |
| `forge-config` | Configuration from the environment |

## Try it

```bash
cargo run -p todo-api
curl localhost:3000/todos
```

## Links

- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

MIT

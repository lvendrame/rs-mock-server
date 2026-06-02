# Library Example

This example shows how to embed `rs-mock-server` routes inside another Axum application.

Run it from this directory:

```bash
cargo run
```

Then try:

```bash
curl http://localhost:8080/
curl http://localhost:8080/host/health
curl http://localhost:8080/hello
curl http://localhost:8080/users
```

Open `http://localhost:8080/` in a browser for an index page with links to the
host route, mock routes, and the rs-mock-server home UI at `/mock-server`.

The host app owns the listener, port, `/` route, and fallback behavior.
`rs-mock-server` contributes routes from `example/mocks` plus its embedded home
UI under `/mock-server`.

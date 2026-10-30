# Design and implementation note

## API

| Request                            | Lookup                         |
|------------------------------------|--------------------------------|
| `GET /icons/file?path=src/main.rs` | Exact filename, then extension |
| `GET /icons/folder?path=src`       | Folder basename                |

Successful responses return the SVG directly with `Content-Type: image/svg+xml`. Clients should URL-encode the `path` query value.

- `400`: missing or empty `path`.
- `404`: unknown icon or route.
- `405`: unsupported method on a known route, with `Allow: GET`.

## Implementation decisions

### HTTP

Uses Hyper with explicit routing for two endpoints. I chose Tokio's `current_thread` runtime for this example; async tasks handle concurrent connections on one thread.

### Assets

`rust-embed` embeds SVGs in the build. The executable is self-contained, with a larger binary and a rebuild required when assets change.

### Paths

`form_urlencoded` decodes the query value before native `std::path::Path` parsing.

### Library cleanup

- removed panics for paths without base names or extensions
- replaced `lazy_static` with `std::sync::LazyLock` 
- the Unix-only byte-access API was removed.

## Validation and limitations

Tests cover SVG contents, encoded paths, filename precedence, and error responses.
For the current scope only Unix was considered. Currently path parsing follows the host platform.

## Production work

### CI/CD

- Formatting, Clippy and tests with pinned Rust and locked dependencies.
- Multi-stage Docker build, image smoke test and registry publication.
- Promote the same image digest between environments

### Deployment and scaling

- Add health check
- Multiple replicas behind a reverse proxy/load balancer handling TLS, connection limits and timeouts
- Readiness checks and graceful shutdown
- Autoscale on CPU or request concurrency

### Observability

- Structured logs and spans through `tracing`
- OpenTelemetry export for traces and metrics
- Request volume, latency, errors and resource usage
- Metric labels by route/status

### Caching

- maybe not needed
- the SVGs are already in memory and a response cache keyed by the full URL duplicates icons across different path queries and can grow larger than the entire icon set

### Performance

- Define throughput and latency targets
- Profile CPU, allocations and telemetry overhead before optimizing

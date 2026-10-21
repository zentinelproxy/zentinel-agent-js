<p align="center">
  <a href="https://zentinelproxy.io">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/zentinelproxy/design/main/marks/lockup/zentinel-lockup-dark.svg">
      <img alt="Zentinel" src="https://raw.githubusercontent.com/zentinelproxy/design/main/marks/lockup/zentinel-lockup-light.svg" height="52">
    </picture>
  </a>
</p>

<h3 align="center">JavaScript agent</h3>

<p align="center">Runs one JavaScript file (QuickJS) on request and response headers to allow, block, redirect or rewrite headers.</p>

<p align="center">
  <a href="https://github.com/zentinelproxy/zentinel-agent-js/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/zentinelproxy/zentinel-agent-js/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-9b4d8a?style=flat&labelColor=1f161e">
  <img alt="Rust: 1.94.1+" src="https://img.shields.io/badge/rust-1.94.1%2B-f68d31?style=flat&labelColor=1f161e&logo=rust&logoColor=white">
  <img alt="Agent protocol: v2" src="https://img.shields.io/badge/agent%20protocol-v2-f68d31?style=flat&labelColor=1f161e">
</p>

<p align="center">
  <a href="https://registry.zentinelproxy.io/js/">Registry</a> &middot;
  <a href="https://docs.zentinelproxy.io">Docs</a> &middot;
  <a href="https://github.com/zentinelproxy">All repositories</a>
</p>

For request logic that is too specific for a built-in filter and too small
for a compiled agent. The agent loads one script at startup and calls its
`on_request_headers(request)` and `on_response_headers(response)` functions;
whatever object they return becomes the decision. A function the script does
not define allows. A script error blocks with `500` unless the agent runs
with `--fail-open`; if the agent itself is unreachable, the filter's
`failure-mode` decides.

## Quick start

```bash
zentinel bundle install js

cat > handler.js <<'EOF'
function on_request_headers(request) {
    if (request.uri.startsWith("/admin")) {
        return { decision: "block", status: 403, body: "Forbidden" };
    }
    return { decision: "allow", add_request_headers: { "X-Processed-By": "js-agent" } };
}
EOF

zentinel-js-agent --script handler.js --grpc-address 127.0.0.1:50052
```

The agent speaks gRPC only (no Unix socket), so the proxy connects with `grpc`:

```kdl
system {
    worker-threads 0
}

listeners {
    listener "http" {
        address "127.0.0.1:8080"
        protocol "http"
    }
}

agents {
    agent "js" type="custom" {
        grpc "http://127.0.0.1:50052"
        events "request_headers" "response_headers"
        timeout-ms 100
    }
}

filters {
    filter "js" {
        type "agent"
        agent "js"
        failure-mode "closed"
    }
}

routes {
    route "default" {
        matches {
            path-prefix "/"
        }
        upstream "backend"
        filters "js"
    }
}

upstreams {
    upstream "backend" {
        target "127.0.0.1:3000"
    }
}
```

## Install

```bash
zentinel bundle install js
```

Or from the repository (the crate is not on crates.io):

```bash
cargo install --git https://github.com/zentinelproxy/zentinel-agent-js
```

Release binaries (`zentinel-js-agent-<version>-<platform>.tar.gz`) for
`linux-x86_64`, `linux-aarch64` and `darwin-aarch64` are on the
[releases page](https://github.com/zentinelproxy/zentinel-agent-js/releases).
From source: `cargo build --release`.

## Configuration

| Option | Default | Meaning |
|---|---|---|
| `--script` / `JS_SCRIPT` | required | JavaScript file to load at startup |
| `--grpc-address` / `GRPC_ADDRESS` | `0.0.0.0:50052` | gRPC listen address |
| `--fail-open` / `FAIL_OPEN` | `false` | On a script error, allow (tags `js-error`, `fail-open`) instead of blocking with `500` |
| `--verbose`, `-v` / `JS_VERBOSE` | `false` | Debug logging |

The default listen address is all interfaces; bind it to `127.0.0.1` unless
the proxy runs on another host. The script is read once; restart the agent to
change it. The proxy does not forward an agent block's `config { }` to the
agent, so use the flags above.

### Script API

The `request` argument has `method`, `uri` (path and query), `client_ip`,
`correlation_id` and `headers`. The `response` argument has `status`,
`correlation_id` and `headers`. Header names are **lowercase** (`user-agent`,
not `User-Agent`), repeated headers are joined with `", "`, and the request
headers include the pseudo-headers `:method` and `:path`.

The returned object:

| Field | Meaning |
|---|---|
| `decision` | `"allow"` (default), `"block"` / `"deny"`, or `"redirect"` |
| `status` | Status for block (default `403`) or redirect (default `302`) |
| `body` | Response body for block; target URL for redirect |
| `add_request_headers`, `remove_request_headers` | Object of headers to set / array of names to remove upstream |
| `add_response_headers`, `remove_response_headers` | The same for the response to the client |
| `tags` | Audit tags |

`console.log`, `console.warn` and `console.error` write to the agent's log.
There is no other host API (no network, no filesystem).

```javascript
function on_request_headers(request) {
    const ua = (request.headers["user-agent"] || "").toLowerCase();
    for (const bot of ["sqlmap", "nikto", "masscan"]) {
        if (ua.includes(bot)) {
            return { decision: "block", status: 403, tags: ["bot-blocked", bot] };
        }
    }
    if (!request.uri.startsWith("/public/") && !request.headers["authorization"]) {
        return { decision: "block", status: 401, body: "Authentication required" };
    }
    return { decision: "allow" };
}

function on_response_headers(response) {
    return {
        decision: "allow",
        add_response_headers: { "X-Content-Type-Options": "nosniff" }
    };
}
```

Unreleased on `main` (not in release 0.3.0): `on_request_body_chunk`,
`on_response_body_chunk` and `on_websocket_frame` hooks, a cached script
context, and a 64 MiB QuickJS memory limit.

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

## Related

- [Registry page](https://registry.zentinelproxy.io/js/)
- [Agent configuration](https://docs.zentinelproxy.io/configuration/agents/)
- [Agent protocol v2](https://docs.zentinelproxy.io/agents/v2/)
- [Lua agent](https://github.com/zentinelproxy/zentinel-agent-lua), the same model in Lua
- [Zentinel](https://github.com/zentinelproxy/zentinel)

## License

Apache-2.0, see [`LICENSE`](LICENSE).

---

<p align="center">
  <sub>Part of <a href="https://zentinelproxy.io">Zentinel</a>, the security-first reverse proxy. The shisa guards the gate.</sub>
</p>

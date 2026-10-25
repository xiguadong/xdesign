# WishFlow Studio — page development loop

This directory is **not** embedded into the `xdesign` binary. It exists so the
operator page under `../assets/` can be iterated without a Rust rebuild.

## Mock server

```sh
node web/dev/mock_server.mjs                  # http://127.0.0.1:7789
node web/dev/mock_server.mjs --port 7790
node web/dev/mock_server.mjs --latency 400    # exercise skeleton states
node web/dev/mock_server.mjs --empty          # real, reachable, empty project (writes enabled)
node web/dev/mock_server.mjs --offline        # API failures  -> labelled read-only sample mode
```

It serves `web/assets/` with the same root-relative URLs as the embedded build
(`/app.js`, `/styles.css`, `/js/*.js`, `/lifecycle.svg`) and implements the
SPEC-shaped `/api/health`, `/api/flows`, `/api/wishes`, `/api/wishes/<id>`,
`/api/wishes/<id>/work`, `/api/wishes/<id>/check`, `/api/wishes/<id>/material`
and `POST /api/wishes` endpoints from `../assets/js/sample-data.mjs`.

Fixtures are shared with the page's labelled sample mode, so the mock server and
the fallback dataset never drift. `POST /api/wishes` appends to the in-memory
list (and the sample wish index) so the create → detail flow can be exercised
end to end. Check/work responses are generated deterministically; stage
transitions are not simulated back into the fixture run.

Empty versus sample semantics (web/SPEC.md §7):

- `--empty` is a **real** project with no wishes: the page renders its empty
  states, the New wish wizard is enabled, and writes work (POST appends in
  memory).
- `--offline` makes the API unreachable so the page falls back to the clearly
  labelled, read-only sample dataset.
- `?demo=1` in the page URL requests the same labelled sample dataset against a
  reachable server (e.g. `http://127.0.0.1:7789/?demo=1#/overview`).

## The real server

The integration environment is the embedded server:

```sh
cargo build --offline
./target/debug/xdesign web --port 7788   # serves the compiled-in assets
```

Rebuild after asset changes: `web/assets/**` is embedded at compile time by
`web/server/build.rs`.

## Screenshot harness

Interactive verification used `cache/frontend/verify.mjs` (mock server) and
`cache/frontend/verify-real.mjs` (embedded server) through headless Chrome over
CDP. Those scripts live under `cache/` because they are temporary tooling, not
repository content.

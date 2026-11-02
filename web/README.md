# xdesign web

The operator web page for the xdesign local store and stage CLI, shipped inside
the `xdesign` executable.

## Run

```sh
cargo build --offline
./target/debug/xdesign web
# xdesign web listening on http://127.0.0.1:7788
```

Options: `--host`, `--port` (`0` = ephemeral), `--open`, `--project-root`,
`--store-root`. Run it from the project root you want to inspect; lifecycle
actions then behave exactly like `xdesign init/next/check` in that checkout.

There is no Node.js build step. Files under `web/assets/` are embedded at
compile time by `web/server/build.rs`; rebuild the binary to pick up a page
change. `web/SPEC.md` is the server/API/frontend contract.

## Develop the page

The server is the integration environment; a page-only loop is:

```sh
cargo build --offline
./target/debug/xdesign web --port 7788
# edit web/assets/* and rebuild to reload
```

When the API is unreachable (for example the assets are opened from a plain
static file server) the page falls back to clearly-labelled read-only sample
data so the visual design can still be reviewed; sample mode is also available
on request with `?demo=1`. A reachable project with no wishes is a normal empty
state: the page shows the real empty state and creating the first wish is
enabled.

## Layout

- `assets/` — the page (HTML, CSS, JavaScript, SVG); all local, no CDN.
- `server/` — embedded Rust module: HTTP layer and API assembly.
- `SPEC.md` — contract; `README.md` — this file.

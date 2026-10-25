# xdesign

xdesign is a SysML v2 engineering tool: a project model explains the system, and each
wish's operational trace explains why its requirements, design, implementation, and
verification followed. New runs use the local store; legacy and bootstrap runs retain
independent SysML traces. The gated four-stage SimpleFlow and two-stage wish-verify flow
are exposed through `init`, `next`, and `check`; the generic controller and full V-model
remain open.

Start with [docs/README.md](docs/README.md) for project composition, usage, and the
package, timeline, and swimlane diagrams together on one page. Use the
[documentation map](docs/index.md) for detailed specifications. Current capability status
and evidence are maintained in [docs/records/status.md](docs/records/status.md).

## xdesign CLI

### Environment installation

Run the commands below from this repository's root in Bash or Zsh. The bundled
SysML compiler targets **Apple Silicon macOS** (`aarch64-apple-darwin`), which is
the verified environment for the complete SimpleFlow CLI. Other platforms need
a compatible compiler and toolchain integration; see the
[toolchain record](tools/sysml_web/toolchain.json).

| Dependency | Setup |
| --- | --- |
| C compiler and linker | On macOS, install Xcode Command Line Tools with `xcode-select --install` if missing. |
| Python 3 | Provide `python3` on `PATH` (3.9 or newer). Install a supported release from [Python's macOS installers](https://www.python.org/downloads/macos/) if needed. CLI checker scripts use the standard library. |
| Rust and Cargo | Use stable Rust; the current CLI was verified with Rust/Cargo 1.99.0. Installation is shown below. |
| SysML compiler | Included at `tools/sysml_web/bin/sysml`; no separate download is needed on Apple Silicon macOS. |

Keep dependency caches, build output, and temporary files inside the project:

```bash
mkdir -p cache/tmp cache/cargo-home
export TMPDIR="$PWD/cache/tmp"
export CARGO_HOME="$PWD/cache/cargo-home"
export CARGO_TARGET_DIR="$PWD/cache/target"
```

If Rust is not installed, install it locally under `cache/` using
[rustup](https://rust-lang.github.io/rustup/installation/index.html):

```bash
export RUSTUP_HOME="$PWD/cache/rustup"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o cache/tmp/rustup-init.sh
sh cache/tmp/rustup-init.sh -y --profile minimal --default-toolchain stable --no-modify-path
export PATH="$CARGO_HOME/bin:$PATH"
```

In the prepared workspace, an existing toolchain can instead be used directly:

```bash
export PATH="$PWD/cache/toolchain/stable-aarch64-apple-darwin/bin:$PATH"
```

Repeat the environment exports in each new terminal. When using the local rustup
installation, also restore `RUSTUP_HOME` and add `$CARGO_HOME/bin` to `PATH`.
The [Rust installation guide](https://doc.rust-lang.org/book/ch01-01-installation.html)
describes the system compiler prerequisites.

### Build and enable the command

```bash
rustc --version
cargo --version
python3 --version
tools/sysml_web/bin/sysml --version
cargo build --locked
cache/target/debug/xdesign --help
```

The executable is `cache/target/debug/xdesign`. The first build downloads Cargo
dependencies; when they are cached, use `cargo build --locked --offline`.
To make `xdesign` available by name, create a user command once:

```bash
mkdir -p "$HOME/.local/bin"
ln -s "$PWD/cache/target/debug/xdesign" "$HOME/.local/bin/xdesign"
export PATH="$HOME/.local/bin:$PATH"
xdesign --help
```

`ln -s` refuses to overwrite an existing command. The link survives new terminals
and rebuilds while the executable remains at that location. Ensure
`$HOME/.local/bin` is on your shell's `PATH` in new terminals; the export above
applies only to the current terminal. You can also invoke
`cache/target/debug/xdesign` directly. Continue running CLI commands from the
repository root: they use this repository's models and checker scripts.

### Initialize a wish and perform its stages

Prepare a SysML wish source at `mbse/wishes/WISH_ID.sysml`, or pass another source with
`--wish FILE`. New runs import it into `.xdesign/` and use the regenerable working copy at
`cache/wishes/WISH_ID.sysml`; they do not require a new Git wish trace. Legacy/bootstrap
runs keep their existing SysML authority. IDs use ASCII letters, digits, hyphens, and
underscores.

Initialize once, then repeat the same two commands at each stage:

```bash
xdesign init WISH_ID --flow simple-flow [--wish FILE]
xdesign next WISH_ID
# Perform the work described by the returned stage.
xdesign check WISH_ID
```

- `next` returns JSON with the current goal, instructions, inputs, expected outputs,
  and exit checkpoints. It leaves the stored run unchanged.
- `check` collects outputs, records their revisions, evaluates the current gates, commits
  the stage revision, and returns correction work, the next stage, or completion.
- On a blocked result, address the returned correction reasons and check again.
  Stop when `completed` is `true` and `work` is `null`.

| Flow selected at initialization | Stages | Judgment required |
| --- | --- | --- |
| `--flow simple-flow` | requirements → design → implement → verify | Fixed checks at each stage; verify also needs an explicit agent or human judgment. |
| `--flow wish-verify` | wish → verify | Both stages need an explicit agent or human judgment. |

### Submit evidence and resume work

For a semantic gate, retain the stage's evidence and prepare a result JSON using
the [result-file contract and example](docs/design/stage-cli.md#d85). It identifies
the wish, run, stage, visit, revision, and state token; supplies artifact paths and
SHA-256 digests; and records the judge's identity, outcome, rationale, and complete
artifact snapshot digest.

```bash
xdesign next WISH_ID
# Review the current work and evidence; prepare cache/stage-result.json.
xdesign check WISH_ID --result cache/stage-result.json --operation-id OP_ID
```

Use the latest work response when preparing the result. New evidence files must
be included in the complete snapshot covered by the judgment. Missing judgment
leaves a semantic gate unevaluated; stale results are refused. The contract also
explains how to record new evidence before supplying its judgment. Reuse the operation
ID for exact retry after an uncertain response and inspect it with `xdesign store
outcome OP_ID`; do not retry an uncertain save under a fresh identity.

Durable revisions, evidence materials, and receipts live under `.xdesign/` by default;
`XDESIGN_STORE_ROOT` overrides that location. Runtime files under `cache/` are regenerable.
Use `xdesign store list`, `get`, `history`, `checkout`, and `doctor` to inspect them.
Completed calls return the recorded completion without reopening the flow or recording
final human acceptance.

| Exit code | Meaning |
| --- | --- |
| `0` | Work returned, advancement succeeded, or run completed |
| `1` | Check ran, but the stage remains blocked |
| `2` | Invalid input, stale result, unavailable writer lock, or another command error |

The current CLI supports these two repository-bound flat flows. See the
[CLI contract](docs/design/stage-cli.md#d85) for storage, result validation, and
the remaining execution limits.

## Documentation

Build or preview the documentation:

```bash
python3 -B tools/docs.py build
python3 -B tools/docs.py serve
```

The wrapper keeps its environment and output under `cache/docs-site/`. The documentation
set is organized as requirements, architecture, design (including one document per API),
interfaces, verification design, and records. Authoring rules are in
[docs/AGENTS.md](docs/AGENTS.md); repository-wide agent rules are in [AGENTS.md](AGENTS.md).

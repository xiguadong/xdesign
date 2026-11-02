# Checker resources and project execution

<a id="d111"></a>

## 1. Ownership and release boundary

This design realizes `CheckerResourceDistribution`, `CheckerExecutorBundle` and
`ProjectCheckerExecution`. Its acceptance contract is [ISO-01–09](../verification/design.md#d112).
Implementation follows design review; executed results belong to store record
`record-execution-checker-script-isolation`, never to this page.

| Bytes | Owner and location | Execution and upgrade |
| --- | --- | --- |
| Built-in scripts, common helper, pinned SysML parser and toolchain identity | Embedded in the released executable; authored in `tools/checks/` and `tools/sysml_web/` | `init` materializes and freezes the binary's resources; a release upgrade affects new runs |
| Editable project script | Project Git, normally `checkpoints/scripts/<id>.py` | Explicit command/library selection; changes affect new runs |
| Declared helpers, rule data and dependency lock files | Project Git; list in the script executor's `files` | Entry plus declared files form one retained bundle |
| Run's exact executor bundle | Project store, with working copies in `cache/xdesign/<wish>/executors/` | Frozen paths and digests in the plan; source changes never rebind it |
| Python environment | Consumer project, normally `cache/checkers/<environment>/` | Optional project-relative `python`; interpreter identity recorded on each invocation |
| Reports, logs and temporary/cache output | Unique evaluation directory under the project's run reports | Reports, logs and execution identities retained with the stage revision; temporary data stays disposable |

The existing built-in `command_uri` values (`tools/checks/simple_*.py`) remain logical
source identifiers for compatibility. Default selection resolves them from embedded
bytes, without testing for a customer `tools/` file. An explicit project command uses
project bytes even when its name matches a built-in source identifier. Built-in library
IDs identify embedded resources; user definitions never become built-ins by path spelling.
There is no fallback into another workspace, executable installation directory or global
writable script cache. Legacy plans without a bundle retain their previous execution
contract; new store-backed registrations always retain script resources.

Built-in requirements and design checks still inspect xdesign's declared SysML packages
and documentation conventions. Packaging them makes them runnable after installation;
it does not make those rules applicable to every engineering project. A different
project binds its own checkpoints and explicit input paths. Default implementation source
collection remains the existing Rust selector. General flow/input mapping, shared package
imports and a unified cross-project history view remain separate work in [D108](checkpoint-library.md#d108).

## 2. Authoring a project script and its dependencies

A script library executor accepts the following optional additions to its existing
schema. Command bindings accept the same `files` and `python` fields beside
`command_uri`. Default and library-selection bindings cannot carry them;
for library mode these fields belong to the selected definition.

```json
{"mode":"script","command_uri":"checkpoints/scripts/budget/check.py",
 "files":["checkpoints/scripts/budget/helper.py",
          "checkpoints/scripts/budget/rules.json",
          "checkpoints/scripts/budget/requirements.lock"],
 "python":"cache/checkers/budget/bin/python3"}
```

`files` lists additional regular project files; the entry is included automatically.
Paths are normalized and project-relative; duplicate files, missing files and symlink
components are refused at registration. Preserve their relative tree inside the bundle,
so imports beside `__file__` and bundled data keep their relationship. Declare every
local executable/data dependency; imports are not inferred. An undeclared helper is
unavailable from the retained script directory. Shared checks can be copied or vendored
explicitly into each project with their declared files; automatic package distribution
and provenance are not delivered here.

`python` names a project-relative executable under `cache/`. Prepare that environment
from the retained dependency lock before use, for example with `python3 -m venv` and the
project's package installation procedure. xdesign does not install dependencies or mutate
a global environment. The environment itself and third-party packages are not archived;
recreation and compatibility remain the project's responsibility. Omitting `python` uses
`python3` from the invocation's PATH, with its actual executable, digest and version
recorded per attempt. A missing interpreter blocks execution explicitly.

## 3. Registration, execution and recovery

Registration resolves each process executor, copies exact entry/helper/data bytes into a
run-confined bundle, records their SHA-256 digests in `checker.bundle`, and includes every
bundle file in the registration's store manifest. Built-in bundles also contain the
selected parser and its toolchain record. Freeze occurs before checker input discovery;
discovery uses the same bundle and the existing checker deadline. Registration commits no
wish revision if resource resolution, discovery or freezing fails.

Both input discovery and evaluation use an explicit project cwd and
`XDESIGN_PROJECT_ROOT`. `XDESIGN_CHECKER_BUNDLE_ROOT` identifies the copied tree;
`XDESIGN_ATTEMPT_DIRECTORY`, `TMPDIR`, `TMP`, `TEMP` and `XDG_CACHE_HOME` identify private
attempt paths. Clear `PYTHONPATH` and `PYTHONHOME`, disable user-site packages and Python
bytecode output. Built-in code uses the project root for all artifact reads and the
retained parser for SysML parsing. Design-check command inspection resolves source
identifiers to frozen entry files, rather than requiring customer `tools/` files.

Before running, verify every bundle digest. The adapter adds all executor files to
`input_sha256`, records interpreter identity and the selected project/attempt, retains
stdout/stderr and applies the existing raw-report validation. The script's declared rules
and assessor authority are unchanged. A missing dependency, changed retained byte,
malformed report, timeout or unavailable interpreter cannot authorize a gate.

Store load recovers missing executor copies from verified material bytes. Existing
copies with different bytes are integrity refusals. The frozen plan, rather than the
current library or source files, controls the bundle. A new run is required to choose a
changed script. Earlier plans, assessments and frozen records are never rewritten.

This is execution-context isolation, not an operating-system sandbox. A trusted custom
script can still access files and run programs permitted to the current user. Full Python
environment reproducibility, filesystem access control and guaranteed cleanup of arbitrary
script side effects are outside the acceptance boundary.

## 4. Source allocation and review

| Source | Responsibility |
| --- | --- |
| `src/core/plan.rs` | Optional serializable `RsCheckerBundle`, structural validation and unchanged legacy omission |
| `src/simple_flow/resources.rs` | Embedded resource catalogue, safe project-file reads, bundle freezing and Python command context |
| `src/simple_flow/blueprint.rs`, `src/checkpoint/library.rs` | Explicit ownership resolution and strict custom file/interpreter declarations |
| `src/simple_flow/check.rs` | Retained executor validation, interpreter/execution evidence and raw-report integration |
| `src/stage_cli/work.rs`, `src/stage_cli/persistent.rs` | Bounded input discovery, registration retention and cache recovery |
| `tools/checks/common.py`, `tools/checks/simple_design.py` | Explicit project/parser context and retained command lookup |
| `tests/checker_isolation.rs` | Process-level acceptance using a copied executable and independent consumer projects |

The existing flow and crate views remain applicable. The ownership table replaces an
additional diagram: it distinguishes authored, released, retained and temporary bytes
without introducing a new flow or authority. The optional bundle API is described in
[U110](api/core/rs-checker-bundle.md#u110); the process adapter contract remains
[U58](api/simple-flow/rs-process-check.md#u58).

# Worktree store verification procedures

> Status: planned, unexecuted procedures. These refine and do not replace or weaken
> [WTSTORE-01–13](design.md#d115). Design: [D116](../design/storage.md#d116),
> [D117](../design/worktree-store-migration.md#d117); interface: [I21](../interfaces/worktree-store.md#i21).

<a id="d118"></a>

## 1. Fixtures, seams and retained evidence

Create every fixture and simulated home beneath this checkout's `cache/`, with unique test
IDs and private modes. Use an injected effective-account lookup, never change the real home
or copy the user's operational store. Rust unit seams are private/test-only; the real-binary
fixture runner uses a test build whose account lookup is compiled for the fixture root.
Production builds must ignore test environment variables. Subprocess test binaries, old
binaries, logs, sockets and SQLite temporary files also stay under cache. Pin and hash the
pre-migration executable/source revision; create genuine old-version stores using that
binary rather than hand-editing an ordinary new store into a supposed compatibility fixture.

Each result retains exact executable/source hashes, argv, selected roots, environment
(overrides and test seams only), exit code, stdout/stderr, platform/filesystem capability
report, baseline/result inventories and explicit assertions. Independent SQL enumeration
uses a read-only connection and the retained schema; it must not merely call the migration's
own equality helper. Compare exact request and JSON bytes, all heads/revisions/materials,
terminal receipts and blob hashes, and validate the source as well as destination. Record
negative fixture byte/mode/identity changes to prove no adoption or collateral mutation.
Fresh disposable clones, not restoration over reused fixture roots, separate crash trials.

Deterministic test seams belong in the planned modules and are disabled outside tests:

| Seam | Controlled observation, not a replacement for the real path |
| --- | --- |
| Account/Git/identity providers | Missing home, bad registration, reused path/device/inode, changed birth time/nonce, no birth evidence, key collision, nonce collision and unsafe ACL/owner; repeat positive tests using real Git/filesystem. |
| Lease barriers | Pause after unlocked parent probe, complete set discovery, first ordered acquisition, source fence, baseline, last blob, DB commit, verified target, source revalidation and decision publication; release from the test controller, no sleeps as race proof. |
| Durable I/O failpoints | Before/after each write, file sync, no-replace publish, pointer replacement, directory sync and extended-attribute flush; return I/O failure or terminate process at named boundaries. |
| Import verification seam | Omit/change an old revision, conflict receipt, raw request whitespace, metadata number, material or trigger; verification must refuse. |
| Lifecycle child seam | Spy actual executable argv/cwd/root, then use the real child binary for an end-to-end run. A mock alone cannot pass Web routing. |
| Capability probes | Cross-process directory locks on macOS/Linux, xattr durability across process restart, birth evidence, same-parent atomic publication, nonblocking lock failure; unsupported mounts fail rather than silently downgrade. |

HMAC/encoding tests include published algorithm known-answer fixtures retained locally,
length-prefix ambiguity, JSON member reordering with unchanged canonical bytes, binary field-order refusal, unknown fields, exact-byte request
mismatch and negative MACs. Capability probes do not certify power-loss survival or hostile
same-owner security. Simulated identity values can test refusal logic, not prove a real
filesystem never reuses an observation.

## 2. Case procedures and expected results

### 2.1 WTSTORE-01 — Context selection

Make a non-Git directory, primary repository, separate-git-dir primary, linked checkout,
nested selected projects and a submodule primary checkout. With no override, assert local
`<selected>/.xdesign` for non-Git/primary and the dedicated versioned home root for each
linked project; no worktree-local store is created for a new linked default. Read-only
inspection before enrollment creates no files/attributes; initialization enrolls once.
Repeat in a fresh process and through canonical parent aliases and prove identical binding
and store. For nested selection assert it is not collapsed to the Git toplevel.

Inject misleading `.git` files, bare repo, stale backlink, wrong commondir, prunable/duplicate
registration, Git command failure, external `GIT_DIR`/ceiling overrides and unavailable/unsafe
home. Assert the exact I21 diagnostic, no local fallback, no created history and unchanged
foreign bytes. HOME alone cannot redirect default; explicit eligible root succeeds with a
missing account-home lookup and does not call Git. Unsupported filesystem capabilities
produce explicit `identity_unprovable`/`coordination_unavailable`, never a guessed identity.

### 2.2 WTSTORE-02 — Identity and isolation

Use same-basename distinct repositories, two linked checkouts per repository, two selected
projects per checkout and the same wish ID in each. Each has distinct binding/root/history;
restart and symlink-parent aliases of one project agree. Inject equal keys and equal random
nonce values for distinct full tuples: publication refuses without sharing. Validate
namespace/file modes, MAC and full binding, not just key lookup.

Move a worktree with Git, move the selected project, remove/recreate an administrative
directory, recreate a repository/worktree at the same path, copy Git/nonce/binding metadata,
edit signed fields, and force reused device/inode with changed birth/nonce through the seam.
Every unproved incarnation refuses adoption. Remove the birth evidence entirely and require
refusal. Inspect the old **unchanged global root** explicitly and verify its history remains
available; raw physically copied stores still fail identity checks. Test new-incarnation
enrollment with explicit old-binding acknowledgement: it creates a new empty binding only
with fresh valid evidence, retains the old record, and never rewrites a copied nonce. Repeat
partial enrollment at each durable boundary; exact ID resumes, changed request refuses.
State that full same-owner/key/filesystem replay is outside the authentication boundary.

### 2.3 WTSTORE-03 — Global container

Seed unrelated files/directories, a separate valid store, prior namespace versions and a
legacy-format directory under the controlled home container. Capture bytes, modes, identities
and whether each existing store opens. Enrollment creates only its namespace children and
leaves all siblings unchanged. Repeat with `.xdesign` itself a valid store, partial recognizable
store, namespace occupied by foreign bytes, symlink, foreign owner or writable ACL: refuse
before creating a child/chmod/adoption. Probe reserved access-sibling collisions separately;
ordinary root open must not modify them or mistake them for a clean absence.

### 2.4 WTSTORE-04 — Overrides and actual Web subprocesses

Cross all four surfaces with absent/empty/relative/absolute option and environment, linked
and non-linked contexts. Assert store CLI option > environment > default; lifecycle environment
> default; Web option > environment > contextual default; direct API only supplied root.
Assert store save's `--project-root` affects material input only. Spaces/non-ASCII paths are
passed as native arguments; tilde is literal; non-UTF-8 contextual identity refuses.

Launch real Web with project A, explicit store B, environment store C and default D. Through
HTTP create, detail, work and check invoke actual children; assert their cwd is A and root B,
health also names B, and C/D contain no new bytes. Repeat next/check helper paths and a
concurrent root/metadata change after parent preflight: child must fail, not fall back.
Use a read-only diagnostic Web state for migration conflict; malformed/unsupported store is
not an empty successful list. Test HTTP error/status mapping and absent-store health wording.

### 2.5 WTSTORE-05 — Complete migration

Seed generic non-wish items, multiple histories, unique/shared and formerly referenced blobs,
small/large and zero-byte materials, exact JSON number/text forms, request whitespace and
committed/conflict receipts and valid noncanonical `format.json` whitespace/member order.
Require the exact marker bytes/digest at destination, not a compact reserialization. Seed both flows' active, blocked and completed wishes, checker
bundles and artifacts. Use the old executable; retain its digest. Enroll and apply a migration
request to the exact absent target after explicit quiescence. Independently compare every
row/byte named in D117 §1.1 at every revision. Assert same logical store ID, distinct physical
root/DB mapping, no extra revisions/receipts and original source history intact.

Negative trials mutate an old revision/receipt or blob, drop a trigger, leave pending work,
use a raw copied target, or pre-initialize an unrelated target with matching head/ID. Assert
no decision and no public target authority. Verify constructor insert ordering with the real
immutable/head/contiguous triggers enabled; source physical fields and trigger definitions
must never be rewritten. Reading old reserved requests must not apply new-save-only limits. For a same-filesystem
rename of a never-bound old root, record the existing physical-check observation rather
than asserting that inode checks detect path history. Bound/migrated roots must refuse the
move via the new witness, and an unproven raw target never becomes default authority.

### 2.6 WTSTORE-06 — Lifecycle and cache loss

Record exact pre-migration work/tokens/IDs/visits/evidence for SimpleFlow and WishVerifyFlow
active, blocked and completed fixtures. Migrate and compare unchanged-head response facts;
only root-selection diagnostics may differ. Delete only proven persistent regenerable cache,
then replay the original terminal operation with its retained exact request. Assert the
original receipt/result and one effect even with original sources gone. Restore work, execute
a valid active check, then stale-token, stale-head and artifact-drift checks. Expected guards
remain unchanged. Completed next/check add no revision or assessment. Changed workspace
artifact/wish bytes survive cache restoration; the declared drift is surfaced.

### 2.7 WTSTORE-07 — Ambiguous and changed sources

Test local-only store before migration: default commands diagnose migration required and do
not create an empty global root. Test independently valid local/global roots, including same
IDs/heads with differing early history: conflict, no merge. After a decision, advance target
while source stays unchanged: usable target and source inspection. Remove only source root:
destination remains authoritative, and its path tombstone blocks source init. Recreate source
with different identity, including same ID and copied bytes: default and explicitly selected
destination mutation fail `source_changed`, source writes remain fenced. With the new client
stopped, deliberately use the old binary to append to retained source; restart and assert
full-baseline conflict before any destination mutation. This demonstrates detection, not
old-client exclusion. Add two commit barriers: an old-client source commit after initial
validation but before the final complete source check must produce `source_changed` and
no destination commit; a prohibited commit after that final observation may race the
destination commit, but every later mutation must detect it before a further commit.
Never label the second barrier old-client fencing. Repeat with a resumed pending operation.
Inspect both histories read-only and retain both inventories.

### 2.8 WTSTORE-08 — Genuine legacy and same-ID inspection

Create actual legacy registrations from the pre-store fixture code or recorded legacy
schema with matching valid Git wish source, frozen plan/work and run; retain construction
commands and all files. Include active/completed runs beside persistent ones. Verify legacy
next/check retains its original backend behavior, completed runs stay complete, and Git
traces are unchanged by migration. Persistent cache with proven store provenance is
regenerable; unmatched cache is ambiguous, not deleted or silently treated as legacy.

Repeat genuine legacy-only read/next/check with no binding and unavailable account lookup,
unsafe unrelated home, and no local candidate; no namespace/history/nonce is created and
legacy active/completed behavior remains unchanged. Repeat with an existing local candidate,
a global candidate located through the permanent route, and persistent cache provenance:
strictly inspect them before any continuation. Same-ID candidates produce `backend_conflict`;
unreadable known candidates produce `backend_ambiguous`. A legacy inspection selector remains
read-only. Verify legacy-only Web health has null persistent root and the real child clears
the inherited root variable, reclassifies, and refuses a newly appeared conflicting candidate.

Create a genuine legacy registration and a persistent item with the same ID and different
history. Assert CLI `inspect --backend legacy` and `--backend store` return each exact run;
HTTP detail/material with both backend selectors returns each corresponding history/bytes.
Listing exposes both with a conflict indicator. Unqualified detail/work and all create/check
mutations refuse before changing either history. Attempt backend selector on a mutation:
reject it. Repeat classification under a barrier while a supported competing client attempts
registration, proving the project backend lease covers classification through commit.

### 2.9 WTSTORE-09 — Preconditions and refusal

Table-drive every source defect: missing, foreign, newer, changed database identity,
missing/altered blob, unknown schema object, bad modes/owner/ACL/symlink, unreadable binding,
occupied destination, pending operation, orphan/staging residue, live/stale/malformed item
lock and another migration ID. Assert exact reason/exit, no decision, no source logical
mutation and no unrelated deletion. Query/recover pending request with the original root's
supported operation procedure; if preparation exists, abort it first. After recovery and
operator lock resolution, start a new migration with a completely new baseline; assert old
prepared digests/target DB are not reused as proof. Old request exact retries remain valid.

### 2.10 WTSTORE-10 — All-access concurrency matrix

Run a row for every operation below across fresh handles and handles opened before migration,
against both source and destination, at `route_only`, `reserved`, `fenced`, `preparing`, `prepared`,
`committed` and `aborted` barriers. Repeat through absolute, relative, canonical,
case/normalization alias (where the host supports it), parent-symlink and final-symlink paths.
A pre-migration destination handle cannot exist because its root is absent; record the
failed open as the construction oracle rather than fabricate an ordinary held handle.
After commit create real held destination handles and repeat subsequent change/alias trials.
Final-root symlinks must be refused, which counts as fence coverage rather than successful
alias support. Unsupported bind/mount aliases must fail the capability check.

| Operation class | Calls to exercise |
| --- | --- |
| Construction | `RsStore::init`, fresh `RsStore::open`, CLI init/info, absent-source init |
| Mutation/retry | Typed `save`, exact `save_bytes`, new/terminal/pending retry, lifecycle init/check, actual Web writes |
| Query | `outcome`, `pending_item_operations`, `list` with/without filters, `get` head/old revision, `history` |
| Bytes/output | `read_material`, checkout, CLI read/request with absent output destinations |
| Inspection | Handle doctor, root doctor on corrupt/partial root, CLI info/location/status |
| Adapter | Lifecycle next/cache restoration, completed next/check, Web list/detail/material/work and health |

Pause writer inside final commit while migration attempts its lease: writer either completes
before the baseline or migration returns busy; no acknowledged update disappears. Pause
migration after baseline and attempt every call: only diagnostic access after release or
retryable refusal is allowed; no reader sees a partial target. After decision, held source
handles cannot save/init; allowed source queries return the retained consistent history.
At abort, source writes resume only after durable baseline invalidation. Race two migrations:
at most one decision; losers return busy/mismatch/conflict, never independent authorities.

Delete/recreate source at the original path while old handles exist, then open fresh handles
through every alias; repeat immediately before decision and after decision with source
absent then recreated. Old handles fail physical identity, fresh calls see tombstones,
pre-decision disappearance prevents decision, post-decision absence does not erase provenance.
Repeat destination replacement and partial target reopen. Inspect lock-order instrumentation
for the single physical-identity total order across namespace, project/backend and parents, no recursive public API lock, and
Web parent lease release before child wait. Demonstrate older clients ignore the new lease;
record the operator quiescence procedure and explicitly do not count it as automatic fencing.

Use a deterministic two-process inversion trial: process A pauses after its unlocked parent
probe, process B takes the first globally ordered descriptor and pauses, then let A attempt ordered acquisition.
A must hold no descriptor when it fails on B's first lock. Let B acquire the remaining
set and finish, then A retries with a revalidated set. Reverse roles and source/
destination entry, and repeat for a pure-legacy command versus enrollment, held API versus
migration, status/abort, source-baseline check and Web preflight. Retain descriptor roles,
physical identities, attempt numbers and acquire/release trace; assert every prefix is
sorted, aliases deduplicate, no lock survives a failed attempt, and changing a generation
restarts while changing a pinned identity refuses. Exhaust 8 attempts deterministically:
return `migration_busy` with no effect. Include crossed explicit roots where a directory
is the backend for A and a parent for B; role-dependent order must never reappear. A subsequent uncontended call must succeed; no
continuous-contention fairness bound is claimed.

### 2.11 WTSTORE-11 — Every pre-decision interruption

Interrupt separately before/after each marker/route temporary write, file sync, no-replace
publication and directory/ancestor sync, at each of the two parents. Until **both** routes
are confirmed durable, no reservation/request publication is legal. With only a route and
no accepted reservation, status from either root is `not_found`, not a reserved outcome;
exact apply can complete routing, and status creates nothing. Foreign/corrupt routes refuse.
Next interrupt empty journal-slot creation/sync, separate draft-directory creation/sync,
exclusive draft creation, partial/complete write, file sync/close, atomic no-replace final
publication, both directory/ancestor syncs, and every derivative request write/sync before
the **first** root pointer. Repeat from both root routes with source present/absent and
parents intact, deleting original request/cache. Destination is normally absent at these
boundaries, so the absent-source trial also tests both roots absent; after destination
allocation below, repeat with that owned root present and separately removed.
Assert the following operations, not only the observed phase:

| First-publication observation | Required assertions from both roots |
| --- | --- |
| No complete final ever published; empty slot or partial/complete draft only | Status/request/abort/discard each return `not_found`, leave source/destination unchanged and create no terminal record; no exact bytes/nonce are fabricated. With a saved full request, same-ID apply starts a fresh attempt; with lost original request, an explicit replacement request at that unused ID or a new ID can start. Both refuse absent source, and neither consumes a draft. |
| Partial draft with live exclusive-create proof | Cancel/unlink only the exact pinned unpublished file and sync its directory; no migration rollback or terminal abort is invented. Inject a replacement inode before cleanup and assert refusal/preservation. |
| Partial or complete draft after process death/proof loss | Quarantine in place byte-for-byte, even if foreign or signed; never promote/delete/reuse it. A fresh exclusive name permits eligible same-ID/new-ID apply without adopting those bytes. Inject foreign content in an ID slot instead: all recovery/apply/new-ID paths refuse `coordination_corrupt` and preserve it. |
| Complete final, publication/durability uncertain | Resolve final and directory durability first: confirmed present becomes `reserved` with exact bytes/nonce; confirmed absent with no effects is the no-accepted-request row; failed confirmation stays exit 7 for status/request/apply/abort/discard/new-ID. Never infer absence from a failed sync/read. |
| Durably accepted final; derivative request partial/missing | Status `reserved`; request recovers exact bytes despite lost cache; exact apply keeps the nonce, changed bytes same ID refuses, new ID is busy. Absent source refuses apply only, with request/abort still available. Discard before abort is busy; abort is durable/idempotent; discard afterward touches only proven destination residue, then a new ID is eligible. Original ID still replays its abort and retains bytes/nonce. |

Capture file identity/byte inventories before/after every assertion. Assert no supported
partial write ever creates `migrations/<ID>/reservation.pending` or a partial final record;
invalid final injection is a corruption case, not a normal crash outcome. Draft/empty-slot
creation does not depend on another intent record. First-publication uncertainty creates no
root/fence/target effect until the final reservation is durably confirmed.

Then terminate after destination pointer publish/sync, source pointer publish/sync, each
attribute install/sync, baseline write, each blob publish, database import/commit/close,
verification publication, source revalidation and immediately before decision publication.
Restart with lost cache/original request. Status plus recovered exact request must resume
from a fresh baseline or return the precise I21 refusal. Source histories remain identical;
partial targets never open normally.
For each eligible state repeat with abort; source is released only after durable invalidation,
old-ID retry returns aborted, and discard-prepared removes only proven owned residue. Inject
an unrelated file/replaced inode before discard: refuse and preserve it. Crash during abort
and during derived pointer update, then reconcile again. Keep failures as evidence, never
report a preparation-only success as migration completion.

### 2.12 WTSTORE-12 — Decision, uncertainty and replay

Terminate before/after atomic decision publication, after file sync but before directory sync,
after durable decision before status update, after status update before response and during
response delivery. Inject sync failures to require exit 7. Status/retry under leases resolves
the same request to a single decision or remains explicitly unknown; no new ID, duplicate
history or post-decision abort. If decision is absent after conclusive recovery, resume
preparation with full revalidation; do not infer rollback from response loss. Repeat reservation publication/sync uncertainty from WTSTORE-11, distinguishing it from
cutover uncertainty: an authenticated reservation alone never authorizes the target. Remove cache
and original request and retry through both permanent routes (even with no root pointers).
For both source/destination entry, explicitly cross source present/absent with decision
publication uncertainty and directory-sync uncertainty: confirmed durable decision yields
the identical committed result even with source absent; unresolved read/publication/sync
returns `migration_outcome_unknown` (exit 7); only conclusive decision absence permits
absent-source `identity_unprovable` (exit 5). With conclusive absence and source present,
exact apply resumes full preparation. Request recovery still exports the durable reservation's
exact bytes while decision confirmation fails; apply without any supplied/recovered full
request remains invalid input, never guessed. Confirmed decision refuses abort/discard and
new-ID retries; unresolved decision permits none. Repeat absent destination after confirmed
decision and assert identity refusal, without rewriting the retained decision. Assert terminal
request/result/mapping equality and source write fence. Corrupt/remove journal/decision or
replace target: fail closed. These are process/I/O-fault tests, not physical power-loss tests.

### 2.13 WTSTORE-13 — Regression and final evidence

After implementation, run pinned compiler parse, model-only and full checks; document audit
and strict build; Rust suite; existing SimpleFlow, stage/store/Web/checkpoint tests; Python
suite and model-renderer tests; focused final figure checks and visual review. Use offline
cached Rust tools, `CARGO_HOME=cache/cargo-home`, `CARGO_TARGET_DIR=cache/target`, and cache
TMP variables. Run the new real-binary acceptance matrix and old-version compatibility tests
with per-case receipts/logs. Record all skips and preexisting failures, exact tested revision,
working-tree diff/source hashes and immutable evidence manifests. Re-run affected cases
when code changes after evidence capture. Preserve all 13 original acceptance rows.

Compiler unevaluated rules and non-evaluated constraint bodies remain limits. A design review,
syntactic SimpleFlow gate, passing docs build or test process exit is not evidence that all
cases passed. Final engineering/user acceptance, unsupported platforms, old-client fencing,
hostile metadata replay and physical power loss remain separate obligations.

## 3. Machine-checkable state and operation oracle

The versioned [oracle matrix](assets/worktree-store-oracle.json) is a design product, not
executed WTSTORE results. It enumerates source/destination outcomes for every phase and
operation class and defines Cartesian handle, alias, missing-root and default-selection
overlays. Its symbols resolve to exact I21 reason, store CLI exit, lifecycle exit and HTTP
status; `null` means that surface has no such endpoint. Read success means validated
consistent retained history; `diagnostic` never exposes partial history. `replay` means
identical terminal bytes, with no new store effect. Store operation request output and
migration request recovery are separate operation classes.

Operation surfaces are explicit masks: query/bytes have store and Web endpoints only,
completed CLI next/check has lifecycle only, and work restoration has lifecycle and Web.
Thus query/bytes always have `lifecycle_exit=null`; completed_read always has `http=null`
and `store_exit=null`, including location refusals. The oracle's separate completed Web
cases cover `GET /api/wishes/{id}/work` (HTTP 200 completed work) and
`POST /api/wishes/{id}/check` (HTTP 409 `already_delivered` after next reports completed).
That POST performs no check/save; it is not the CLI's completed check success. I21 location
refusals take precedence and retain their existing status mappings on each actual surface.

Apply overlays in the listed precedence; no wildcard permits an unspecified success.
Unsupported alias and invalid-identity cells are refusals even for a held handle. A held
handle that cannot have been constructed uses its recorded construction refusal; tests must
not invent one. Unheld/fresh and supported aliases otherwise share identical outcomes.
Busy physical lease ownership overrides diagnostic access too; the diagnostic cells assume
the complete lease was acquired. Migration operations have no HTTP route; Web health reports
a selection diagnostic without claiming successful history access. The fixture checker
checks the matrix shape plus focused corrected cells, including the ordered
`decision_resolution` resolver (`confirmed`, `unresolved`, `absent`). That resolver precedes
the absent-source apply overlay; confirmed reuses the committed row, unresolved retains
exit 7, and conclusive absence reuses prepared-state preconditions. A focused design-contract
fixture need not expand unaffected Cartesian axes. Real implementation must execute the
applicable cells before acceptance.

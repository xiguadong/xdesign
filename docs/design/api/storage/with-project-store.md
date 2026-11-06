# `with_project_store`

> Layer: `xdesign::storage::cli` | Status: planned additive library/binary adapter bridge.
> Language-side rationale: the stage CLI binary needs a command-long lease without an
> unchecked import API. Allocation: `PlannedWorktreeStoreAccess`; planned source:
> `src/storage/{cli,access,mod}.rs`.

<a id="u115"></a>

## Guarded command access

```rust
pub enum WithProjectStoreError<E> {
    Store(RsStoreError),
    Action(E),
}

pub fn with_project_store<T, E>(
    project_root: &std::path::Path,
    root: &std::path::Path,
    initialize: bool,
    action: impl for<'a> FnOnce(&'a RsStore) -> Result<T, E>,
) -> Result<T, WithProjectStoreError<E>>;

pub fn validate_project_write(store: &RsStore) -> Result<(), RsStoreError>;
pub fn read_completed_project(store: &RsStore, item_id: &str)
    -> Result<Option<serde_json::Value>, RsStoreError>;
pub fn project_candidate_contains(store: &RsStore, project: &std::path::Path,
    item_id: &str) -> Result<bool, RsStoreError>;
```

`T` and `E` are fixed outside the higher-ranked borrow. A callback may return owned data,
including a `String` error, but cannot return a borrow of the supplied store. `RsStore`
provides no public clone, raw connection, lease accessor or owned-store extraction.
`Store` carries selection/lease/open/init failures intact; `Action` carries the callback's
error unchanged. Neither arm converts to `InvalidInput` or parses free-form English.

Implementation refinement: the binary calls `validate_project_write` on its borrowed
handle before cache restoration, checker scratch writes or terminal-retry cache publication.
It reuses the held lease, checks the live database and mutation fence/source baseline, and
returns the typed error unchanged. It does not create a transferable authorization token;
save still validates again immediately before commit. The Web's internal optional-store
read bridge similarly holds classification and material reads under one command lease,
including the absent-store case, and releases it before waiting for a lifecycle child.
`read_completed_project` is the terminal read counterpart: it validates the retained
head, plan and run at a location whose mutation guard reports `source_retired` or
`source_changed`, then returns the existing completed response without cache publication.
An unfinished item or a writable location returns `None`, preserving ordinary cache
restoration. This additive adapter does not change
the response, state-token grammar, flow rules or operation-retry authority.
`project_candidate_contains` rechecks all known candidate roots under the command's
already acquired directory set. A newly required parent returns `migration_busy` for an
outer retry; unreadable candidates return `backend_ambiguous`. The binary uses the result
for legacy same-ID conflict detection before continuation, rather than relying on its
earlier unlocked candidate scan.

The root is already selected after the adapter's initial backend classification in
[D117 §4](../../worktree-store-migration.md#d117). Run the complete probe/release/collect/
deduplicate/order/acquire/revalidate algorithm in D117 §2.1.1, with project association and
source-baseline checks. Under the acquired set, classification is repeated before cache or
history mutation. `initialize=true` uses private guarded initialization only where eligible;
otherwise guarded open. Keep the lease through cache restoration, checker execution, final
source/live/fence check, commit and response construction. Stage helpers receive this
borrowed handle instead of reopening it.

### Private ownership and nested operations

A lifetime-free `RsStore` privately holds `Option<Arc<AccessLease>>` for a command handle.
`AccessLease` owns the pinned descriptors, lock guards, validated generation/context and
process-local owner token; it contains no reference to the callback. Generation bookkeeping
uses short private mutex scopes; no mutex guard is stored across a callback. Descriptor
ownership and synchronized bookkeeping preserve existing auto-trait compatibility; `Arc`
does not make SQLite shared access safe or introduce a `Sync` promise. The wrapper and store
hold strong references during the call. Private guarded helpers borrow `&AccessLease`
(or a short cloned `Arc` kept entirely inside the call), validate the token/context and
recheck live identity and fences. Nested public operations on **this handle** reuse that
lease and perform their own checks; they do not reacquire locks. Nested read/output helpers
use guarded variants and preserve the final output check. Guarded operations cannot expand
the lock set: unexpected peers/generations are an error requiring a new outer invocation.

An ordinary `RsStore::open` handle has `None` and acquires a fresh operation lease on each
method. A process-local acquisition registry refuses a fresh public open/init/bridge while
that thread holds a command lease (`migration_busy`), even for a different root; this
conservative rule prevents nested inversions. Other threads use the same physical lock-domain
registry plus kernel locks; they cannot get a recursive authorization. No new `Send`, `Sync`,
async callback or child-process inheritance contract is added. Close-on-exec descriptors
and process-local tokens cannot authorize a checker child. Same-root reentrant open returns
busy immediately, never waits for its own callback.

On success, callback error, setup error after partial acquisition, or Rust unwind, RAII drops
all store/temporary references and releases descriptors in reverse order when the last
private lease owner drops. `panic=abort`/process death relies on OS descriptor release; durable
fences remain. Drop cleanup never panics on a poisoned bookkeeping mutex and never needs callback data.
No reference cycle is permitted. There is no unsafe lifetime extension or
public lease cloning. The callback cannot keep a usable command handle after wrapper return.

### Actual String-based stage adaptation

Today's `persistent::{init,check,load}`, `observe` and many helpers return `Result<_, String>`.
The generic bridge accepts such a callback directly and preserves its String as `Action`;
this proves compatibility, but cannot recover a store error already formatted by today's
`persistent::error`. The planned binary changes therefore introduce a private typed error:

```rust
enum StageError { Store(RsStoreError), Message(String) }
impl From<String> for StageError { /* Message, without reinterpretation */ }
impl From<RsStoreError> for StageError { /* Store, lossless */ }
impl From<WithProjectStoreError<StageError>> for StageError {
    /* Store(e) => StageError::Store(e), Action(e) => e */
}
```

In `src/stage_cli/{mod,store,persistent}.rs`, helpers that can produce/propagate store errors
(`init`, `check`, `load`, `observe`, recovery, save and material reads) change their **private**
error type to `StageError`, taking `&RsStore` where needed. Pure String-producing validators
remain unchanged and adapt with `map_err(StageError::from)`/`?`. Replace early
`map_err(persistent::error)` with typed conversion; the old formatting boundary moves to
final dispatch rendering. These are planned source changes, not claims that the current
String signatures already preserve typed errors. Public storage signatures/variants stay
unchanged except this additive bridge/wrapper.

The final binary renderer preserves lifecycle success/completed exit 0, checked blocked
work exit 1 and command-error exit 2. It formats `UnknownOutcome { operation_id }`, and
`Io`/`Database` optional operation IDs without loss; save IDs never become migration IDs.
It recovers `location_reason` only from the defined stable prefix in a **Store** variant,
retaining the I21 code and unchanged location message for Web's mapping. Structured
selected-project/root fields come from the adapter’s captured context, never from parsing
English; migration-specific structured diagnostics stay in its private command type. An arbitrary String
message containing that prefix cannot forge a typed location diagnostic: final
`StageError::Message` rendering prefixes `stage_error;` before its unchanged message, while
only `StageError::Store` may render a leading `location_reason` token. Plan/work/run
JSON and historical response/receipt bytes are not rewritten. The store CLI retains its
separate I21 exit map; adapting a callback is not permission to flatten either protocol.

Web releases preflight before starting a lifecycle child; the child owns its command lease.
A genuine legacy command takes the private binary-side project/backend wrapper using the
same discovery/order algorithm and performs no persistent open/enrollment merely for legacy
read/continuation. WTSTORE-08/10 cover that route. Compile-level contract fixtures exercise
String and typed callbacks, uncertain IDs, nested guarded methods, early errors, unwind,
reentrant busy and a rejected escaped borrow; they do not implement storage or prove OS locks.

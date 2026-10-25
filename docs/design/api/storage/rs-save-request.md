# `RsSaveRequest`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes `XDesignStorage::SaveRequest` in
> [D86](../../storage.md#d86); implementation projection `src/storage/mod.rs`.

<a id="u80"></a>

## RsSaveRequest

```rust
use serde_json::{Map, Value};

pub struct RsSaveRequest {
    pub schema_version: u32,
    pub operation_id: String,
    pub item_id: String,
    pub kind: String,
    pub expected_revision: u64,
    pub metadata: Map<String, Value>,
    pub materials: Vec<RsSaveMaterial>,
    pub result: Value,
}
```

One exact logical save request. A struct literal alone is not validated authority;
[`RsStore::save`](rs-store-save.md#u82) validates every field before reserving the operation.

**Fields**

| Field | Contract |
| --- | --- |
| `schema_version` | Exactly `1`; another request version is unsupported rather than interpreted as version 1. |
| `operation_id` | Stable identity of this exact operation; follows the common operation ID rule. Exact request bytes, not only this ID or JSON semantics, govern retry. |
| `item_id` | Target generic item; follows the common item ID rule. A wish uses its wish ID. |
| `kind` | Immutable item kind token. Creation fixes it; later saves must retain it. |
| `expected_revision` | `0` creates the item. A positive value appends only when it equals the current head in the commit transaction; the next revision must fit the integer contract. |
| `metadata` | A JSON object by construction. It has no independently authoritative wish-status field. Wish metadata requirements apply when `kind == "wish"`. |
| `materials` | The complete desired manifest, including unchanged entries. URIs are unique and every [`RsSaveMaterial`](rs-save-material.md#u85) is validated; this is not a delta. |
| `result` | Any strict JSON value retained in the receipt. The store does not interpret it as a gate verdict or proof of commit. |

Common IDs, digests, URIs, JSON limits, metadata, and size rules are owned by
[I13](../../../interfaces/store-format.md#i13); they are not delegated to callers.

**Errors and conflicts**

Malformed fields are `RsStoreError::InvalidInput`; an unsupported request schema is
`UnsupportedFormat`. Source and path failures are reported by the save operation. Reusing
`operation_id` with different valid request bytes is `OperationMismatch`. A stale expected
head, create collision, or kind conflict is **not** an error: save durably records and returns
an [`RsOperationReceipt`](rs-operation-receipt.md#u90) whose outcome is
[`Conflict`](rs-operation-outcome.md#u89), whose `revision` is `None`, and whose `result` is
only the proposed result.

**Persistence**

Constructing, cloning, serializing, or dropping this value has no storage effect. A successful
save either commits the complete revision, receipt, and head advance together or commits only
a conflict receipt. Failure after reservation can leave a pending receipt; an error does not
prove absence of effects.

**Logic**

1. Validate the complete typed request.
2. Encode it as deterministic compact JSON: recursively sorted object members, no
   insignificant whitespace, and primitive forms from the pinned `serde_json` serializer.
   This is the project's deterministic encoding, **not** RFC 8785/JCS interoperability.
3. Hash and retain those exact bytes for operation identity.
4. Execute the exact-operation reservation, publication, and commit/conflict protocol in
   [`RsStore::save`](rs-store-save.md#u82).

**Example** (illustrative, not executed):

```rust
let request = RsSaveRequest {
    schema_version: 1,
    operation_id: "example-save-1".into(),
    item_id: "example-item".into(),
    kind: "example".into(),
    expected_revision: 0,
    metadata: serde_json::Map::new(),
    materials: vec![],
    result: serde_json::Value::Null,
};
assert_eq!(request.expected_revision, 0); // create, not append
```

**See also:** [request JSON I13](../../../interfaces/store-format.md#i13),
[`RsSaveMaterial`](rs-save-material.md#u85), and [`RsStore::save`](rs-store-save.md#u82).

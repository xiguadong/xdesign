# `RsMaterial`

> `xdesign::storage` · struct · implemented; STORE-01–STORE-10 evidence executed within disclosed limits · realizes `XDesignStorage::Material` in
> [D86](../../storage.md#d86); implementation projection `src/storage/mod.rs`.

<a id="u86"></a>

## RsMaterial

```rust
pub struct RsMaterial {
    pub uri: String,
    pub role: String,
    pub sha256: String,
    pub byte_length: u64,
}
```

The committed, output-side form of one complete-manifest entry. Unlike
[`RsSaveMaterial`](rs-save-material.md#u85), it exposes verified byte length and has no source
path.

**Fields**

| Field | Contract |
| --- | --- |
| `uri` | Logical project-relative URI, unique within the containing revision. |
| `role` | Role copied from the accepted request entry. |
| `sha256` | Lowercase SHA-256 identity of the exact published blob; the physical path derives only from this digest. |
| `byte_length` | Exact blob length in bytes, bounded by the 256 MiB material cap and checked through EOF before trusted reads. |

Serialization names and required-field rules are in
[I13](../../../interfaces/store-format.md#i13). A manifest entry is eligible only when it
belongs to a committed revision; a digest-identical orphan is not a committed material.

**Errors**

The value has no independent fallible operation. Invalid stored manifest rows or references
are `RsStoreError::Integrity`. Missing item/revision/URI selection is `NotFound`; unsafe blob
paths are `UnsafePath`; missing, truncated, altered, or overlong stored bytes fail verified
reads rather than yielding an unverified prefix.

**Persistence**

Constructing, cloning, serializing, or dropping this value has no storage effect. In a returned
revision it describes an immutable manifest row and its content-addressed blob. Changing a
workspace source later does not change this value or the selected revision.

**Logic**

Save converts each accepted [`RsSaveMaterial`](rs-save-material.md#u85) into this committed
form after resolving its exact bytes. Revision queries return all entries in deterministic URI
order. [`RsStore::read_material`](rs-store-read-material.md#u104) uses the digest-derived path,
then independently checks recorded length and SHA-256 before returning bytes.

**Example** (illustrative value, not proof of a stored blob):

```rust
let material = RsMaterial {
    uri: "docs/design/storage.md".into(),
    role: "artifact".into(),
    sha256: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".into(),
    byte_length: 4096,
};
assert_eq!(material.byte_length, 4096);
```

**See also:** [`RsItemRevision`](rs-item-revision.md#u87),
[`RsStore::read_material`](rs-store-read-material.md#u104), and
[physical blob rules I12](../../../interfaces/store-format.md#i12).

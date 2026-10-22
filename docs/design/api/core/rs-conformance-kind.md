# `RsConformanceKind`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `ConformanceKind` ([F02](../../adapters.md#f02))

<a id="u50"></a>

## RsConformanceKind

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsConformanceKind {
    ArtifactPresence,
    IdentifierConformance,
    DefinitionCoverage,
    TraceConformance,
}
```

RsConformanceKind names the conformance classes a fixed checker says it establishes. Use it to declare what a syntactic gate proves and what remains outside its evidence.

| Variant | Meaning |
| --- | --- |
| `ArtifactPresence` | Required artifacts exist |
| `IdentifierConformance` | Model identifiers resolve and agree |
| `DefinitionCoverage` | Declared design definitions cover the required elements |
| `TraceConformance` | Required trace links and records conform |

**See also:** [`RsCheckerSpec`](rs-checker-spec.md#u35), [checkpoint kinds F02](../../adapters.md#f02).

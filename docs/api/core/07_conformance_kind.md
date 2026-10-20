# `RsConformanceKind`

> `xdesign::core::status` · enum · implemented xdesign slice · mirrors `ConformanceKind` ([F02](../../details/01_flows_and_checkpoints.md#f02))

<a id="u50"></a>

## U50 RsConformanceKind

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RsConformanceKind {
    ArtifactPresence,
    IdentifierConformance,
    DefinitionCoverage,
    TraceConformance,
}
```

The conformance classes a fixed checker says it establishes. These variants
mirror the SysML enum and are carried by
[`RsCheckerSpec`](15_checker_spec.md#u35).

| Variant | Meaning |
| --- | --- |
| `ArtifactPresence` | Required artifacts exist |
| `IdentifierConformance` | Model identifiers resolve and agree |
| `DefinitionCoverage` | Declared design definitions cover the required elements |
| `TraceConformance` | Required trace links and records conform |

**See also:** [`RsCheckerSpec`](15_checker_spec.md#u35), [checkpoint kinds F02](../../details/01_flows_and_checkpoints.md#f02).

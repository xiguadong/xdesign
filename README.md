# xdesign

xdesign is a SysML v2 engineering tool: a project model explains the system, and an
independent trace for each wish explains why its requirements, design, implementation, and
verification followed. The implemented slices are the gated four-stage SimpleFlow and the
gated two-stage wish-verify flow; the generic controller and full V-model remain open.

Read the documentation from [docs/index.md](docs/index.md). Current capability status and
evidence are maintained in [docs/records/status.md](docs/records/status.md).

Build or preview the documentation:

```bash
python3 -B tools/docs.py build
python3 -B tools/docs.py serve
```

The wrapper keeps its environment and output under `cache/docs-site/`. The documentation
set is organized as requirements, architecture, design (including one document per API),
interfaces, verification design, and records. Authoring rules are in
[docs/AGENTS.md](docs/AGENTS.md); repository-wide agent rules are in [AGENTS.md](AGENTS.md).

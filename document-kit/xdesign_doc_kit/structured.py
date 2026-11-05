"""Schema 1.2 operations sharing rendering and validation with retained packs."""

from __future__ import annotations

import json
import os
import re
from pathlib import Path, PurePosixPath

from . import core as c
from .compatibility import DocumentKitError, _error, diagnostic
from .catalog import select_template, _section_blocks, _parse_decisions, DECISION_FIELDS
from .routes import request_selection, resolve, reverse
from .io_boundary import LAUNCH_PIN, CURRENT, phase, published, annotate


def preflight(
    operation,
    *,
    document_class=None,
    name=None,
    layer=None,
    metadata=None,
    sections=None,
    section_key=None,
    content=None,
    fields=None,
    figure=None,
    expected_sha256=None,
    expected_contract_sha256=None,
    template_id=None,
    concern=None,
    decision=None,
    selection=None,
):
    c._validate_expected_contract_request(expected_contract_sha256)
    if operation in (
        "plan_document",
        "create_document",
        "update_section",
        "attach_figure",
    ):
        request_selection(document_class, name, layer)
        if template_id is not None and (
            not isinstance(template_id, str)
            or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", template_id)
        ):
            raise _error("unknown-template", "Malformed template ID")
    if operation == "create_document":
        if not isinstance(metadata, dict) or set(metadata) != {
            "title",
            "status",
            "revision",
            "model_sources",
        }:
            raise _error(
                "missing-metadata",
                "Metadata needs title, status, revision, and model_sources",
                "DOC-002",
            )
        if any(
            not isinstance(metadata[k], str) for k in ("title", "status", "revision")
        ) or not isinstance(metadata["model_sources"], list):
            raise _error("missing-metadata", "Metadata types are invalid", "DOC-002")
        if not isinstance(sections, dict):
            raise _error("invalid-request", "sections must be an object")
        for key, section in sections.items():
            if (
                not isinstance(key, str)
                or not isinstance(section, dict)
                or set(section) != {"anchor", "content", "fields"}
                or not isinstance(section["anchor"], str)
                or not isinstance(section["content"], str)
                or not isinstance(section["fields"], dict)
                or any(
                    not isinstance(k, str) or not isinstance(v, str)
                    for k, v in section["fields"].items()
                )
            ):
                raise _error(
                    "invalid-request",
                    "Each section needs anchor, content, and a string field map",
                )
    if operation in ("update_section", "attach_figure"):
        if (
            not isinstance(section_key, str)
            or not isinstance(expected_sha256, str)
            or not re.fullmatch("[0-9a-f]{64}", expected_sha256)
        ):
            raise _error(
                "invalid-request",
                "section_key and a lowercase expected_sha256 are required",
            )
    if operation == "update_section":
        if not isinstance(content, str) or (
            fields is not None
            and (
                not isinstance(fields, dict)
                or any(
                    not isinstance(k, str) or not isinstance(v, str)
                    for k, v in fields.items()
                )
            )
        ):
            raise _error("invalid-request", "content and fields have invalid types")
    if operation == "attach_figure":
        required = {
            "asset",
            "title",
            "Question",
            "Model view",
            "Companions",
            "Status",
            "Reading",
            "Limits",
        }
        if (
            not isinstance(figure, dict)
            or set(figure) != required
            or any(not isinstance(v, str) for v in figure.values())
        ):
            raise _error(
                "invalid-request",
                "Figure needs asset, title, and all six string fields",
            )
        if concern is not None and not isinstance(concern, str):
            raise _error("invalid-request", "concern must be text or null")
        if decision is not None and (
            not isinstance(decision, dict)
            or set(decision) != DECISION_FIELDS
            or not isinstance(decision["applies"], bool)
            or any(
                not isinstance(decision[k], str)
                for k in ("concern", "choice", "rationale", "asset", "model_view")
            )
        ):
            raise _error(
                "invalid-request", "decision must have the six typed decision fields"
            )
    if operation == "check_documents":
        if document_class is not None and not isinstance(document_class, str):
            raise _error("invalid-request", "document_class must be text or null")
        if selection is not None:
            if (
                not isinstance(selection, dict)
                or not {"document_class", "name"}.issubset(selection)
                or not set(selection).issubset({"document_class", "name", "layer"})
            ):
                raise _error(
                    "invalid-request",
                    "selection needs document_class, name and optional layer",
                )
            request_selection(
                selection["document_class"], selection["name"], selection.get("layer")
            )
            if (
                document_class is not None
                and document_class != selection["document_class"]
            ):
                raise _error("invalid-request", "document_class and selection disagree")


def load(profile_root, expected=None):
    phase("pack-read")
    contract = c._contract(profile_root)
    c._expected_contract(contract, expected)
    pin = LAUNCH_PIN.get()
    if pin is not None and (
        contract["contract_sha256"] != pin or expected is not None and expected != pin
    ):
        raise _error("contract-drift", "Selected contract differs from MCP launch pin")
    return contract


def plan_parts(
    target_root,
    document_class,
    name,
    profile_root,
    template_id=None,
    layer=None,
    expected=None,
):
    selection = request_selection(document_class, name, layer)
    contract = load(profile_root, expected)
    relative = resolve(contract["profile"], selection)
    choice = select_template(contract, document_class, template_id)
    phase("route", relative)
    target = c._root(target_root, "Target project root")
    c._reject_symlinks(target, PurePosixPath(relative))
    return (
        contract,
        target,
        choice["spec"],
        relative,
        choice["effective_identity"],
        selection,
    )


def plan_document(
    target_root,
    document_class,
    name,
    profile_root=None,
    template_id=None,
    *,
    layer=None,
):
    contract, target, spec, relative, effective, selection = plan_parts(
        target_root, document_class, name, profile_root, template_id, layer
    )
    exists = (target / relative).exists()
    digest = c._sha(c._read_project_file(target, relative)) if exists else None
    return dict(
        ok=True,
        **c._identity(contract),
        document_class=document_class,
        name=name,
        path=relative,
        exists=exists,
        sha256=digest,
        template_id=effective["template_id"],
        effective_identity=effective,
        route_selection=selection,
        required_metadata=["title", "status", "revision", "model_sources"],
        required_sections=spec["required_sections"],
        anchor_policy=contract["profile"]["anchor_policy"],
        figure_block=contract["profile"]["figure_block"],
        semantic_review_required=True,
    )


def anchors_and_findings(texts, focus=None):
    occurrences = {}
    for relative, text in sorted(texts.items()):
        for number, line in enumerate(c._masked_lines(text), 1):
            match = c.ANCHOR.fullmatch(line)
            if match:
                occurrences.setdefault(match[1].casefold(), []).append(
                    (relative, number)
                )
    findings = []
    for anchor, locations in occurrences.items():
        if len(locations) > 1 and (
            focus is None or any(p == focus for p, n in locations)
        ):
            for relative, line in locations:
                if focus is None or relative == focus:
                    findings.append(
                        diagnostic(
                            "DOC-002",
                            "duplicate-anchor",
                            relative,
                            f"Anchor {anchor} occurs at "
                            + ", ".join(f"{p}:{n}" for p, n in locations),
                            line,
                        )
                    )
    return {k: v[0] for k, v in occurrences.items()}, findings


def managed_findings(target, contract, spec, effective, relative, text, models):
    findings = c._managed_findings(
        relative, text, spec, contract, target, models, effective
    )
    selection = reverse(spec, relative, spec["document_class"])
    if selection is None:
        findings.append(
            diagnostic(
                "DOC-001",
                "invalid-document-name",
                relative,
                "Managed path differs from effective class route",
            )
        )
    if (
        contract["profile"]["schema_version"] == "1.2"
        and spec.get("selector_kind") == "api"
    ):
        for line, row in enumerate(c._masked_lines(text), 1):
            match = c.ANCHOR.fullmatch(row)
            if match and not re.fullmatch("U[0-9]+", match[1]):
                findings.append(
                    diagnostic(
                        "DOC-002",
                        "missing-anchor",
                        relative,
                        "API anchors require allocated U IDs",
                        line,
                    )
                )
        blocks = _section_blocks(text, spec["required_sections"])
        layer = re.search(r"^\*\*Layer:\*\* ([^\n]+)$", blocks.get("purpose", ""), re.M)
        if selection and (layer is None or layer[1].strip() != selection["layer"]):
            findings.append(
                diagnostic(
                    "DOC-002",
                    "invalid-request",
                    relative,
                    "API layer field differs from route selection",
                )
            )
    # A bare non-applicability label is not an explanation.
    for line, row in enumerate(c._masked_lines(text), 1):
        if contract["profile"]["schema_version"] == "1.2" and re.fullmatch(
            r"\*\*[^*]+:\*\*\s*not applicable[.\s]*", row, re.I
        ):
            findings.append(
                diagnostic(
                    "DOC-002",
                    "missing-field",
                    relative,
                    "Not applicable requires an author reason",
                    line,
                )
            )
    return findings


def validate_candidate(
    target, selected, contract, spec, effective, relative, text, previous=None
):
    texts = c._existing_texts(target, selected["profile"]["project"]["docs_root"])
    texts[relative] = text
    anchors, findings = anchors_and_findings(texts)
    models = set(c._model_files(target, selected["profile"])) | set(
        c._model_files(target, contract["profile"])
    )
    findings += managed_findings(
        target, contract, spec, effective, relative, text, sorted(models)
    )
    findings += c._local_link_findings(target, relative, text, anchors)
    findings += c._annotation_findings(
        target, sorted(models), texts, anchors, focus=relative
    )
    if previous and contract["profile"]["schema_version"] == "1.2":
        old_blocks = _section_blocks(previous, spec["required_sections"])
        new_blocks = _section_blocks(text, spec["required_sections"])
        for key, old in old_blocks.items():
            new_decisions = _parse_decisions(new_blocks.get(key, ""), key)
            for decision in _parse_decisions(old, key):
                if decision["choice"] == "figure" and f"]({decision['asset']})" in old:
                    match = next(
                        (
                            d
                            for d in new_decisions
                            if d["concern"] == decision["concern"]
                        ),
                        None,
                    )
                    if match is None or any(
                        match[k] != decision[k]
                        for k in ("choice", "applies", "asset", "model_view")
                    ):
                        findings.append(
                            diagnostic(
                                "DOC-005",
                                "invalid-diagram-decision",
                                relative,
                                "Update cannot remove or retarget an attached figure decision",
                            )
                        )
    findings.sort(key=lambda f: (f["file"], f["line"] or 0, f["code"], f["detail"]))
    if findings:
        raise DocumentKitError(findings[0]["code"], findings[0]["detail"], findings)


def commit(profile_root, contract, expected):
    current = load(profile_root, expected)
    if current["contract_sha256"] != contract["contract_sha256"]:
        raise _error("contract-drift", "Contract bytes changed before publication")


def cleanup_creation(target, relative, data, created_stat, failure):
    """Only unlink our unchanged inode through the held root, never a replacement."""
    try:
        raw = c._read_project_file(target, relative)
        with c._parent_fd(target, PurePosixPath(relative)) as (parent, name):
            now = os.stat(name, dir_fd=parent, follow_symlinks=False)
            if (now.st_dev, now.st_ino) != created_stat or raw != data:
                raise _error(
                    "publication-unknown", "Created page changed before cleanup"
                )
            phase("cleanup", relative)
            os.unlink(name, dir_fd=parent)
    except (OSError, DocumentKitError) as cleanup_error:
        try:
            unchanged = c._read_project_file(target, relative) == data
            with c._parent_fd(target, PurePosixPath(relative)) as (parent, name):
                now = os.stat(name, dir_fd=parent, follow_symlinks=False)
                unchanged = unchanged and (now.st_dev, now.st_ino) == created_stat
        except (OSError, DocumentKitError):
            unchanged = False
        phase("cleanup", relative)
        published("page-only" if unchanged else "unknown")
        raise annotate(
            _error(
                "publication-incomplete" if unchanged else "publication-unknown",
                "Registry publication failed and creation cleanup could not be confirmed",
            )
        ) from cleanup_error
    published("none")
    if isinstance(failure, DocumentKitError):
        if failure.outcome:
            failure.outcome["publication_state"] = "none"
        raise failure
    phase("registry-publish", relative)
    raise annotate(
        _error(
            "access-denied",
            "Registry publication failed; newly created page was removed",
        )
    ) from failure


def create_document(
    target_root,
    document_class,
    name,
    metadata,
    sections,
    profile_root=None,
    expected_contract_sha256=None,
    template_id=None,
    *,
    layer=None,
):
    contract, target, spec, relative, effective, selection = plan_parts(
        target_root,
        document_class,
        name,
        profile_root,
        template_id,
        layer,
        expected_contract_sha256,
    )
    known = c._model_files(target, contract["profile"])
    c._model_sources(target, contract["profile"], metadata["model_sources"], known)
    with c._writer_lock(target):
        if (target / relative).exists():
            raise _error(
                "document-exists",
                f"Document already exists: {relative}",
                "DOC-001",
                relative,
            )
        registry = c._load_registry(target, contract)
        if any(e["path"] == relative for e in registry["entries"]):
            raise _error(
                "registry-mismatch",
                f"Registry already owns {relative}",
                "DOC-002",
                relative,
            )
        rendered, anchors = c._document_parts(
            contract, target, document_class, metadata, sections, known, spec, effective
        )
        validate_candidate(
            target, contract, contract, spec, effective, relative, rendered
        )
        commit(profile_root, contract, expected_contract_sha256)
        data = rendered.encode("utf-8")
        c._replace(target, relative, data, create=True)
        try:
            with c._parent_fd(target, PurePosixPath(relative)) as (parent, filename):
                st = os.stat(filename, dir_fd=parent, follow_symlinks=False)
                created_stat = (st.st_dev, st.st_ino)
        except OSError as error:
            published("unknown")
            raise annotate(
                _error("publication-unknown", "Cannot establish new page identity")
            ) from error
        registry["entries"].append(
            c._registry_entry(contract, relative, document_class, effective)
        )
        try:
            c._replace(target, c._registry_path(contract), c._registry_bytes(registry))
        except (OSError, DocumentKitError) as failure:
            if CURRENT.get().publication_state == "unknown":
                phase("registry-publish", relative)
                raise annotate(
                    _error(
                        "publication-unknown",
                        "Registry publication effect requires inspection",
                    )
                ) from failure
            cleanup_creation(target, relative, data, created_stat, failure)
    phase("response", relative)
    return dict(
        ok=True,
        **c._identity(contract),
        path=relative,
        sha256=c._sha(data),
        anchors=anchors,
        template_id=effective["template_id"],
        effective_identity=effective,
        route_selection=selection,
        semantic_review_required=True,
    )


def change_fields(source, spec, key, fields):
    section = next((s for s in spec["required_sections"] if s["key"] == key), None)
    if section is None:
        raise _error("invalid-request", f"Unknown section key: {key}")
    if fields is None:
        return source
    if not set(fields).issubset(section["required_fields"]):
        raise _error("missing-field", f"Unexpected fields in {key}", "DOC-002")
    start, end = c._section_span(source, spec, key)
    block = source[start:end]
    field_end = block.find(c.CONTENT_START.format(key=key))
    if field_end < 0:
        raise _error("empty-content", "Content boundary is missing")
    area = block[:field_end]
    for field, value in fields.items():
        label = field.replace("_", " ").capitalize()
        pattern = re.compile(rf"(?m)^(\*\*{re.escape(label)}:\*\*)[^\n]*$")
        if len(pattern.findall(area)) != 1:
            raise _error("missing-field", f"Field {label} is missing")
        area = pattern.sub(
            lambda m: m[1] + " " + c._text(value, field, "missing-field"), area
        )
    return source[:start] + area + block[field_end:] + source[end:]


def update_section(
    target_root,
    document_class,
    name,
    section_key,
    content,
    expected_sha256,
    fields=None,
    profile_root=None,
    expected_contract_sha256=None,
    *,
    layer=None,
):
    contract, target, _, relative, _, selection = plan_parts(
        target_root,
        document_class,
        name,
        profile_root,
        layer=layer,
        expected=expected_contract_sha256,
    )
    with c._writer_lock(target):
        source, _, effective_contract, spec, effective = c._managed_current(
            target, contract, relative, document_class, expected_sha256
        )
        changed = change_fields(source, spec, section_key, fields)
        begin, finish = c._section_bounds(changed, section_key)
        changed = (
            changed[:begin]
            + "\n"
            + c._text(content, "Section content", "empty-content")
            + "\n"
            + changed[finish:]
        )
        validate_candidate(
            target,
            contract,
            effective_contract,
            spec,
            effective,
            relative,
            changed,
            source,
        )
        commit(profile_root, contract, expected_contract_sha256)
        if c._sha(c._read_project_file(target, relative)) != expected_sha256:
            raise _error("stale-document", "Document changed before commit")
        data = changed.encode()
        c._replace(target, relative, data)
    phase("response", relative)
    return dict(
        ok=True,
        **c._identity(contract),
        path=relative,
        sha256=c._sha(data),
        effective_identity=effective,
        route_selection=selection,
        semantic_review_required=True,
    )


def attach_figure(
    target_root,
    document_class,
    name,
    section_key,
    figure,
    expected_sha256,
    profile_root=None,
    expected_contract_sha256=None,
    concern=None,
    *,
    layer=None,
    decision=None,
):
    contract, target, _, relative, _, selection = plan_parts(
        target_root,
        document_class,
        name,
        profile_root,
        layer=layer,
        expected=expected_contract_sha256,
    )
    with c._writer_lock(target):
        source, _, effective_contract, spec, effective = c._managed_current(
            target, contract, relative, document_class, expected_sha256
        )
        if (
            decision is not None
            and effective_contract["profile"]["schema_version"] != "1.2"
        ):
            raise _error(
                "invalid-request", "decision requires a schema 1.2 effective pack"
            )
        if not any(s["key"] == section_key for s in spec["required_sections"]):
            raise _error("invalid-request", f"Unknown section key: {section_key}")
        selected_concern = next(
            (
                v
                for v in spec["view_concerns"]
                if v["section_key"] == section_key and v["concern"] == concern
            ),
            None,
        )
        if concern is not None and selected_concern is None:
            raise _error("invalid-diagram-decision", f"Unknown concern: {concern}")
        changed = source
        if selected_concern is not None:
            existing = _parse_decisions(
                _section_blocks(source, spec["required_sections"])[section_key],
                section_key,
            )
            matches = [d for d in existing if d["concern"] == concern]
            if not matches and selected_concern["applicability"] == "optional":
                if decision is None:
                    raise _error(
                        "missing-diagram-decision",
                        "Optional figure requires an explicit decision",
                    )
                if (
                    decision["concern"] != concern
                    or decision["applies"] is not True
                    or decision["choice"] != "figure"
                    or "figure" not in selected_concern["alternatives"]
                    or not decision["rationale"].strip()
                    or decision["asset"] != figure["asset"]
                    or decision["model_view"] != figure["Model view"]
                ):
                    raise _error(
                        "invalid-diagram-decision",
                        "Decision differs from supplied figure or permitted concern",
                    )
                changed = change_fields(
                    source,
                    spec,
                    section_key,
                    {
                        "diagram_decisions": json.dumps(
                            existing + [decision], ensure_ascii=False
                        )
                    },
                )
            elif decision is not None:
                raise _error(
                    "invalid-diagram-decision",
                    "Attachment cannot replace a stored decision",
                )
        elif decision is not None:
            raise _error(
                "invalid-diagram-decision", "decision needs a declared optional concern"
            )
        stored = c.attachment_decision(changed, spec, section_key, concern)
        if stored is not None and (
            stored["asset"] != figure["asset"]
            or stored["model_view"] != figure["Model view"]
        ):
            raise _error(
                "invalid-diagram-decision", "Figure differs from selected decision"
            )
        asset = figure["asset"]
        if not re.fullmatch(r"assets/[a-z0-9]+(?:-[a-z0-9]+)*\.svg", asset):
            raise _error(
                "unsafe-path", "Figure asset must be semantic assets/<name>.svg"
            )
        for k, v in figure.items():
            c._text(v, k, "figure-block")
        if (
            figure["Status"].split(";", 1)[0].strip()
            not in effective_contract["profile"]["figure_block"]["statuses"]
        ):
            raise _error("figure-status", "Unknown primary figure status")
        asset_relative = (PurePosixPath(relative).parent / asset).as_posix()
        c._read_project_file(target, asset_relative, "missing-figure")
        if f"]({asset})" in source:
            raise _error("duplicate-figure", "Asset is already embedded")
        start, end = c._section_span(changed, spec, section_key)
        block = changed[start:end]
        anchor = next(
            (
                m[1]
                for row in reversed(changed[:start].splitlines()[-3:])
                if (m := c.ANCHOR.fullmatch(row))
            ),
            None,
        )
        if anchor is None:
            raise _error("missing-anchor", "Section lacks an anchor")
        ordinal = (
            max(
                [
                    int(m[1])
                    for m in re.finditer(rf"(?m)^> \*\*Figure {anchor}-(\d+) —", block)
                ],
                default=0,
            )
            + 1
        )
        values = dict(
            asset=asset,
            title=figure["title"].strip(),
            section_id=anchor,
            ordinal=str(ordinal),
        )
        values.update(
            {
                token: figure[label].strip()
                for label, token in effective_contract["profile"]["figure_block"][
                    "field_tokens"
                ].items()
            }
        )
        rendered = c.TOKEN.sub(
            lambda m: values[m[1]], effective_contract["templates"]["figure-block"]
        )
        _, finish = c._section_bounds(changed, section_key)
        insertion = finish + len(c.CONTENT_END.format(key=section_key))
        changed = (
            changed[:insertion]
            + "\n\n"
            + rendered.rstrip()
            + "\n"
            + changed[insertion:]
        )
        validate_candidate(
            target,
            contract,
            effective_contract,
            spec,
            effective,
            relative,
            changed,
            source,
        )
        commit(profile_root, contract, expected_contract_sha256)
        if c._sha(c._read_project_file(target, relative)) != expected_sha256:
            raise _error("stale-document", "Document changed before commit")
        data = changed.encode()
        c._replace(target, relative, data)
    phase("response", relative)
    return dict(
        ok=True,
        **c._identity(contract),
        path=relative,
        sha256=c._sha(data),
        figure_number=f"{anchor}-{ordinal}",
        effective_identity=effective,
        route_selection=selection,
        provenance_review_notes=[
            "Review that the SVG faithfully renders the cited SysML view"
        ],
        semantic_review_required=True,
    )


def check_documents(
    target_root, document_class=None, profile_root=None, *, selection=None
):
    contract = load(profile_root)
    profile = contract["profile"]
    target = c._root(target_root, "Target project root")
    normalized = None
    focus = None
    if selection is not None:
        normalized = request_selection(
            selection["document_class"], selection["name"], selection.get("layer")
        )
        document_class = normalized["document_class"]
        focus = resolve(profile, normalized)
        c._reject_symlinks(target, PurePosixPath(focus))
    elif (
        document_class is not None and document_class not in profile["document_classes"]
    ):
        raise _error("invalid-request", f"Unknown document class: {document_class}")
    registry = c._load_registry(target, contract)
    entries = {e["path"]: e for e in registry["entries"]}
    if focus is not None and (
        focus not in entries or entries[focus]["class"] != normalized["document_class"]
    ):
        raise _error("registry-mismatch", "Selected document is not registered")
    models = set(c._model_files(target, profile))
    texts = c._existing_texts(target, profile["project"]["docs_root"])
    anchors, global_findings = anchors_and_findings(texts, focus)
    checked = []
    identities = {}
    findings = []

    def included(path):
        if focus is not None:
            return path == focus
        if document_class is None:
            return True
        if path in entries:
            return entries[path]["class"] == document_class
        return (
            reverse(profile["document_classes"][document_class], path, document_class)
            is not None
        )

    for path in sorted(set(texts) | set(entries)):
        if not included(path):
            continue
        checked.append(path)
        if path not in texts:
            global_findings.append(
                diagnostic(
                    "DOC-002",
                    "registry-mismatch",
                    path,
                    "Registered document is missing",
                )
            )
            continue
        text = texts[path]
        findings += c._local_link_findings(target, path, text, anchors)
        if path in entries:
            try:
                spec, effective_contract, effective = c.resolve_document(
                    contract, entries[path], text, path
                )
                known = c._model_files(target, effective_contract["profile"])
                models.update(known)
                identities[path] = effective
                findings += managed_findings(
                    target, effective_contract, spec, effective, path, text, known
                )
            except DocumentKitError as error:
                if error.code in (
                    "unreadable-input",
                    "access-denied",
                    "unsafe-path",
                    "missing-model-root",
                ):
                    raise
                global_findings += error.diagnostics
        else:
            if text.startswith("<!-- document-kit: profile="):
                global_findings.append(
                    diagnostic(
                        "DOC-002",
                        "registry-mismatch",
                        path,
                        "Managed marker has no registry entry",
                        1,
                    )
                )
            findings += c._figure_findings(target, path, text, contract, managed=False)
    global_findings += c._annotation_findings(
        target, sorted(models), texts, anchors, focus=focus
    )
    order = lambda f: (f["file"], f["line"] or 0, f["code"], f["detail"])
    global_findings.sort(key=order)
    findings = sorted(global_findings + findings, key=order)
    hashes = {p: c._sha(texts[p].encode()) for p in checked if p in texts}
    return dict(
        ok=not findings,
        **c._identity(contract),
        checked_paths=checked,
        scanned_paths=sorted(texts),
        sha256_by_path=hashes,
        document_hashes=hashes,
        effective_identity_by_path=identities,
        document_class_scope=document_class,
        scope=dict(
            document_class=document_class,
            selection=normalized,
            document_checks=checked,
            global_checks=["anchor-uniqueness", "registry-integrity", "model-locators"],
        ),
        global_findings=global_findings,
        diagnostics=findings,
        semantic_review_required=True,
        unevaluated_checks=[
            "SysML compiler syntax/name resolution",
            "engineering adequacy",
            "figure readability",
        ],
    )

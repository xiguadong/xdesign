"""JSON CLI for the shared document kit engine."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from .core import (_structured_selection, DocumentKitError, _unique_pairs, attach_figure, check_documents,
                   create_document, get_contract, plan_document, update_section)


class _InvocationError(ValueError):
    pass


class _Parser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise _InvocationError(message)


def _input(value: str) -> dict[str, Any]:
    try:
        path = Path(value)
        raw = value if value.lstrip().startswith("{") else path.read_text(encoding="utf-8") if path.is_file() else value
        parsed = json.loads(raw, object_pairs_hook=_unique_pairs)
    except (OSError, UnicodeError, ValueError) as error:
        raise DocumentKitError("invalid-request", f"Invalid JSON input: {error}") from error
    if not isinstance(parsed, dict):
        raise DocumentKitError("invalid-request", "JSON input must be an object")
    return parsed


def main(argv: list[str] | None = None) -> int:
    parser = _Parser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    for name in ("get-contract", "plan-document", "create-document", "update-section",
                 "attach-figure", "check-documents"):
        command = sub.add_parser(name)
        command.add_argument("--profile-root")
        if name != "get-contract":
            command.add_argument("--target-root", required=True)
        if name in ("plan-document", "create-document", "update-section", "attach-figure"):
            command.add_argument("--class", dest="document_class", required=True)
            command.add_argument("--name", required=True)
            command.add_argument("--layer")
        elif name == "check-documents":
            command.add_argument("--class", dest="document_class")
            command.add_argument("--name")
            command.add_argument("--layer")
        if name in ("plan-document", "create-document"):
            command.add_argument("--template-id")
        if name == "attach-figure":
            command.add_argument("--concern")
        if name in ("create-document", "update-section", "attach-figure"):
            command.add_argument("--input", required=True)
            command.add_argument("--expected-contract-sha256")
        if name in ("update-section", "attach-figure"):
            command.add_argument("--section", required=True)
    try:
        args = parser.parse_args(argv)
    except _InvocationError as error:
        print(json.dumps(DocumentKitError("invalid-request", str(error)).as_dict(),
                         ensure_ascii=False, sort_keys=True))
        return 2
    try:
        if args.operation in ("create-document", "update-section", "attach-figure"):
            payload = _input(args.input)
            if _structured_selection(args.profile_root):
                required, optional = {
                    "create-document": ({"metadata", "sections"}, set()),
                    "update-section": ({"content", "expected_sha256"}, {"fields"}),
                    "attach-figure": ({"figure", "expected_sha256"}, {"decision"}),
                }[args.operation]
                if not required.issubset(payload) or not set(payload).issubset(required | optional):
                    raise DocumentKitError("invalid-request", "JSON payload keys differ from the operation contract")
        if args.operation == "get-contract":
            result = get_contract(args.profile_root)
        elif args.operation == "plan-document":
            result = plan_document(args.target_root, args.document_class, args.name,
                                   args.profile_root, template_id=args.template_id, layer=args.layer)
        elif args.operation == "create-document":
            payload = _input(args.input)
            result = create_document(args.target_root, args.document_class, args.name,
                                     payload.get("metadata"), payload.get("sections"), args.profile_root,
                                     template_id=args.template_id, layer=args.layer, expected_contract_sha256=args.expected_contract_sha256)
        elif args.operation == "update-section":
            payload = _input(args.input)
            result = update_section(args.target_root, args.document_class, args.name,
                                    args.section, payload.get("content"), payload.get("expected_sha256"),
                                    payload.get("fields"), args.profile_root, expected_contract_sha256=args.expected_contract_sha256, layer=args.layer)
        elif args.operation == "attach-figure":
            payload = _input(args.input)
            result = attach_figure(args.target_root, args.document_class, args.name,
                                   args.section, payload.get("figure"), payload.get("expected_sha256"),
                                   args.profile_root, concern=args.concern, layer=args.layer, decision=payload.get("decision"), expected_contract_sha256=args.expected_contract_sha256)
        else:
            if args.name is None and args.layer is not None or args.name is not None and args.document_class is None:
                raise DocumentKitError("invalid-request", "Selected check needs --class and --name; --layer requires --name")
            selection = None if args.name is None else dict(document_class=args.document_class, name=args.name, layer=args.layer)
            result = check_documents(args.target_root, args.document_class, args.profile_root, selection=selection)
    except DocumentKitError as error:
        result = error.as_dict()
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

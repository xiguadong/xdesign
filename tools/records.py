#!/usr/bin/env python3
"""Read maintained project records from the local operational store.

Project plans, reviews, executions, status, and wish-history updates live as
immutable `record` items in `.xdesign` (conventions: docs/design/storage.md#d86).
This helper prints only the small structured part of a record revision, so an
agent does not have to read a whole Markdown tree to learn a record's state.

    python3 tools/records.py list
    python3 tools/records.py show record-status
    python3 tools/records.py show record-plan-storage --revision 1

The archived Markdown bytes stay available through the ordinary store commands:

    xdesign store read ITEM --uri URI --output FILE
    xdesign store checkout ITEM --revision N --destination DIR
"""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[1]
BINARY = next((p for p in (REPO / "cache/target/debug/xdesign", REPO / "target/debug/xdesign") if p.is_file()), REPO / "cache/target/debug/xdesign")


def store(*arguments: str, root: pathlib.Path | None) -> dict:
    proc = subprocess.run(
        [str(BINARY), "store", *arguments, *(["--root", str(root)] if root is not None else [])],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip()
        raise SystemExit(f"store {arguments[0]} failed (exit {proc.returncode}): {detail}")
    return json.loads(proc.stdout)["data"]


def command_list(args) -> None:
    items = store("list", "--kind", "record", root=args.root)["items"]
    rows = [
        {"item_id": item["item_id"], "head_revision": item["head_revision"]}
        for item in items
    ]
    print(json.dumps(rows, indent=1))


def command_show(args) -> None:
    revision_args = ["--revision", str(args.revision)] if args.revision else []
    revision = store("get", args.item, *revision_args, root=args.root)["revision"]
    receipt = store("outcome", revision["operation_id"], root=args.root)["receipt"]
    record = {
        "item_id": revision["item_id"],
        "kind": revision["kind"],
        "revision": revision["revision"],
        "metadata": revision["metadata"],
        "body": receipt.get("result"),
        "materials": [
            {
                "uri": material["uri"],
                "role": material["role"],
                "byte_length": material["byte_length"],
                "sha256": material["sha256"],
            }
            for material in revision["materials"]
        ],
    }
    print(json.dumps(record, indent=1))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument(
        "--root",
        type=pathlib.Path,
        default=None,
        help="explicit store root relative to the repository; default delegates contextual selection to xdesign",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("list", help="list record item heads")
    show = subparsers.add_parser("show", help="print one record revision's compact body")
    show.add_argument("item", help="record item ID, for example record-status")
    show.add_argument("--revision", type=int, help="revision to read (default: head)")
    args = parser.parse_args()
    if args.command == "list":
        command_list(args)
    else:
        command_show(args)
    return 0


if __name__ == "__main__":
    sys.exit(main())

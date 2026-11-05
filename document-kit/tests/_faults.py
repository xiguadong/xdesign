"""Test-only deterministic OS fault installation, never imported by product code."""

from contextlib import contextmanager
from unittest.mock import patch
import errno
import os

from xdesign_doc_kit import core
from xdesign_doc_kit.io_boundary import CURRENT


@contextmanager
def fault(mode):
    original_open, original_replace, original_unlink = os.open, os.replace, os.unlink
    original_publish = core._replace

    def denied():
        return PermissionError(errno.EACCES, "injected test access denial")

    def open_file(path, *args, **kwargs):
        current = CURRENT.get()
        if (
            current
            and mode == current.phase
            and mode
            in (
                "pack-read",
                "model-read",
                "document-read",
                "registry-read",
                "route",
                "lock",
                "staging",
            )
        ):
            raise denied()
        return original_open(path, *args, **kwargs)

    def replace_file(src, dst, *args, **kwargs):
        current = CURRENT.get()
        if current and (
            mode == current.phase
            or mode in ("orphan", "unknown-cleanup")
            and current.phase == "registry-publish"
        ):
            raise denied()
        return original_replace(src, dst, *args, **kwargs)

    def unlink_file(path, *args, **kwargs):
        current = CURRENT.get()
        if (
            current
            and current.phase == "cleanup"
            and mode in ("orphan", "unknown-cleanup")
            and str(path).endswith(".md")
        ):
            raise denied()
        return original_unlink(path, *args, **kwargs)

    def publish(*args, **kwargs):
        if mode == "page-publish" and kwargs.get("create"):
            # Fail the actual no-replace syscall, not prevalidation.
            with patch("os.link", side_effect=denied()):
                return original_publish(*args, **kwargs)
        result = original_publish(*args, **kwargs)
        if mode == "after-publication" and not str(args[1]).endswith(".json"):
            raise OSError(errno.EIO, "injected after publication")
        return result

    original_read = core._read_project_file

    def read(*args, **kwargs):
        current = CURRENT.get()
        if (
            mode == "unknown-cleanup"
            and current
            and current.publication_state == "page-only"
            and str(args[1]).endswith(".md")
        ):
            raise denied()
        return original_read(*args, **kwargs)

    with (
        patch("os.open", open_file),
        patch("os.replace", replace_file),
        patch("os.unlink", unlink_file),
        patch.object(core, "_replace", publish),
        patch.object(core, "_read_project_file", read),
    ):
        yield

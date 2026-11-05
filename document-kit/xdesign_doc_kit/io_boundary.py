"""Operation-local I/O diagnostics; publication is never a two-file transaction."""

from __future__ import annotations

from contextlib import contextmanager
from contextvars import ContextVar
from dataclasses import dataclass

from .compatibility import DocumentKitError


@dataclass
class Operation:
    operation: str
    phase: str = "pack-read"
    path: str | None = None
    publication_state: str = "none"


CURRENT = ContextVar("document_kit_operation", default=None)
LAUNCH_PIN = ContextVar("document_kit_launch_pin", default=None)
LAUNCH_SCHEMA = ContextVar("document_kit_launch_schema", default=None)


def phase(name, path=None):
    current = CURRENT.get()
    if current is not None:
        current.phase, current.path = name, path


def published(state):
    current = CURRENT.get()
    if current is not None:
        current.publication_state = state


def annotate(error):
    current = CURRENT.get()
    if current and error.outcome is None:
        error.outcome = dict(
            operation=current.operation,
            phase=current.phase,
            publication_state=current.publication_state,
            path=current.path,
        )
    return error


@contextmanager
def operation(name):
    current = Operation(name)
    token = CURRENT.set(current)
    try:
        yield current
    except DocumentKitError as error:
        if error.code in (
            "access-denied",
            "atomic-replace-failed",
            "publication-incomplete",
            "publication-unknown",
            "unreadable-input",
        ):
            annotate(error)
        elif current.phase == "pack-read" and error.code in (
            "invalid-profile",
            "invalid-catalog",
        ):
            cause = error
            while cause is not None and not isinstance(cause, OSError):
                cause = cause.__cause__
            if cause is not None:
                annotate(error)
        raise
    except OSError as error:
        code = (
            "publication-unknown"
            if current.publication_state != "none"
            else (
                "unreadable-input"
                if current.phase in ("model-read", "document-read", "registry-read")
                else "access-denied"
            )
        )
        if code == "publication-unknown":
            current.publication_state = "unknown"
        raise annotate(
            DocumentKitError(code, f"I/O failed during {current.phase}: {error}")
        ) from error
    finally:
        CURRENT.reset(token)

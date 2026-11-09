"""Select a historical contract without changing the running implementation.

Schema-specific regression suites keep their original fixtures, error ordering and
assertions. BDK suites exercise the current root independently.
"""
from functools import wraps
import inspect


def select_release(operation, pack):
    signature = inspect.signature(operation)

    @wraps(operation)
    def selected(*args, **kwargs):
        bound = signature.bind_partial(*args, **kwargs)
        if 'profile_root' not in bound.arguments:
            kwargs['profile_root'] = pack
        return operation(*args, **kwargs)

    return selected

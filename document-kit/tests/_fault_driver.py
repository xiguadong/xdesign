"""Process-only test entry point for CLI and real MCP fault parity."""

import sys
from _faults import fault

mode, adapter, *args = sys.argv[1:]
if adapter == "cli":
    from xdesign_doc_kit.__main__ import main

    with fault(mode):
        raise SystemExit(main(args))
else:
    from xdesign_doc_kit import mcp_server

    original = mcp_server._tool_call

    def faulted(*args, **kwargs):
        with fault(mode):
            return original(*args, **kwargs)

    mcp_server._tool_call = faulted
    raise SystemExit(mcp_server.main(args))

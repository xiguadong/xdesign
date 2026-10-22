"""Verify that the built site keeps the real documentation home page."""

from pathlib import Path


def on_post_build(config):
    """Fail the build if docs/index.md was not published as the site home."""
    index = Path(config.site_dir) / "index.html"
    if not index.is_file():
        raise RuntimeError("The built site has no index.html from docs/index.md")
    text = index.read_text(encoding="utf-8", errors="replace").lower()
    if "00_index" in text or "http-equiv=\"refresh\"" in text:
        raise RuntimeError("The site home still contains the retired 00_index redirect")

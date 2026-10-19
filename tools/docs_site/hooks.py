"""Open the numbered documentation entry from the site root."""

from pathlib import Path


def on_post_build(config):
    """Send the site root to the documentation index, preserving anchors."""
    destination = "00_index/"
    html = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>xdesign documentation</title>
  <meta http-equiv="refresh" content="0; url={destination}">
  <script>location.replace({destination!r} + location.search + location.hash);</script>
</head>
<body><p><a href="{destination}">Open the xdesign documentation</a></p></body>
</html>
"""
    (Path(config.site_dir) / "index.html").write_text(html, encoding="utf-8")

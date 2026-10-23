"""Publish the requested README overview alongside the documentation map."""

from pathlib import Path

from mkdocs.structure.files import File, InclusionLevel


def on_files(files, config):
    """Keep README.md links while avoiding its default index.md output collision."""
    files.append(File(
        "README.md", config.docs_dir, config.site_dir, config.use_directory_urls,
        dest_uri="overview/index.html", inclusion=InclusionLevel.INCLUDED,
    ))
    return files


def on_post_build(config):
    """Fail the build if docs/index.md was not published as the site home."""
    index = Path(config.site_dir) / "index.html"
    if not index.is_file():
        raise RuntimeError("The built site has no index.html from docs/index.md")
    text = index.read_text(encoding="utf-8", errors="replace").lower()
    if "00_index" in text or "http-equiv=\"refresh\"" in text:
        raise RuntimeError("The site home still contains the retired 00_index redirect")

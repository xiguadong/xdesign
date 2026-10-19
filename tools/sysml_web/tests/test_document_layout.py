"""Regression checks for the documentation directory convention."""

from pathlib import Path
import tempfile
import unittest

from tools.check_documentation import REPO, inspect


class DocumentLayoutTests(unittest.TestCase):
    def setUp(self):
        temporary_root = REPO / "cache" / "tmp"
        temporary_root.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=temporary_root)
        self.root = Path(self.temporary.name)
        (self.root / "docs" / "example").mkdir(parents=True)
        (self.root / "docs" / "00_index.md").write_text("# Entry\n")
        (self.root / "mbse").mkdir()
        (self.root / "mbse" / "model.sysml").write_text("package Example {}\n")

    def tearDown(self):
        self.temporary.cleanup()

    def test_missing_overview_and_reserved_zero_fail(self):
        (self.root / "docs" / "example" / "00_topic.md").write_text("# Topic\n")
        result = inspect(self.root)
        codes = {finding["code"] for finding in result["diagnostics"]}
        self.assertIn("missing-overview", codes)
        self.assertIn("reserved-overview-index", codes)
        self.assertFalse(result["ok"])

    def test_overview_and_contiguous_topic_pass(self):
        (self.root / "docs" / "example" / "00_overview.md").write_text("# Overview\n")
        (self.root / "docs" / "example" / "01_topic.md").write_text("# Topic\n")
        result = inspect(self.root)
        self.assertTrue(result["rules"]["directory-layout"]["passed"])
        self.assertTrue(result["ok"])

    def test_missing_root_entry_fails(self):
        (self.root / "docs" / "00_index.md").unlink()
        (self.root / "docs" / "example" / "00_overview.md").write_text("# Overview\n")
        result = inspect(self.root)
        self.assertIn("missing-root-entry", {finding["code"] for finding in result["diagnostics"]})
        self.assertFalse(result["ok"])

    def test_figure_in_carrying_document_assets_passes(self):
        (self.root / "docs" / "example" / "00_overview.md").write_text("# Overview\n")
        (self.root / "docs" / "example" / "assets").mkdir()
        (self.root / "docs" / "example" / "assets" / "01_figure.svg").write_text("<svg/>\n")
        (self.root / "docs" / "example" / "01_topic.md").write_text(
            '<a id="d01"></a>\n\n## D01 Topic\n\n'
            '![Model figure](assets/01_figure.svg)\n\n'
            '> **Figure D01-1 — Model figure.**\n'
            '> **Question:** What is shown?\n'
            '> **Model view:** Example::view.\n'
            '> **Companions:** D01.\n'
            '> **Status:** renderable-now.\n'
            '> **Reading:** The figure shows the example.\n'
            '> **Limits:** No engineering acceptance is implied.\n'
        )
        (self.root / "mbse" / "model.sysml").write_text(
            'package Example {\n'
            '    @XDesignCommon::DocSection {\n'
            '        document = "docs/example/01_topic.md";\n'
            '        section = "D01";\n'
            '    }\n'
            '    @XDesignCommon::DocFigure {\n'
            '        document = "docs/example/01_topic.md";\n'
            '        figure = "docs/example/assets/01_figure.svg";\n'
            '    }\n'
            '}\n'
        )
        result = inspect(self.root)
        self.assertTrue(result["ok"], result["diagnostics"])
        self.assertEqual(result["counts"]["documentation_directories"], 1)

    def test_legacy_central_assets_location_fails(self):
        (self.root / "docs" / "example" / "00_overview.md").write_text("# Overview\n")
        (self.root / "docs" / "example" / "01_topic.md").write_text(
            '<a id="d01"></a>\n\n## D01 Topic\n\n'
            '![Model figure](../assets/example/01_figure.svg)\n'
        )
        old_assets = self.root / "docs" / "assets" / "example"
        old_assets.mkdir(parents=True)
        (old_assets / "01_figure.svg").write_text("<svg/>\n")
        (self.root / "mbse" / "model.sysml").write_text(
            'package Example {\n'
            '    @XDesignCommon::DocFigure {\n'
            '        document = "docs/example/01_topic.md";\n'
            '        figure = "docs/assets/example/01_figure.svg";\n'
            '    }\n'
            '}\n'
        )
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertIn("image-directory-mismatch", codes)
        self.assertIn("figure-directory-mismatch", codes)

    def test_nested_asset_directory_does_not_require_overview(self):
        (self.root / "docs" / "example" / "00_overview.md").write_text("# Overview\n")
        nested_assets = self.root / "docs" / "example" / "assets" / "exports"
        nested_assets.mkdir(parents=True)
        (nested_assets / "notes.md").write_text("# Unexpected Markdown\n")
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertIn("markdown-in-assets", codes)
        self.assertNotIn("missing-overview", codes)


if __name__ == "__main__":
    unittest.main()

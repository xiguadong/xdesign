"""Regression checks for the documentation class, naming, locator, and figure rules."""

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
        (self.root / "docs" / "architecture").mkdir(parents=True)
        (self.root / "docs" / "index.md").write_text("# Documentation\n")
        (self.root / "mbse").mkdir()
        (self.root / "mbse" / "model.sysml").write_text("package Example {}\n")

    def tearDown(self):
        self.temporary.cleanup()

    def good_page(self, name="topic.md"):
        (self.root / "docs" / "architecture" / "assets").mkdir(parents=True, exist_ok=True)
        (self.root / "docs" / "architecture" / "assets" / "figure.svg").write_text("<svg/>\n")
        (self.root / "docs" / "architecture" / name).write_text(
            '<a id="d01"></a>\n\n## Topic\n\n'
            '![Model figure](assets/figure.svg)\n\n'
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
            '        document = "docs/architecture/topic.md";\n'
            '        section = "D01";\n'
            '    }\n'
            '    @XDesignCommon::DocFigure {\n'
            '        document = "docs/architecture/topic.md";\n'
            '        figure = "docs/architecture/assets/figure.svg";\n'
            '    }\n'
            '}\n'
        )

    def test_valid_class_page_and_figure_pass(self):
        self.good_page()
        result = inspect(self.root)
        self.assertTrue(result["ok"], result["diagnostics"])
        self.assertEqual(result["counts"]["DocSection"], 1)
        self.assertEqual(result["counts"]["DocFigure"], 1)
        self.assertEqual(result["counts"]["images"], 1)

    def test_missing_root_entry_fails(self):
        (self.root / "docs" / "index.md").unlink()
        result = inspect(self.root)
        self.assertIn("missing-root-entry", {finding["code"] for finding in result["diagnostics"]})
        self.assertFalse(result["ok"])

    def test_root_readme_can_reuse_canonical_figure(self):
        self.good_page()
        (self.root / "docs" / "README.md").write_text(
            '# Project overview\n\n'
            '![Model figure](architecture/assets/figure.svg)\n\n'
            'See the [owning design](architecture/topic.md#d01).\n'
        )
        result = inspect(self.root)
        self.assertTrue(result["ok"], result["diagnostics"])
        self.assertEqual(result["counts"]["overview_figure_reuses"], 1)
        self.assertEqual(result["counts"]["figure_blocks"], 1)

    def test_root_readme_cannot_reuse_unowned_figure(self):
        self.good_page()
        (self.root / "mbse" / "model.sysml").write_text("package Example {}\n")
        (self.root / "docs" / "README.md").write_text(
            '# Project overview\n\n![Model figure](architecture/assets/figure.svg)\n'
        )
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertEqual(codes, {"unowned-overview-image"})

    def test_nested_readme_does_not_get_overview_exception(self):
        self.good_page()
        (self.root / "docs" / "architecture" / "README.md").write_text(
            '# Another overview\n\n![Model figure](assets/figure.svg)\n'
        )
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertIn("invalid-document-name", codes)
        self.assertIn("missing-figure-block", codes)

    def test_other_root_page_does_not_get_overview_exception(self):
        (self.root / "docs" / "overview.md").write_text("# Another overview\n")
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertIn("invalid-document-directory", codes)

    def test_numeric_filename_is_not_allowed(self):
        self.good_page("01_topic.md")
        result = inspect(self.root)
        self.assertIn("invalid-document-name", {finding["code"] for finding in result["diagnostics"]})
        self.assertFalse(result["ok"])

    def test_non_class_directory_is_not_allowed(self):
        (self.root / "docs" / "notes").mkdir()
        (self.root / "docs" / "notes" / "topic.md").write_text("# Topic\n")
        result = inspect(self.root)
        self.assertIn("invalid-document-directory", {finding["code"] for finding in result["diagnostics"]})

    def test_central_assets_are_not_allowed(self):
        (self.root / "docs" / "architecture" / "topic.md").write_text(
            '<a id="d01"></a>\n\n## Topic\n\n'
            '![Model figure](../assets/example/figure.svg)\n'
        )
        old_assets = self.root / "docs" / "assets" / "example"
        old_assets.mkdir(parents=True)
        (old_assets / "figure.svg").write_text("<svg/>\n")
        (self.root / "mbse" / "model.sysml").write_text(
            'package Example {\n'
            '    @XDesignCommon::DocFigure {\n'
            '        document = "docs/architecture/topic.md";\n'
            '        figure = "docs/assets/example/figure.svg";\n'
            '    }\n'
            '}\n'
        )
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertIn("image-directory-mismatch", codes)
        self.assertIn("figure-directory-mismatch", codes)

    def test_markdown_in_assets_fails_without_requiring_overviews(self):
        self.root.joinpath("docs", "architecture", "assets").mkdir(parents=True)
        self.root.joinpath("docs", "architecture", "assets", "notes.md").write_text("# Unexpected\n")
        codes = {finding["code"] for finding in inspect(self.root)["diagnostics"]}
        self.assertIn("markdown-in-assets", codes)
        self.assertNotIn("missing-overview", codes)

    def test_anchor_without_heading_fails(self):
        (self.root / "docs" / "architecture" / "topic.md").write_text('<a id="d01"></a>\n\nText only.\n')
        result = inspect(self.root)
        self.assertIn("missing-section-heading", {finding["code"] for finding in result["diagnostics"]})


if __name__ == "__main__":
    unittest.main()

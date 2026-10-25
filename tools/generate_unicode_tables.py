#!/usr/bin/env python3
"""Generate the static Unicode tables used by the storage manifest-name rules.

The storage layer decides whether two material URIs name the same filesystem
entry. It must do so identically on every host, so the implementation owns its
Unicode data instead of relying on the host filesystem or a platform library.
The equivalence implemented in ``src/storage/unicode.rs`` is:

    fold(path) = NFD(casefold(NFD(path)))

i.e. canonical decomposition, full Unicode case folding, and canonical
decomposition/canonical ordering again. This script derives the tables for that
function from the Unicode Character Database bundled with Python's standard
library (``unicodedata``), so the committed Rust tables have no runtime Python
or data-file dependency.

Usage::

    python3 tools/generate_unicode_tables.py [output-path]

The default output path is ``src/storage/unicode.rs``.
"""

from __future__ import annotations

import os
import sys
import unicodedata

# The provenance recorded in the generated file. Update only together with a
# deliberate Unicode version change and regenerate the tables.
UNICODE_VERSION = "13.0.0"

# Hangul syllable decomposition is algorithmic (Unicode 3.12, section 3.12);
# keeping its 11,172 syllables out of the table keeps the generated source small.
S_BASE = 0xAC00
L_BASE = 0x1100
V_BASE = 0x1161
T_BASE = 0x11A7
L_COUNT = 19
V_COUNT = 21
T_COUNT = 28
N_COUNT = V_COUNT * T_COUNT
S_COUNT = L_COUNT * N_COUNT

SURROGATES = range(0xD800, 0xE000)
HANGUL_SYLLABLES = range(S_BASE, S_BASE + S_COUNT)


def rust_string(text: str) -> str:
    """Render *text* as a Rust string literal with explicit escapes."""
    rendered = []
    for char in text:
        code_point = ord(char)
        if char == "\\":
            rendered.append("\\\\")
        elif char == '"':
            rendered.append('\\"')
        elif 0x20 <= code_point <= 0x7E:
            rendered.append(char)
        else:
            rendered.append("\\u{%X}" % code_point)
    return '"' + "".join(rendered) + '"'


def canonical_decomposition(code_point: int) -> str:
    return unicodedata.normalize("NFD", chr(code_point))


def collect_tables():
    decompositions = []
    combining_classes = []
    case_foldings = []
    for code_point in range(0x110000):
        if code_point in SURROGATES:
            continue
        char = chr(code_point)
        if code_point not in HANGUL_SYLLABLES:
            decomposed = canonical_decomposition(code_point)
            if decomposed != char:
                decompositions.append((code_point, decomposed))
        combining_class = unicodedata.combining(char)
        if combining_class != 0:
            combining_classes.append((code_point, combining_class))
        if code_point not in HANGUL_SYLLABLES:
            # Folding is applied after canonical decomposition, so only
            # NFD-stable code points need a case-folding entry.
            if canonical_decomposition(code_point) == char:
                folded = unicodedata.normalize("NFD", char.casefold())
                if folded != char:
                    case_foldings.append((code_point, folded))
    return decompositions, combining_classes, case_foldings


def render_table(
    name: str,
    doc: str,
    entries,
    render_entry,
    chunk_size: int = 6,
) -> str:
    lines = [
        f"/// {doc}",
        "#[rustfmt::skip]",
        f"static {name}: &[(u32, {render_entry[1]})] = &[",
    ]
    for start in range(0, len(entries), chunk_size):
        chunk = entries[start : start + chunk_size]
        rendered = ", ".join(
            f"({render_entry[0](entry[0])}, {render_entry[2](entry[1])})"
            for entry in chunk
        )
        lines.append(f"    {rendered},")
    lines.append("];")
    return "\n".join(lines)


def code_point_literal(code_point: int) -> str:
    return f"0x{code_point:04X}"


def rust_u8(value: int) -> str:
    return str(value)


def verify(decompositions, combining_classes, case_foldings) -> None:
    """Verify the generated tables reproduce the intended NFD(casefold(NFD()))."""
    if unicodedata.unidata_version != UNICODE_VERSION:
        raise SystemExit(
            "unicodedata reports Unicode %s, expected %s; refusing to generate "
            "tables with different provenance"
            % (unicodedata.unidata_version, UNICODE_VERSION)
        )
    decomposition_map = dict(decompositions)
    combining_map = dict(combining_classes)
    folding_map = dict(case_foldings)

    def fold(value: str) -> str:
        decomposed = []
        for char in value:
            code_point = ord(char)
            if code_point in HANGUL_SYLLABLES:
                index = code_point - S_BASE
                decomposed.append(chr(L_BASE + index // N_COUNT))
                decomposed.append(chr(V_BASE + (index % N_COUNT) // T_COUNT))
                if index % T_COUNT:
                    decomposed.append(chr(T_BASE + index % T_COUNT))
            else:
                decomposed.extend(decomposition_map.get(code_point, char))
        folded = []
        for char in decomposed:
            folded.extend(folding_map.get(ord(char), char))
        # Canonical ordering (stable insertion sort over combining classes).
        for index in range(1, len(folded)):
            combining_class = combining_map.get(ord(folded[index]), 0)
            if combining_class == 0:
                continue
            position = index
            while position > 0:
                previous = combining_map.get(ord(folded[position - 1]), 0)
                if previous == 0 or previous <= combining_class:
                    break
                folded[position - 1], folded[position] = (
                    folded[position],
                    folded[position - 1],
                )
                position -= 1
        return "".join(folded)

    for code_point in range(0x110000):
        if code_point in SURROGATES:
            continue
        char = chr(code_point)
        expected = unicodedata.normalize("NFD", char.casefold())
        actual = fold(char)
        if actual != expected:
            raise SystemExit(
                "fold mismatch at U+%04X: table produced %r, expected %r"
                % (code_point, actual, expected)
            )
    samples = [
        "\u00e9.txt",  # é.txt
        "e\u0301.txt",  # e + combining acute
        "\u03a3.txt",  # Σ.txt
        "\u03c2.txt",  # ς.txt
        "\u0130",  # İ
        "\u1e9e",  # ẞ
        "\u01c4",  # Ǆ
        "\u1e9b",  # ẛ
        "\ud55c\uad6d",  # 한국
        "\ufb01le",  # ﬁle
    ]
    for sample in samples:
        expected = unicodedata.normalize("NFD", sample.casefold())
        if fold(sample) != expected:
            raise SystemExit(
                "fold mismatch for sample %r: %r != %r"
                % (sample, fold(sample), expected)
            )
        if fold(fold(sample)) != fold(sample):
            raise SystemExit("fold is not idempotent for sample %r" % sample)


def main(argv) -> int:
    output_path = (
        argv[1] if len(argv) > 1 else os.path.join("src", "storage", "unicode.rs")
    )
    decompositions, combining_classes, case_foldings = collect_tables()
    verify(decompositions, combining_classes, case_foldings)

    parts = [
        "//! Static Unicode tables for host-independent storage manifest names.",
        "//!",
        f"//! Generated by `tools/generate_unicode_tables.py` from Unicode Character",
        f"//! Database version {UNICODE_VERSION} as bundled with Python 3.9",
        "//! (`unicodedata.unidata_version`). Do not edit this file by hand; rerun",
        "//! the generator instead:",
        "//!",
        "//! ```text",
        "//! python3 tools/generate_unicode_tables.py src/storage/unicode.rs",
        "//! ```",
        "//!",
        "//! The storage layer treats two path entries as the same directory entry",
        "//! after canonical decomposition (NFD, including the algorithmic Hangul",
        "//! decomposition), Unicode full case folding, and canonical ordering. The",
        "//! tables below supply those mappings without runtime data files, Python, or",
        "//! any platform Unicode library.",
        "",
        "#[allow(dead_code)] // Provenance marker; the tables are the data that matter.",
        f'pub(crate) const UNICODE_VERSION: &str = "{UNICODE_VERSION}";',
        "",
        "const S_BASE: u32 = 0xAC00;",
        "const L_BASE: u32 = 0x1100;",
        "const V_BASE: u32 = 0x1161;",
        "const T_BASE: u32 = 0x11A7;",
        "const V_COUNT: u32 = 21;",
        "const T_COUNT: u32 = 28;",
        "const N_COUNT: u32 = V_COUNT * T_COUNT;",
        "const S_COUNT: u32 = 19 * N_COUNT;",
        "",
        render_table(
            "CANONICAL_DECOMPOSITION",
            "Full canonical decompositions (NFD expansion) by code point.",
            decompositions,
            (code_point_literal, "&'static str", rust_string),
        ),
        "",
        render_table(
            "CANONICAL_COMBINING_CLASS",
            "Canonical combining classes (zero classes are omitted).",
            combining_classes,
            (code_point_literal, "u8", rust_u8),
        ),
        "",
        render_table(
            "CASE_FOLDING",
            "Full case folding for NFD-stable code points, expanded to NFD.",
            case_foldings,
            (code_point_literal, "&'static str", rust_string),
        ),
        "",
        RUN_TIME.lstrip("\n"),
    ]
    rendered = "\n".join(parts)
    with open(output_path, "w", encoding="utf-8") as output:
        output.write(rendered)
    print(
        "wrote %s: %d decomposition, %d combining-class, %d case-folding entries, %d bytes"
        % (
            output_path,
            len(decompositions),
            len(combining_classes),
            len(case_foldings),
            len(rendered.encode("utf-8")),
        ),
        file=sys.stderr,
    )
    return 0


RUN_TIME = r'''
fn lookup(table: &'static [(u32, &'static str)], code_point: u32) -> Option<&'static str> {
    table
        .binary_search_by_key(&code_point, |&(key, _)| key)
        .ok()
        .map(|index| table[index].1)
}

/// Returns the canonical combining class of *value*, or 0 for a starter.
fn combining_class(value: char) -> u8 {
    CANONICAL_COMBINING_CLASS
        .binary_search_by_key(&(value as u32), |&(key, _)| key)
        .ok()
        .map_or(0, |index| CANONICAL_COMBINING_CLASS[index].1)
}

fn decompose(value: char, output: &mut Vec<char>) {
    let code_point = value as u32;
    if (S_BASE..S_BASE + S_COUNT).contains(&code_point) {
        let index = code_point - S_BASE;
        output.push(
            char::from_u32(L_BASE + index / N_COUNT)
                .expect("Hangul leading jamo are valid scalar values"),
        );
        output.push(
            char::from_u32(V_BASE + (index % N_COUNT) / T_COUNT)
                .expect("Hangul vowel jamo are valid scalar values"),
        );
        let trailing = index % T_COUNT;
        if trailing != 0 {
            output.push(
                char::from_u32(T_BASE + trailing)
                    .expect("Hangul trailing jamo are valid scalar values"),
            );
        }
        return;
    }
    match lookup(CANONICAL_DECOMPOSITION, code_point) {
        Some(expansion) => output.extend(expansion.chars()),
        None => output.push(value),
    }
}

/// Canonical ordering: stable-sort each run of combining marks by combining
/// class without moving marks across starters.
fn canonical_order(chars: &mut [char]) {
    for index in 1..chars.len() {
        let class = combining_class(chars[index]);
        if class == 0 {
            continue;
        }
        let mut position = index;
        while position > 0 {
            let previous = combining_class(chars[position - 1]);
            if previous == 0 || previous <= class {
                break;
            }
            chars.swap(position - 1, position);
            position -= 1;
        }
    }
}

/// Host-independent path equivalence: canonical decomposition (NFD), full
/// Unicode case folding, and canonical ordering again. This is a conservative
/// (superset) approximation of filesystem name equivalence: any two paths that
/// fold together are treated as one directory entry, including a few pairs no
/// host filesystem would merge.
pub(crate) fn fold(value: &str) -> String {
    let mut decomposed = Vec::with_capacity(value.len());
    for character in value.chars() {
        decompose(character, &mut decomposed);
    }
    let mut folded = Vec::with_capacity(decomposed.len());
    for character in decomposed {
        match lookup(CASE_FOLDING, character as u32) {
            Some(expansion) => folded.extend(expansion.chars()),
            None => folded.push(character),
        }
    }
    canonical_order(&mut folded);
    folded.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::fold;

    fn assert_folds_equal(left: &str, right: &str) {
        assert_eq!(fold(left), fold(right), "{left:?} vs {right:?}");
    }

    #[test]
    fn canonical_and_case_equivalents_fold_together() {
        assert_folds_equal("\u{e9}.txt", "e\u{301}.txt");
        assert_folds_equal("\u{3a3}.txt", "\u{3c2}.txt");
        assert_folds_equal("File.txt", "file.txt");
        assert_folds_equal("\u{c5}.txt", "\u{212b}.txt");
        assert_folds_equal("\u{1e9e}", "ss");
        assert_folds_equal(
            "\u{d55c}\u{ad6d}",
            "\u{1112}\u{1161}\u{11ab}\u{1100}\u{116e}\u{11a8}",
        );
        assert_folds_equal("e\u{0301}\u{0323}", "e\u{0323}\u{0301}");
    }

    #[test]
    fn distinct_names_fold_apart() {
        assert_ne!(fold("\u{e9}.txt"), fold("e.txt"));
        assert_ne!(fold("\u{3c2}"), fold("\u{3c3}.txt"));
    }

    /// Expected forms computed with Python's `unicodedata` (Unicode 13.0.0):
    /// `unicodedata.normalize("NFD", value.casefold())`. This cross-checks the
    /// Rust tables and runtime against the Unicode data the generator read.
    #[test]
    fn tricky_folds_match_unicode_data() {
        for (value, expected) in [
            ("e\u{0323}\u{0301}", "e\u{0323}\u{0301}"),
            ("\u{1e9b}", "s\u{0307}"),
            ("\u{fb01}le", "file"),
            ("\u{d55c}", "\u{1112}\u{1161}\u{11ab}"),
            ("\u{0130}", "i\u{0307}"),
            ("\u{1e9e}", "ss"),
            ("\u{00df}", "ss"),
            ("\u{01c4}", "\u{01c6}"),
            ("\u{01c5}", "\u{01c6}"),
            ("\u{1ec7}", "e\u{0323}\u{0302}"),
            ("\u{0345}", "\u{03b9}"),
            ("\u{00c5}", "a\u{030a}"),
            ("\u{212b}", "a\u{030a}"),
        ] {
            assert_eq!(fold(value), expected, "{value:?}");
        }
    }

    #[test]
    fn folding_is_idempotent() {
        for value in ["", "a/b", "\u{3a3}.txt", "\u{1e9b}", "e\u{0301}"] {
            let folded = fold(value);
            assert_eq!(fold(&folded), folded, "{value:?}");
        }
    }

    #[test]
    fn ascii_is_preserved_except_case() {
        assert_eq!(fold("Dir/File-01.txt"), "dir/file-01.txt");
    }
}
'''


if __name__ == "__main__":
    sys.exit(main(sys.argv))

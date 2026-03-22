# osc66

A CLI filter that wraps text in [OSC 66 / kitty text-sizing-protocol](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/) escape codes, using HarfBuzz for accurate glyph shaping and cell-width calculation.

## The Problem

Terminal emulators measure character widths using Unicode's `EastAsianWidth` property. This works for Latin and CJK, but fails for complex scripts like Malayalam, Arabic, and Devanagari — where shaping transforms multiple codepoints into a single glyph with a different visual width.

Example: the Malayalam conjunct **ക്ഷ** is 3 codepoints, but HarfBuzz shapes it into one glyph that needs 2 cells. A terminal using `wcwidth()` sees 3 codepoints, allocates 3 cells, and the glyph either overflows or gets clipped.

```
wcwidth("ക്ഷ") → 3    # wrong: 3 codepoints, each width 1
harfbuzz       → 2    # correct: one shaped glyph, 2 cells wide
```

## The Solution

`osc66` shapes input text with HarfBuzz, groups glyphs into clusters, computes each cluster's real advance width in cells, and emits an OSC 66 escape sequence per cluster carrying that width. Kitty reads the `w=` parameter and allocates exactly the right number of cells.

```
stdin → shape with HarfBuzz → cluster grouping → cell width calc → OSC 66 → stdout
```

Each cluster becomes:

```
ESC ] 66 ; w=<cells> ; <utf8-text> BEL
```

## Installation

Requires: Rust, libharfbuzz, libfontconfig.

```sh
cargo build --release
# binary: target/release/osc66
```

## Usage

```sh
echo "സന്തോഷ്" | osc66 Manjari
```

- **Argument 1 (optional):** font family name, as recognised by fontconfig. Defaults to `monospace`.
- **stdin:** UTF-8 text, any number of lines.
- **stdout:** same text with each grapheme cluster wrapped in an OSC 66 escape sequence.

```sh
# Pipe a file
cat sample.txt | osc66 Manjari

# Works with any script HarfBuzz supports
echo "नमस्ते" | osc66 "Noto Sans Devanagari"

# Use default monospace font for Latin
echo "hello" | osc66
```

## How It Works

1. **Font loading** — fontconfig locates the font file by family name; HarfBuzz loads a `Face` and `Font` from it.
2. **Reference advance** — shapes the ASCII `'0'` to get its `x_advance`. This is the baseline: 1 cell = that many font units.
3. **Shaping** — each input line is shaped as a single HarfBuzz buffer to preserve cross-glyph context (ligatures, contextual forms).
4. **Cluster grouping** — glyphs are grouped by their `cluster` field (a byte offset into the input). The `x_advance` values within each group are summed.
5. **Cell count** — `cells = ceil(cluster_advance / ref_advance)`, clamped to 0–7 (the protocol's 3-bit `w` field).
6. **Emission** — each cluster is written as `ESC]66;w=<cells>;<text>BEL`. Zero-width clusters (virama, ZWJ, etc.) are skipped.

## Dependencies

- [`harfbuzz_rs`](https://crates.io/crates/harfbuzz_rs) 2.0.1 — HarfBuzz bindings
- [`fontconfig-rs`](https://crates.io/crates/fontconfig) 0.1.1 — fontconfig bindings

## License

MIT. See [LICENSE](LICENSE).

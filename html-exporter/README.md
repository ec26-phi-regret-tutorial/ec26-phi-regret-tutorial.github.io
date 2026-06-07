# Notes HTML Exporter

This is a small, dependency-free Rust exporter for the Typst-flavored notes in this repository. It does not use Typst's experimental HTML backend; instead it parses the note conventions used here and renders a static lecture-page shell inspired by the Sum-of-Squares public notes.

The exporter translates the Typst-flavored math used in these notes into KaTeX-compatible TeX, resolves local labels/references, renders theorem-like blocks, handles simple citations from `meta/refs.bib`, and compiles standalone Typst figures from `figures/` into SVGs for the generated HTML.

## Usage

```sh
cargo run --manifest-path html-exporter/Cargo.toml -- \
  P4-multicalibration.typ \
  public/P4-multicalibration.html \
  --root . \
  --title "Multicalibration" \
  --pdf "../P4-multicalibration.pdf"
```

Useful options:

- `--root <dir>`: set the Typst project root.
- `--site-title <title>`: change the header title.
- `--authors <text>`: change the author line.
- `--index <href>` and `--pdf <href>`: add header links.

The generated HTML is self-contained except for KaTeX, which is loaded from a CDN when needed by the browser.

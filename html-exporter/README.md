# Notes HTML Exporter

This Rust binary exports the notes by compiling Typst with its experimental HTML backend and then applying a light postprocessing pass for the public lecture layout.

The Typst side lives primarily in `meta/gabri_notes_html.typ`, which emits HTML-friendly classes and attributes for the exporter. The Rust side calls Typst as a library, reads the resulting HTML, and handles the page shell, chapter rail, citation and footnote sidenotes, equation sizing hooks, bibliography cleanup, and optional KaTeX conversion.

## Usage

The canonical entry point is the repository Makefile:

```sh
make html
```

That regenerates `public/P*.html` with the notes CSS embedded in each chapter and builds matching PDFs in `public/pdf/`.

For a single chapter:

```sh
cargo run --manifest-path html-exporter/Cargo.toml -- \
  --root . \
  --math katex \
  P4-multicalibration.typ \
  public/P4-multicalibration.html
```

Useful options:

- `--root <dir>`: set the Typst project root.
- `--math <svg|katex>`: choose the math backend. The default is `katex`.
- `--site-title <title>`: change the header title.
- `--authors <text>`: change the author line.
- `--index <href>` and `--pdf <href>`: add header links.

## Checks

```sh
make check-html
```

This runs the Rust tests, regenerates the public HTML and PDFs, and fails if the generated files differ from the committed files.

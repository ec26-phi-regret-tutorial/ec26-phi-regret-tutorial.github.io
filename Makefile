EXPORT_CONFIG := html-export.yaml
CHAPTERS := $(basename $(shell sed -n 's/^[[:space:]]*source:[[:space:]]*//p' $(EXPORT_CONFIG)))
SOURCES := $(addsuffix .typ,$(CHAPTERS))
PUBLIC_HTML := $(addprefix docs/,$(addsuffix .html,$(CHAPTERS)))
PUBLIC_PDF := $(addprefix docs/pdf/,$(addsuffix .pdf,$(CHAPTERS)))
FIGURE_SOURCES := $(wildcard figures/*.typ)
PUBLIC_FIGURES := $(patsubst figures/%.typ,docs/figures/%.svg,$(FIGURE_SOURCES))
EXPORTER := cargo run --release --manifest-path html-exporter/Cargo.toml --
EXPORT_FLAGS := --root . --math katex --config $(EXPORT_CONFIG)
TYPST := typst

.PHONY: html pdf test-html check-html clean clean-html

html: pdf $(PUBLIC_FIGURES)
	@set -e; \
	for src in $(SOURCES); do \
		out="docs/$${src%.typ}.html"; \
		pdf="pdf/$${src%.typ}.pdf"; \
		echo "Exporting $$src -> $$out"; \
		$(EXPORTER) $(EXPORT_FLAGS) --pdf "$$pdf" "$$src" "$$out"; \
	done

pdf: $(PUBLIC_PDF)

docs/pdf/%.pdf: %.typ
	@mkdir -p docs/pdf
	$(TYPST) compile --root . $< $@

docs/figures/%.svg: figures/%.typ
	@mkdir -p docs/figures
	$(TYPST) compile --root . $< $@

test-html:
	cargo test --release --manifest-path html-exporter/Cargo.toml

check-html: test-html html
	git diff --exit-code -- $(PUBLIC_HTML) $(PUBLIC_PDF)

clean:
	rm -rf docs/P* docs/figures docs/pdf

clean-html:
	rm -f $(PUBLIC_HTML) $(PUBLIC_PDF)

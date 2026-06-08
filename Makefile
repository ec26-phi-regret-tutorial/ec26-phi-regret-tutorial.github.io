EXPORT_CONFIG := html-export.yaml
CHAPTERS := $(basename $(shell sed -n 's/^[[:space:]]*source:[[:space:]]*//p' $(EXPORT_CONFIG)))
SOURCES := $(addsuffix .typ,$(CHAPTERS))
PUBLIC_HTML := $(addprefix public/,$(addsuffix .html,$(CHAPTERS)))
PUBLIC_PDF := $(addprefix public/pdf/,$(addsuffix .pdf,$(CHAPTERS)))
FIGURE_SOURCES := $(wildcard figures/*.typ)
PUBLIC_FIGURES := $(patsubst figures/%.typ,public/figures/%.svg,$(FIGURE_SOURCES))
EXPORTER := cargo run --release --manifest-path html-exporter/Cargo.toml --
EXPORT_FLAGS := --root . --math katex --config $(EXPORT_CONFIG)
TYPST := typst

.PHONY: html pdf test-html check-html clean-html

html: pdf $(PUBLIC_FIGURES)
	@set -e; \
	for src in $(SOURCES); do \
		out="public/$${src%.typ}.html"; \
		pdf="pdf/$${src%.typ}.pdf"; \
		echo "Exporting $$src -> $$out"; \
		$(EXPORTER) $(EXPORT_FLAGS) --pdf "$$pdf" "$$src" "$$out"; \
	done

pdf: $(PUBLIC_PDF)

public/pdf/%.pdf: %.typ
	@mkdir -p public/pdf
	$(TYPST) compile --root . $< $@

public/figures/%.svg: figures/%.typ
	@mkdir -p public/figures
	$(TYPST) compile --root . $< $@

test-html:
	cargo test --release --manifest-path html-exporter/Cargo.toml

check-html: test-html html
	git diff --exit-code -- $(PUBLIC_HTML) $(PUBLIC_PDF)

clean-html:
	rm -f $(PUBLIC_HTML) $(PUBLIC_PDF)

CHAPTERS := P1-introduction P2-semi_separation P3-phi-regret-learning P4-multicalibration P5-treeswap P6-profile
SOURCES := $(addsuffix .typ,$(CHAPTERS))
PUBLIC_HTML := $(addprefix public/,$(addsuffix .html,$(CHAPTERS)))
PUBLIC_PDF := $(addprefix public/pdf/,$(addsuffix .pdf,$(CHAPTERS)))
EXPORTER := cargo run --release --manifest-path html-exporter/Cargo.toml --
EXPORT_FLAGS := --root . --math katex
TYPST := typst

.PHONY: html pdf test-html check-html clean-html

html: pdf
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

test-html:
	cargo test --manifest-path html-exporter/Cargo.toml

check-html: test-html html
	git diff --exit-code -- $(PUBLIC_HTML) $(PUBLIC_PDF)

clean-html:
	rm -f $(PUBLIC_HTML) $(PUBLIC_PDF)

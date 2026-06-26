INSTALL_DIR := $(HOME)/.local/bin

.PHONY: compile test install clean

compile:
	@true

test:
	@cargo fmt --check
	@cargo build --quiet
	@for f in bin/*; do \
		[ -f "$$f" ] || continue; \
		bash -n "$$f" && echo "✓ $$f" || exit 1; \
	done

install:
	@mkdir -p $(INSTALL_DIR)
	@cargo build --quiet --release --bin doc-pointers
	@install -m 755 target/release/doc-pointers "$(INSTALL_DIR)/doc-pointers"
	@echo "✓ doc-pointers → $(INSTALL_DIR)/"
	@for f in bin/*; do \
		[ "$$(basename "$$f")" = "doc-pointers" ] && continue; \
		install -m 755 "$$f" "$(INSTALL_DIR)/$$(basename $$f)"; \
		echo "✓ $$(basename $$f) → $(INSTALL_DIR)/"; \
	done

clean:
	@true

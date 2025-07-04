# UltraFast MCP Makefile
# Release automation and development commands

.PHONY: help build test clean release-prepare release-tag release-push release-create

# Default target
help:
	@echo "UltraFast MCP - Available Commands:"
	@echo ""
	@echo "Development:"
	@echo "  build          - Build the entire workspace"
	@echo "  test           - Run all tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  check          - Check code without building"
	@echo "  format         - Format code with rustfmt"
	@echo "  clippy         - Run clippy linter"
	@echo ""
	@echo "Release Process:"
	@echo "  release-prepare VERSION=x.y.z - Prepare release (update versions)"
	@echo "  release-tag VERSION=x.y.z     - Create and push git tag"
	@echo "  release-push                  - Push changes to main branch"
	@echo "  release-create VERSION=x.y.z  - Create GitHub release"
	@echo "  release VERSION=x.y.z         - Full release process"
	@echo ""
	@echo "Examples:"
	@echo "  make release VERSION=202506018.1.0-rc.1.3"
	@echo "  make build"
	@echo "  make test"

# Development commands
build:
	@echo "🔨 Building UltraFast MCP workspace..."
	cargo build --workspace

test:
	@echo "🧪 Running tests..."
	cargo test --workspace

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

check:
	@echo "🔍 Checking code..."
	cargo check --workspace

format:
	@echo "📝 Formatting code..."
	cargo fmt --all

clippy:
	@echo "🔍 Running clippy..."
	cargo clippy --workspace --all-targets --all-features

# Release commands
release-prepare:
	@if [ -z "$(VERSION)" ]; then \
		echo "❌ Error: VERSION is required. Use: make release-prepare VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "🚀 Preparing release $(VERSION)..."
	@echo "📝 Updating workspace version..."
	@sed -i '' 's/version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@echo "📝 Updating README version..."
	@sed -i '' 's/Release Candidate .* (v.*)/Release Candidate $(shell echo $(VERSION) | sed "s/.*rc\.//") (v$(VERSION))/' README.md
	@sed -i '' 's/version = ".*", features = \[/version = "$(VERSION)", features = [/' README.md
	@echo "📝 Updating internal dependencies..."
	@find crates/ -name "Cargo.toml" -exec sed -i '' 's/version = "202506018\.[0-9]*\.[0-9]*-rc\.[0-9]*\.[0-9]*"/version = "$(VERSION)"/g' {} \;
	@echo "✅ Release preparation complete!"

release-tag:
	@if [ -z "$(VERSION)" ]; then \
		echo "❌ Error: VERSION is required. Use: make release-tag VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "🏷️  Creating git tag v$(VERSION)..."
	git tag v$(VERSION)
	@echo "📤 Pushing tag to remote..."
	git push origin v$(VERSION)
	@echo "✅ Tag v$(VERSION) created and pushed!"

release-push:
	@echo "📤 Pushing changes to main branch..."
	git add .
	git commit -m "Release v$(shell grep 'version = ' Cargo.toml | head -1 | sed 's/.*version = "\(.*\)".*/\1/')"
	git push origin main
	@echo "✅ Changes pushed to main!"

release-create:
	@if [ -z "$(VERSION)" ]; then \
		echo "❌ Error: VERSION is required. Use: make release-create VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "📋 Creating GitHub release for v$(VERSION)..."
	@if [ -f "RELEASE_NOTES_RC$(shell echo $(VERSION) | sed 's/.*rc\.//').md" ]; then \
		echo "📄 Using existing release notes..."; \
		gh release create v$(VERSION) \
			--title "UltraFast MCP v$(VERSION)" \
			--notes-file RELEASE_NOTES_RC$(shell echo $(VERSION) | sed 's/.*rc\.//').md; \
	else \
		echo "📄 Creating default release notes..."; \
		gh release create v$(VERSION) \
			--title "UltraFast MCP v$(VERSION)" \
			--notes "Release v$(VERSION) of UltraFast MCP"; \
	fi
	@echo "✅ GitHub release created!"

# Full release process
release:
	@if [ -z "$(VERSION)" ]; then \
		echo "❌ Error: VERSION is required. Use: make release VERSION=x.y.z"; \
		echo "Example: make release VERSION=202506018.1.0-rc.1.3"; \
		exit 1; \
	fi
	@echo "🚀 Starting full release process for v$(VERSION)..."
	@echo ""
	@echo "Step 1: Preparing release..."
	$(MAKE) release-prepare VERSION=$(VERSION)
	@echo ""
	@echo "Step 2: Building and testing..."
	$(MAKE) build
	$(MAKE) test
	@echo ""
	@echo "Step 3: Pushing changes..."
	$(MAKE) release-push
	@echo ""
	@echo "Step 4: Creating git tag..."
	$(MAKE) release-tag VERSION=$(VERSION)
	@echo ""
	@echo "Step 5: Creating GitHub release..."
	$(MAKE) release-create VERSION=$(VERSION)
	@echo ""
	@echo "🎉 Release v$(VERSION) complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  - Monitor GitHub Actions CI/CD pipeline"
	@echo "  - Verify crates are published to crates.io"
	@echo "  - Check documentation deployment"

# Utility commands
package-check:
	@echo "📦 Checking package validity..."
	@for crate in crates/*/; do \
		if [ -f "$$crate/Cargo.toml" ]; then \
			echo "Checking $$(basename $$crate)..."; \
			cd $$crate && cargo package --allow-dirty || exit 1; \
			cd ../..; \
		fi; \
	done
	@echo "✅ All packages are valid!"

version-check:
	@echo "🔍 Checking version consistency..."
	@WORKSPACE_VERSION=$$(grep 'version = ' Cargo.toml | head -1 | sed 's/.*version = "\(.*\)".*/\1/'); \
	echo "Workspace version: $$WORKSPACE_VERSION"; \
	for crate in crates/*/; do \
		if [ -f "$$crate/Cargo.toml" ]; then \
			CRATE_VERSION=$$(grep 'version = ' $$crate/Cargo.toml | head -1 | sed 's/.*version = "\(.*\)".*/\1/'); \
			if [ "$$CRATE_VERSION" != "$$WORKSPACE_VERSION" ]; then \
				echo "❌ Version mismatch in $$(basename $$crate): $$CRATE_VERSION != $$WORKSPACE_VERSION"; \
				exit 1; \
			fi; \
		fi; \
	done
	@echo "✅ All versions are consistent!"

# Development workflow
dev-setup:
	@echo "🔧 Setting up development environment..."
	cargo install cargo-watch
	cargo install cargo-audit
	@echo "✅ Development environment ready!"

watch:
	@echo "👀 Watching for changes..."
	cargo watch -x check -x test

audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Documentation
docs:
	@echo "📚 Building documentation..."
	cargo doc --workspace --no-deps --open

docs-serve:
	@echo "🌐 Serving documentation..."
	cargo doc --workspace --no-deps
	python3 -m http.server 8000 -d target/doc

# Benchmarks
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench

# Examples
examples:
	@echo "📖 Building examples..."
	cargo build --examples

examples-run:
	@echo "🚀 Running examples..."
	@for example in examples/*/; do \
		if [ -f "$$example/Cargo.toml" ]; then \
			echo "Running $$(basename $$example)..."; \
			cd $$example && cargo run --bin server & \
			sleep 2 && cargo run --bin client; \
			cd ../..; \
		fi; \
	done 
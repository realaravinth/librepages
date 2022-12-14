define cache_bust ## run cache_busting program
	npm run sass
	cd utils/cache-bust && cargo run
endef

default: ## Debug build
	$(call cache_bust)
	cargo run -- serve

cache-bust: ## Run cache buster on static assets
	$(call cache_bust)

check: ## Check for syntax errors on all workspaces
	cargo check --workspace --tests --all-features
	#cd utils/cache-bust && cargo check --tests --all-features

clean: ## Clean all build artifacts and dependencies
	@cargo clean

coverage: ## Generate HTML code coverage
	$(call cache_bust)
	cargo tarpaulin -t 1200 --out Html

dev-env: ## Download development dependencies
	npm install
	cargo fetch
	./scripts/conductor.sh

doc: ## Prepare documentation
	cargo doc --no-deps --workspace --all-features

docker: ## Build docker images
	docker build \
		-t realaravinth/pages:master \
		-t realaravinth/pages:latest \
		-t realaravinth/pages:0.1.0 .

docker-publish: docker ## Build and publish docker images
	docker push realaravinth/pages:master 
	docker push realaravinth/pages:latest
	docker push realaravinth/pages:0.1.0

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features

migrate: ## run migrations
	$(call cache_bust)
	unset DATABASE_URL && cargo build
	cargo run -- migrate

release: ## Release build
	$(call cache_bust)
	cargo build --release

run: default ## Run debug build
	cargo run -- serve

sqlx-offline-data: ## prepare sqlx offline data
	cargo sqlx prepare  \
		--database-url=${DATABASE_URL} -- \
		--all-features

test: ## Run tests
	$(call cache_bust)
	cargo test --all-features --no-fail-fast

xml-test-coverage: ## Generate cobertura.xml test coverage
	$(call cache_bust)
	cargo tarpaulin -t 1200 --out Xml

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z_-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

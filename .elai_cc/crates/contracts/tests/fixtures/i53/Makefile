.PHONY: clean build check

build: ## Compile the project
	cargo build

clean:
	rm -rf target

check: ## Run cargo check
	cargo check

output.txt: src/main.rs
	echo "file target not phony"

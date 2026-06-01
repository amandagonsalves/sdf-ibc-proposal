.PHONY: install f0 f1 f1-create-client f1-create-stellar-client upload-lc-wasm deploy-contracts caribic-preflight cosmos-only api-doc

SCRIPT_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
FLOWS_DIR  := $(SCRIPT_DIR)flows
REPO_ROOT  := $(SCRIPT_DIR)..

install:
	@cargo install --path $(REPO_ROOT)/cli --force

f0:
	@cargo run -q --manifest-path $(REPO_ROOT)/Cargo.toml -p stellar-ibc-cli -- bootstrap

f1:
	@bash $(FLOWS_DIR)/f1-create-cosmos-client.sh

f1-create-client:
	@bash $(FLOWS_DIR)/f1-create-client.sh

f1-create-stellar-client:
	@bash $(FLOWS_DIR)/f1-create-stellar-client.sh

upload-lc-wasm:
	@cargo run -q --manifest-path $(REPO_ROOT)/Cargo.toml -p stellar-ibc-cli -- contracts upload-wasm

deploy-contracts:
	@cargo run -q --manifest-path $(REPO_ROOT)/Cargo.toml -p stellar-ibc-cli -- contracts deploy-all

caribic-preflight:
	@bash $(FLOWS_DIR)/caribic-preflight.sh

cosmos-only:
	@bash $(FLOWS_DIR)/cosmos-only.sh

api-doc:
	@cargo doc -p stellar-api --no-deps --open

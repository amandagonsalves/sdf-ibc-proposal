.PHONY: f0 hermes-image gateway-image upload-lc-wasm deploy-contracts caribic-preflight

SCRIPT_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
FLOWS_DIR  := $(SCRIPT_DIR)flows

f0:
	@bash $(FLOWS_DIR)/f0-bootstrap.sh

hermes-image:
	@bash $(FLOWS_DIR)/build-hermes-image.sh

gateway-image:
	@bash $(FLOWS_DIR)/build-gateway-image.sh

upload-lc-wasm:
	@bash $(FLOWS_DIR)/upload-lc-wasm.sh

deploy-contracts:
	@bash $(FLOWS_DIR)/upload-and-deploy-contracts.sh

caribic-preflight:
	@bash $(FLOWS_DIR)/caribic-preflight.sh


.PONY: setup build-wasm

setup:
	@echo Setting up environment...
	@echo Build wasm-sys...
	cd ./wasm-sys && wasm-pack build --features debuggable
	@echo Install dependencies...
	pnpm install
	@echo Build wasm...
	cd ./wasm && pnpm run build

build-wasm:
	@echo Build wasm...
	cd ./wasm-sys && wasm-pack build --features debuggable
	cd ./wasm && pnpm run build

build-circuits:
	@echo Build circuits...
	cd ./circuits && npx hardhat compile-circuit --degree 08 elgamal_pubkey
	cd ./circuits && npx hardhat compile-circuit --degree 21 shuffle_encrypt
	cd ./circuits && pnpm run build
CIRCUITS = elgamal_pubkey

define mkdir
ifeq ($(OS),Windows_NT)
	@mkdir "$(1)"
else
	@mkdir -p $(1)
endif
endef

.PHONY: all clean deploy

clean:
ifeq ($(OS),Windows_NT)
	@del /s /q build
else
	@rm -rf build
endif

all: circuits zkey sol

cache/%.ptau:
	@echo "Downloading $*.ptau"
	$(call mkdir,$(dir $@))
	@curl -L https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_$*.ptau -o $@

circuits: $(addprefix build/,$(addsuffix .r1cs,$(CIRCUITS)))
zkey: $(addprefix build/,$(addsuffix .zkey,$(CIRCUITS)))
sol: $(addprefix contracts/,$(addsuffix .sol,$(CIRCUITS)))

build/elgamal_pubkey.r1cs: circuits/elgamal_pubkey.circom
build/elgamal_pubkey.zkey: build/elgamal_pubkey.r1cs cache/08.ptau

build/%.r1cs:
	@echo "Building $*"
	$(call mkdir,$(dir $@))
	@circom $< --r1cs --sym --wasm --O2 --output $(dir $@)

build/%.zkey:
	@echo "Building $@"
	@snarkjs plonk setup $< $(word 2,$^) $@

contracts/%.sol: build/%.zkey
	@echo "Building $@"
	$(call mkdir,$(dir $@))
	@snarkjs zkey export solidityverifier $< $@
# zkMahjong

## Build

### Prerequisites

-   [Rust toolchain](https://www.rust-lang.org/tools/install)
-   [make](https://www.gnu.org/software/make/#download)
    -   Windows:
        -   Install [Chocolatey](https://chocolatey.org/install)
        -   Run `choco install make` in an elevated command prompt
-   [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
-   [pnpm](https://pnpm.js.org/en/installation)
-   [circom2](https://docs.circom.io/getting-started/installation/)
-   [snarkjs](https://github.com/iden3/snarkjs#install-snarkjs)

### Setup

```
make setup
```

### Manual Setup

```
# build zk-mahjong-wasm-sys package
cd wasm-sys
wasm-pack build --features debuggable

# install dependencies
cd ..
pnpm install

# build zk-mahjong-wasm package
cd wasm
pnpm run build
```

### Run dev server

```
# start web server
cd frontend
pnpm run dev
```

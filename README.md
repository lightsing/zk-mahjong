# zkMahjong


## Build

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [pnpm](https://pnpm.js.org/en/installation)

### Setup

```
git clone https:://github.com/lightsing/zk-mahjong
cd zk-mahjong

# build zk-mahjong-wasm-sys package
cd wasm-sys
wasm-pack build --features debuggable

# install dependencies
cd ..
pnpm install

# build zk-mahjong-wasm package
cd wasm
pnpm run build

# start web server
cd web
pnpm run dev
```

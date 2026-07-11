# Stick War

Stick War is a small side-view strategy game built with Rust and [Bevy](https://bevy.org/). Build an economy, train an army of stick figures, and destroy the opposing statue before the enemy destroys yours.

The game is inspired by the simple silhouettes, tug-of-war battlefield, and mix of army-level orders and direct unit control found in classic browser strategy games such as the *Stick War* series. This is an independent project and is not affiliated with its creators.

## Gameplay

- Train **miners** to gather gold, **swordsmen** for close combat, and **archers** for ranged support.
- Issue **attack**, **defend**, or **retreat** orders to the whole army.
- Press <kbd>Tab</kbd> to take direct control of a swordsman or archer.
- Defeat the enemy by reducing its statue's health to zero.

| Action | Keyboard |
| --- | --- |
| Train miner / swordsman / archer | <kbd>M</kbd> / <kbd>S</kbd> / <kbd>R</kbd> |
| Attack / defend / retreat | <kbd>1</kbd> / <kbd>2</kbd> / <kbd>3</kbd> |
| Cycle directly controlled unit | <kbd>Tab</kbd> |
| Move controlled unit | <kbd>A</kbd>/<kbd>D</kbd> or arrow keys |
| Attack with controlled unit | <kbd>Space</kbd> |
| Release controlled unit | <kbd>Esc</kbd> |
| Pan camera (when no unit is controlled) | Left/right arrow keys |

The HUD buttons provide mouse controls for training and army orders.

## Local setup

### Prerequisites

- A current stable [Rust toolchain](https://rustup.rs/)
- Git
- A graphics driver supported by Bevy

Clone and run the native game:

```sh
git clone https://github.com/Spaceface16518/Stick-War.git
cd Stick-War
cargo run
```

The first build compiles Bevy and may take a few minutes. Gameplay values such as costs, health, AI timing, and unit speed can be adjusted in `config/game_config.ron`.

### macOS

Install the Xcode command-line tools if they are not already present, then install Rust and run the commands above:

```sh
xcode-select --install
```

### Windows

Install Rust with `rustup-init.exe`. When prompted, install the **Desktop development with C++** workload from Visual Studio Build Tools, which provides the MSVC linker required by the Rust toolchain. Then run the setup commands in PowerShell.

### Linux

Bevy needs a C compiler and common windowing, audio, and device libraries. On Ubuntu or Debian:

```sh
sudo apt update
sudo apt install build-essential pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

Package names differ by distribution; after installing the equivalents, run `cargo run` as above.

## Browser build

The browser build additionally requires Node.js/npm, Rust's WebAssembly target, and `wasm-bindgen-cli`. The CLI version must match the `wasm-bindgen` version in `Cargo.lock` (currently `0.2.126`).

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.126 --locked
npm run build:web
npm run serve:web
```

Open the local URL printed by `serve`. The build script creates `dist/`, compiles an optimized WebAssembly binary, generates its JavaScript bindings, and copies in the browser shell from `web/`.

## Project layout

- `src/` — Bevy gameplay, combat, rendering, units, and UI
- `config/game_config.ron` — balance and behavior configuration
- `web/` — HTML and JavaScript browser shell
- `scripts/build-web.mjs` — WebAssembly build pipeline

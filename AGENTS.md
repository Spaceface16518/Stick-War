# AGENTS.md

Guidance for coding agents working in this repository.

## Project overview

Stick War is a single-crate Rust game built with Bevy 0.19. It runs as a native desktop application and can be compiled to WebAssembly. The player trains miners, swordsmen, and archers, issues army orders, and can directly control combat units. A battle ends when either team's statue reaches zero health.

## Repository map

- `src/main.rs`: Bevy app and window setup.
- `src/game.rs`: top-level plugin, shared resources, and camera setup.
- `src/battle.rs`: battle lifecycle, ordered system sets, input, AI, movement, combat, economy, and victory checks.
- `src/model.rs`: components, resources, messages, shared gameplay calculations, and unit tests.
- `src/units.rs`: unit, statue, and gold-deposit spawning and visuals.
- `src/rendering.rs`: battlefield rendering, animation, health bars, and presentation helpers.
- `src/ui.rs`: menu, HUD, buttons, results screen, and displayed control hints.
- `src/config.rs` and `config/game_config.ron`: typed configuration and gameplay balance values.
- `web/`: browser shell loaded by the WebAssembly build.
- `scripts/build-web.mjs`: browser build pipeline; output goes to ignored `dist/`.

## Development commands

Use the checked-in lockfile for reproducible verification:

```sh
cargo run --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

For browser builds, install the `wasm32-unknown-unknown` Rust target and a `wasm-bindgen-cli` version matching the `wasm-bindgen` package in `Cargo.lock`, then run:

```sh
npm run build:web
npm run serve:web
```

Do not commit generated `target/`, `dist/`, or the local `bevy/` directory; all are ignored.

## Architecture and gameplay conventions

- Keep simulation order explicit. `BattleSet` in `src/battle.rs` chains input, decisions, movement, combat, consequences, and presentation; place new systems in the appropriate set.
- Scope battle-only systems with `AppState::Battle`, and tag spawned battle objects with `BattleEntity` so restart and menu transitions clean them up.
- Put tunable balance values in `config/game_config.ron` and add corresponding typed fields in `src/config.rs`. Avoid scattering balance constants through systems.
- `load_game_config()` reads the external RON file for native development and falls back to the compile-time embedded copy. Keep the file valid for both paths.
- Preserve team symmetry by using helpers such as `Team::direction`, `Team::opponent`, `statue_x`, `mine_x`, and formation helpers rather than duplicating player/enemy branches.
- Communicate training and damage through the existing Bevy messages (`TrainUnitRequest` and `DamageMessage`).
- Controlled combat units are excluded from automatic movement and attacks. If direct-control behavior changes, verify both controlled and autonomous paths.
- Projectiles must continue to ignore friendly units and use swept collision so fast arrows do not tunnel through targets.
- If a keyboard binding or cost shown to the player changes, update both the functional input handling and labels/control hints in `src/ui.rs`, plus `README.md` when applicable.

## Testing expectations

- Add small deterministic unit tests in `src/model.rs` for pure gameplay calculations and invariants.
- Run `cargo fmt --check` and `cargo test --locked` after Rust changes.
- Run Clippy for logic or structural changes when practical.
- Run `npm run build:web` when changing Cargo features/profiles, startup, configuration loading, the web shell, or the build script.
- For gameplay/UI changes, launch the native game and smoke-test training, all three army orders, direct control, restart, and returning to the main menu as relevant.

## Platform notes

- Prefer existing Node/npm and Rust tooling. Do not install development tooling globally on the host when a project-local command or isolated container is sufficient.
- Native Bevy builds require platform graphics/windowing dependencies; see `README.md` for macOS, Windows, and Linux prerequisites.
- Browser builds use `--no-default-features` because the default `fast-compile` feature enables Bevy dynamic linking, which is intended for native development rather than WebAssembly.

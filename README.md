# Stick War

Stick War is a small side-view strategy game built with Rust and [Bevy](https://bevy.org/). Build an economy, train an army of stick figures, and destroy the opposing statue before the enemy destroys yours.

The game is inspired by the simple silhouettes, tug-of-war battlefield, and mix of army-level orders and direct unit control found in classic browser strategy games such as the *Stick War* series. This is an independent project and is not affiliated with its creators.

## Gameplay

- Train **miners** to gather gold, **swordsmen** for close combat, and **archers** for ranged support.
- Issue **attack**, **defend**, or **retreat** orders to the whole army.
- Press <kbd>Tab</kbd> to take direct control of a swordsman or archer.
- Defeat the enemy by reducing its statue's health to zero.
- Use **Sandbox** mode to train units and issue orders for either team, change
  the population cap, pause the simulation, and toggle training costs or times.
  Sandbox starts paused with costs and training time disabled.

| Action | Keyboard |
| --- | --- |
| Train miner / swordsman / archer | <kbd>M</kbd> / <kbd>S</kbd> / <kbd>R</kbd> |
| Attack / defend / retreat | <kbd>1</kbd> / <kbd>2</kbd> / <kbd>3</kbd> |
| Cycle directly controlled unit | <kbd>Tab</kbd> |
| Move controlled unit | <kbd>A</kbd>/<kbd>D</kbd> or arrow keys |
| Attack with controlled unit | <kbd>Space</kbd> |
| Release controlled unit | <kbd>Esc</kbd> |
| Pan camera (when no unit is controlled) | Swipe on touchscreens, sideways mouse/trackpad scroll, or left/right arrow keys |

The compact HUD includes the battle timer, both economies and statue health,
active training countdowns, and mouse controls for training and army orders.
Only one unit of each type can train for a team at once. Unit training times,
like the other balance values, are configured in `config/game_config.ron`.

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

Screen lifecycles are separated with Bevy states. Menu and results entities and
systems exist only in their respective states and use Bevy's reactive desktop
update mode, so static UI screens sleep between window/input events. Entering a
battle switches to the continuous game update mode required by simulation and
animation; leaving it cleans up battle entities and restores reactive updates.
This policy is centralized in the computed `RuntimeActivity` state: new screens
default to reactive behavior, while states that run gameplay simulation must be
explicitly classified as continuous.

## Character animation editor

The repository includes a native example for authoring hot-reloaded, rigged 2D
characters and modular weapons:

```sh
cargo run --example character_editor --locked
```

While it runs, edit the character, animation, and weapon definitions under
`config/characters/`. Character RON files contain the joint hierarchy,
appearance, animation timing, and keyframes. Weapon RON files are separate and
attach to a named hand joint; the sword is static, while the bow demonstrates
procedural string and arrow states layered onto the animated hand transform.
The editor toolbar lists its keyboard controls, current frame/time, loaded
assets, playback mode, hot-reload count, and cursor world coordinates.
Press <kbd>V</kbd> to switch between the clean drawn-character preview and the
colored rig view. The character RON `visuals` block controls rounded limb width,
head/hand/foot ellipse sizes, and rig overlay dimensions.
Joint `length` is optional. Without it, the rendered bone connects directly to
the first child joint; leaf joints become attachment points. Use an explicit
length only for intentional offsets such as placing the head ellipse above its
joint.

The battle runtime loads these same RON assets through Bevy's asset server for
swordsmen and archers. Movement and combat state select the authored clips,
enemy rigs are mirrored, and the modular weapon remains attached to `hand_r`.
The bow string, nocked arrow, and draw amount are generated procedurally from
the archer's live attack timing; editing a character or weapon RON file during
native development hot-reloads the corresponding battlefield visuals.

In rig view, hover a joint and press <kbd>E</kbd> to edit its position key for
the current animation frame. Arrow keys move by 5 units and Shift+arrow moves
by 0.1 units. Editing initially snaps the cursor to the joint and follows mouse
movement. The first arrow press restores the edit-start position and switches
to keyboard-only movement. Press <kbd>E</kbd> again to serialize the character
back to its RON file, or <kbd>Esc</kbd> to restore the pre-edit asset and exit.
Playback, frame stepping, restart, and character/animation/weapon-state changes
also cancel the pending edit before performing their normal action.

The <kbd>A</kbd> cycle includes a non-playing **base model** entry. Edits made in
that entry update the joint's base `position`; edits made while an animation is
selected update only its current frame key.

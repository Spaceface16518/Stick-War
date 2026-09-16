# Rewrite validation

This is a browser-first release candidate on `codex/3d-rewrite`, based on verified `origin/master` commit `1f1883191a8c644b8d5981d785093a229efc650c`. The original character-editor checkout was not replaced. Its two uncommitted pose files were preserved and hash-checked.

## Automated checks

- TypeScript strict checking, ESLint and Prettier.
- 29 headless tests: economy symmetry over full miner trajectories, gathering/carrying/return, training reservation and cap rules, all orders, retreat around home statues, interrupted miners returning to deposits, backpedaling archers facing their targets, AI recruitment/replacement, direct-control ownership, facing-based swords, manual ballistic bows, cooldown preservation, frozen attack/training parameters, swept first-impact filtering, simultaneous draw, sandbox continuation, cleanup and a victory using unchanged balance.
- Browser suite: 16 applicable scenarios pass across desktop 1280 × 720 and emulated landscape touch 844 × 390; two cases are intentionally skipped on the inapplicable device project. Scenarios include both possession classes, complete victory/defeat, all orders, restart/menu, five repeated resets, stable geometry/texture counts, loading failure/retry, hot reload rejection/recovery, pointer-lock loss, simultaneous touch movement/aim/attack, portrait pause, 100-unit stress and static-menu render idling.
- Nine optimized GLBs pass Khronos validation with zero errors. Required materials, clips, joints and arena anchors survive optimization. Model details are recorded in `asset-validation.json`.
- The complete production site is approximately 5.9 MiB raw, under the 12 MiB budget. The gzip total is an estimate, not a measurement of GitHub Pages transfer encoding.

## Evidence

`evidence/` contains actual game screenshots, not concept art:

- `desktop-commander.png`, `desktop-swordsman-pov.png`, `desktop-archer-pov.png`.
- `mobile-commander.png`, `mobile-swordsman-pov.png`, `mobile-archer-pov.png`, `mobile-touch-combat.png`.
- `performance-desktop.json` and `performance-touch.json` contain frame-interval samples and simulation/rendering object counts.

Performance samples use the local Mac's Chrome in headless mode, up to 180 rendered frame intervals per scenario, bounded to a three-second window. The endurance scenario gives combat units additional health solely in the test to hold population while fighting; movement, collision, attack timings and rendering remain unchanged. Timing is not a guarantee for every GPU/browser. The separate simulation stress scenario uses ordinary unit health.

Physical phone/tablet performance and Safari compatibility have **not** been measured. The touch project verifies layout and concurrent PointerEvent handling on a desktop browser; it cannot establish actual mobile thermal, GPU, touch latency or Safari behavior. Those device checks remain a release follow-up.

## Reproduce and release

```sh
npm ci
npm run check
npm run lint
npm run format:check
npm test
npm run assets:check
npm run build:web
npm run test:browser
```

GitHub Actions performs these gates for the PR and master. After merge to `master`, the same workflow publishes the Vite `dist/` artifact to Pages. This draft PR does not replace the currently published game until merged. Blender is required only for authoring, not for running the game or validating committed runtime assets.

Recorded combat-centered samples averaged 16.67 ms per frame (approximately 60 fps) for both 24 and 100 units on local headless Chrome. Desktop 95th-percentile intervals were 16.8 ms; landscape emulation was 16.8/16.7 ms. These are short, refresh-limited samples, not a long-duration device benchmark. `desktop-commander-combat.png` and `mobile-commander-combat.png` show the measured scene, paused immediately after sampling.

CI browser tests allow longer action budgets than local tests. Timing samples stop after three seconds even on slow renderers; a fixed frame-count wait must not turn low frame rate into a hung test. The first Linux run reached the final menu step but exhausted its old 45-second test budget, and its fixed-frame timing probes timed out. The CI-specific budgets and bounded timing probe address those failures.

Continuous trace screenshots are disabled because the Linux renderer reported repeated GPU readback stalls. Explicit acceptance screenshots and failure screenshots are retained, along with DOM/action traces. This keeps the recorder from dominating WebGL test timing. Pointer-lock fallback is explicitly exercised: when the browser cannot lock, left mouse still attacks and dragging still aims.

The Linux rerun also exposed a file-watching race: two hot updates arrived, but the rapid restoration after invalid data did not, leaving the server's transformed module stale for subsequent pages. Test configuration edits now use atomic replacement, and the development watcher waits for completed writes. The reload test still requires the running session to recover to the original values and clear its error; it does not bypass the real hot-reload path.

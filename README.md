# Stick War 3D

A browser strategy game on a narrow 3D battlefield. Train miners, swordsmen and archers; protect your statue; take first-person control of a combat unit. Original tabletop visuals, TypeScript simulation, Three.js presentation and Rapier collision queries.

## Run locally

Node 22 or newer; dependencies are project-local.

```sh
npm ci
npm run dev
```

Open the printed local URL. Desktop browsers and landscape touch layouts are supported. Portrait orientation and backgrounding pause the game. Audio starts after a user gesture.

```sh
npm run check
npm run lint
npm test
npm run test:browser
npm run assets:check
npm run build:web
```

Playwright uses installed Chrome locally. In CI, install its Chromium browser with `npx playwright install --with-deps chromium`. The production site is generated in `dist/`; `npm run serve:web` previews it. GitHub Pages uses relative asset URLs.

## Play

| Action | Desktop | Touch |
| --- | --- | --- |
| Train miner / sword / archer | M / S / R in commander view | Training buttons |
| Attack / defend / retreat | 1 / 2 / 3 | Order buttons |
| Possess / cycle | Tab, or select then Control | Select then Control; cycle button |
| Move in first person | WASD / arrows | Left stick |
| Aim | Mouse; drag if pointer lock unavailable | Right-side drag |
| Attack | Hold left mouse / Space | Hold attack; drag it to aim simultaneously |
| Release | Escape | Commander button |
| Commander camera | Arrows, horizontal scroll, wheel zoom | Swipe / pinch |

Both teams start with 150 gold, one miner and a 500-health statue. Miners carry 25 gold home; passive income adds 5 gold every 2 seconds. Training reserves gold and population immediately, with one slot per class. Defend holds the home formation and intercepts threats throughout your half of the field, including ranged attackers and troops behind the statue. Reinforcements enter through rear lanes beside the statue. Retreat cancels automatic attacks and recalls miners with their carried gold. Destroy the opposing statue to win.

Sandbox starts paused with free, instant training. Choose either team, set orders, change the cap, possess either army, and enable costs or training timers. Existing units and reservations survive cap reductions. Statue destruction does not end sandbox.

## Code and content

- `src/simulation/`: fixed 60 Hz rules, typed commands, snapshots and events; no renderer or DOM dependencies.
- `src/spatial/`: Rapier adapter for movement, melee and swept projectiles.
- `src/presentation/`: asset loading, Three.js cameras, animation and bounded effects.
- `src/platform/` and `src/ui/`: input, audio, settings and DOM interface.
- `config/`: validated balance and arena JSON. Vite reloads valid balance changes; invalid values retain the previous configuration. Arena changes require restarting.
- `art/`: Blender sources and reproducible authoring/export scripts.
- `public/assets/`: runtime GLBs and size manifest.
- `tests/`: deterministic rules and browser acceptance tests.

See [rewrite decisions](docs/REWRITE.md). The previous Rust game and editor remain in Git history; balance reference values are retained in `docs/legacy-game-config.ron`.

Battle saves, native packaging and full ragdoll simulation are outside this release. Device performance targets are 60 fps desktop and 30 fps phones; browser emulation does not establish physical phone performance.

[Blender authoring and export guide](art/README.md) · [Validation and measured limits](docs/VALIDATION.md) · [Commander gameplay screenshot](docs/evidence/desktop-commander-combat.png) · [First-person screenshot](docs/evidence/desktop-swordsman-pov.png)

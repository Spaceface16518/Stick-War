# Stick War 3D

TypeScript browser game. Keep simulation free of DOM and Three.js. Input and AI submit typed commands. Simulation order and 60 Hz timestep are explicit. Teams share all mechanics. Possession suppresses AI movement and attacks; never reset cooldowns when switching. Projectiles use filtered swept collision with first time of impact.

Balance lives in config/game.json; arena anchors are exported from Blender. Validate hot reloads. Visual animation never applies damage. All battle resources must be disposed on restart/menu.

Use npm ci, npm run check, npm run lint, npm test, npm run build:web, npm run assets:check, npm run test:browser. Use project-local Node packages and installed Blender 5; keep extra tooling in containers. Never commit node_modules, dist or scratch exports.

Source art lives in art/blender; runtime GLB files and the manifest live in public/assets. Regenerate with npm run assets:build. Inspect actual browser screenshots after visual changes. Test desktop and landscape multi-touch separately; emulation does not prove physical-device performance.

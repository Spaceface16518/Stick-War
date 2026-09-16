# Stick War 3D

Browser-first TypeScript / Three.js / Rapier rewrite. Reference master: 1f1883191a8c644b8d5981d785093a229efc650c; reference character branch: f01ffd2. Historical RON is retained beside this document. Old code and the character editor remain available in Git history.

Simulation is independent of presentation. Commands are processed while paused; only running sessions advance the 60 Hz clock. Three.js consumes snapshots and events. Rapier supplies spatial queries and kinematic collision movement. Both teams use the same rules.

Intentional corrections: passive income for both teams, immediate free/instant sandbox placement, projectile first-time-of-impact selection, deterministic simultaneous-destruction draw.

Assets are authored in Blender 5 and exported to GLB, with a shared humanoid rig, in-place animation and explicit attachment sockets. Source files, scripts and runtime exports are committed. No bespoke character editor.

## Milestones
1. Complete headless rules and simple-model playable game, desktop and landscape touch.
2. Blender characters, first-person models, ridge scene, animation and sound.
3. Browser QA, asset/bundle validation and Pages CI.

## Reference art
The concept studies were generated using built-in image generation during design. They express art direction, not measured game output. Production detail is constrained by the browser budgets.

Camera prompt: two views of the same narrow linear grassy limestone battlefield, elevated side-on command view and eye-level first-person swordsman view; matching blue/red bases, guardian statues, miners, swordsmen and archers; sculpted painted-tabletop medieval humans, warm sunlight, blue and terracotta team accents, no dense HUD.

Character prompt: three-column MINER / SWORDSMAN / ARCHER sheet, each with full-body three-quarter and rear views; recognizable human faces, six-head proportions, practical pickaxe and sack / sword with empty off-hand / bow and quiver; common humanoid rig proportions, tactile cloth/leather/steel, interchangeable cobalt and terracotta team accents.

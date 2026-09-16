# Rig animation and character detail pass

Geometry and reusable rig actions remain together in each Blender source. The authoring scripts separate model design, skeleton conventions and choreography. The ordinary asset build now exports saved `.blend` edits; explicit regeneration commands distinguish actions from the complete model. Action-only regeneration was run on all five character/first-person sources and verified unchanged mesh coordinates, topology, materials and skin weights before saving. A fresh-cache, single-source export also verified that the saved Blender source and all seven unrelated runtime GLBs stayed unchanged, with all nine manifest entries retained.

## Motion and mechanics

- Six distance-driven gait clips cover walking, running, carrying, backpedaling and both lateral directions. After-collision displacement drives phase, including interpolated frames; smoothed targeting velocity cannot make an obstructed unit keep stepping. Planted-foot trajectories cancel world travel, with knee bend, heel strike, swing clearance and articulated toe-off. Upper-body attacks retain the moving leg layer.
- Swords choose slash, backhand or overhead. Bows choose quick, ordinary or held draw. Weighted seeded selection and ±7% tempo variation change windup and recovery while retaining damage, range and the same rules for possessed/autonomous units. All random choices use the existing 64-bit seed generator.
- A swing/shot captures its clip, contact marker, windup and cooldown when starting. Active parameters survive balance reload. Possession still cancels autonomous windups and retains cooldown. Animation never produces damage.
- Directional hit clips add compression, head lag and settling over the current action. Aggregated hit impulses select one of four falls. The current posed skeleton transfers into the bounded corpse pool, blends into the fall and slides briefly with impact/movement. Knees buckle, arms brace and bodies/equipment settle against the floor; this is authored cosmetic motion rather than a ragdoll simulation.
- Character additions include facial planes and eyelids, layered armor and rivets, tunic/apron seams, boot soles/laces/toes, hand/finger shapes and equipment trim. The sword-only loadout remains.

| Attack | Selection weight | Base windup before tempo variation | Base cooldown before existing jitter/extra |
| --- | ---: | ---: | ---: |
| Slash | 35% | 0.152 s | 0.656 s |
| Backhand | 40% | 0.240 s | 0.800 s |
| Overhead | 25% | 0.352 s | 1.024 s |
| Quick bow | 25% | 0.297 s | 1.260 s |
| Draw bow | 50% | 0.440 s | 1.400 s |
| Held bow | 25% | 0.616 s | 1.540 s |

Sword damage stays 20; arrow damage stays 15. Weighted base cooldowns average 0.806 s and 1.400 s respectively, before the retained 0.010 s extra and ±0.005 s jitter. This keeps average cadence close to the previous balance while making individual attacks less synchronized.

## Verification

Headless tests cover deterministic style/timing sequences, distinct windups, contact within one 60 Hz step, captured timing after reload, summed directional impacts, gait progress by distance, pause/obstruction and additive recoil during a moving attack. Existing complete sandbox scenarios and unchanged-balance victory remain regression coverage.

Actual GLB browser checks cover all three bow actions in close/commander/first-person assets, walking/running/carrying/backwards/lateral planted feet, and four falls on each body model. The tests caught and corrected lateral travel reversal, floor penetration, the exporter's retained one-frame start offset and release keys straddling the sampling interval. Optimized clips now start at zero and preserve exact contact keys/durations; validation rejects regressions.

Ground/foot tolerances allow interpolation and compression: planted-foot drift below 15 mm over the tested stance segment; sampled fall surfaces within 12 mm of the floor, with a support point within 25 mm. Bow grip and drawing-hand gaps must stay below 40 mm during the held part of the draw. These are checks of the exported models, not just source rig calculations.

Close characters contain 5,020–5,714 triangles; commander variants contain 1,755–1,766, with two main materials and the shared 1K atlas. All nine GLBs pass Khronos and project validation. The complete production site is approximately 7.27 MiB raw / 3.15 MiB estimated gzip.

The full browser suite also exercises victories/defeats, the mirrored three-archer defense reproduction, both possession classes, concurrent touch move/aim/attack, pointer-lock loss, pause/background behavior, restart/menu, loading retry, configuration reload, 100-unit sandbox stress, and repeated corpse disposal. Desktop and landscape touch are local Chrome checks; physical-phone performance and Safari remain unmeasured.

## Evidence and reproduction

`evidence/animation-detail-poses.png` is a contact sheet rendered from the actual optimized GLBs. The desktop/mobile commander and both POV captures show the updated models in the game. See `VALIDATION.md` for the complete verification commands and evidence paths.

```sh
npm run assets:animate -- --asset swordsman
npm run assets:check
npm test
npx playwright test tests/rig.spec.ts
npm run test:browser
```

Use `npm run assets:build` to export manual Blender edits, and reserve `assets:animate` for deliberately replacing rig actions. See [the replacement mesh workflow](../art/README.md).

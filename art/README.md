# Blender authoring

Each character's `.blend` contains its geometry, shared humanoid rig and reusable rig actions. Replace the mesh on that skeleton to reuse its animations in the same file. The first-person arms use a smaller separate rig. All models and animation are original, authored with installed Blender 5.2.2 LTS.

## Edit a character or reuse its animation

1. Open `sources/miner.blend`, `swordsman.blend` or `archer.blend`. Select the armature and choose an action in Blender's Action Editor to preview it. The corresponding NLA tracks are muted to keep clips independent.
2. Edit the existing mesh, or add a replacement mesh at metre scale with feet at the origin. Keep the armature's bone names, hierarchy and rest transforms. Parent the new mesh to the existing armature and assign/transfer weights; then remove or hide the old geometry from export. The actions already animate that rig, so no copying of mesh animation is needed.
3. Keep the palm sockets and the appropriate equipment. Use the `Team cloth` material for recoloring; use `Painted palette` for everything else. The shared atlas is `textures/tabletop-palette.png` (1024 × 1024).
4. Save the `.blend`, then run `npm run assets:build -- --asset swordsman` (substitute the asset ID) and `npm run assets:check`. Inspect the result in the game, including close and commander views.

A model with substantially different proportions or a different skeleton needs retargeting and pose adjustments. In particular, check planted feet, drawing-hand reach and the floor during falls. Reweighting geometry onto the unchanged skeleton preserves the existing animation directly.

## Export versus regenerate

```sh
npm run assets:build                         # Export saved Blender files; preserve manual edits
npm run assets:build -- --asset archer       # Export one saved source and its commander LOD
npm run assets:animate -- --asset archer     # Replace rig actions; preserve geometry and weights
npm run assets:author -- --asset archer      # Regenerate geometry, rig AND actions from scripts
npm run assets:check
```

`assets:build` never regenerates or saves source files. `assets:animate` explicitly replaces the actions in the selected source; it compares mesh geometry, topology, materials and skin weights before/after and aborts if those changed. `assets:author` replaces the selected source from the procedural authoring code. Use these two regeneration commands only when deliberately changing the generated art or motion. Without `--asset`, a command handles all sources.

`BLENDER_PATH` overrides the Blender executable (the application bundle on macOS, or `blender` on PATH on Linux). Scratch GLBs go to ignored `art/export/`; project-local glTF Transform writes optimized assets and the manifest to `public/assets/`. `node scripts/build-assets.mjs --exports-only` optimizes existing scratch exports. Arena anchors are validated against `config/arena.json` rather than changing gameplay layout silently.

Authoring code has small, separate responsibilities:

- `authoring/characters.py`, `equipment.py`, `geometry.py`: mesh design, materials and weights.
- `authoring/rig.py`: common skeleton and attachment convention.
- `authoring/posing.py`: authoring-time arm/leg IK and anatomical transforms.
- `authoring/motion.py`, `attacks.py`: rig choreography and action baking.
- `authoring/arena.py`: scenery and named arena anchors.
- `build_scene.py`: source loading, optional regeneration and export.

There is no runtime IK, retargeting package or character editor.

## Animation contract

Blender uses Z-up and forward -Y; exported characters are Y-up, forward +Z. First-person assets face camera -Z. Exports retain names, skinning, actions and sockets, omit cameras/lights, and start clips at zero. Poses are baked at 30 Hz plus exact contact/end keys. Linear quaternion interpolation connects the already eased poses; export must preserve fractional contact frames instead of resampling them away.

`config/animation.json` records authored clip lengths, contact markers and travel per gait cycle. `config/game.json` controls weighted attack selection, windup/cooldown multipliers and seeded timing variation. The simulation captures each attack's complete timing when it starts. Presentation retimes its chosen action to that windup and recovery; animation never applies damage.

| Motion | Clips / behavior |
| --- | --- |
| Locomotion | `walk`, `run`, `carry`, `walk_back`, `strafe_left`, `strafe_right`; actual distance after collision determines cycle progress. |
| Mining | `mine`; both hands grip the pick. |
| Sword | `melee_slash`, `melee_backhand`, `melee_overhead`; separate preparation, contact, follow-through and recovery. |
| Bow | `bow_quick`, `bow_draw`, `bow_hold`; different lift/draw/hold timing, flexing limbs, release and reload. |
| Hits | `hit_front`, `hit_back`, `hit_left`, `hit_right`; additive recoil lets walking and attacks continue. |
| Death | `death_front`, `death_back`, `death_left`, `death_right`; impact direction selects the fall, knees buckle and limbs settle against the floor. |

Walking uses a linear planted-foot phase, lifted swing, heel strike and articulated toe-off. The runtime advances cycle phase by travelled distance, including backwards and lateral movement; it stops advancing when obstructed. Separate upper/lower playback layers retain footsteps during an attack. Corpses reuse the already posed skeleton, blend into the fall, slide briefly with the impact and expire within a bounded pool.

Required bones include root, pelvis, spine, head, paired upper arms/forearms/hands/thighs/shins/feet/toes, plus `eye_socket`, `weapon_socket`, `bow_socket`, `arrow_socket`. `bow_upper`, `bow_lower` and `string_nock` deform the bow. The nock belongs to the bow socket so torso blending never detaches the string.

Character IDs are `miner`, `swordsman`, `archer`; commander models append `_lod`. First-person IDs are `fp_swordsman` and `fp_archer`. The swordsman carries a sword with an empty off-hand. Arena markers are `blue_statue`, `blue_mine`, `blue_spawn` and their red counterparts.

The close characters use 5,020–5,714 triangles; commander variants use 1,755–1,766. Each has two main materials and shares the painted atlas at runtime. glTF validation checks budgets, joints, rig-only animation channels, clips, zero-based durations, materials and arena anchors. Browser tests measure actual exported foot planting, bow grip/string alignment and floor contact. See `docs/ANIMATION_QA.md` for this pass's evidence and scope.

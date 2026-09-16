# Blender authoring

The original models were authored with installed **Blender 5.2.2 LTS**. No external models or animation packs are used. `sources/` contains six editable Blender projects: miner, swordsman, archer, arena, and two first-person arm/weapon scenes. `textures/tabletop-palette.png` is the shared 1024 × 1024 painted palette.

## Rebuild

```sh
npm run assets:build
npm run assets:check
```

`BLENDER_PATH` can point to another Blender 5 executable. The default is `/Applications/Blender.app/Contents/MacOS/Blender`. This command regenerates the sources from `build_scene.py`, exports scratch GLBs into ignored `art/export/`, then runs project-local glTF Transform optimization. **Manual edits to a source `.blend` must be exported separately before regenerating**, or translated into the authoring script.

To optimize GLBs already exported manually into `art/export/`:

```sh
node scripts/build-assets.mjs --exports-only
```

Use the export settings in `build_scene.py`: GLB, metres, Y-up, actions, skinning, no cameras/lights. Blender authoring coordinates are Z-up and forward -Y; exported characters face +Z. First-person geometry uses camera coordinates, facing -Z. Feet and the root are at the origin. Arena marker coordinates are exported and compared to `config/arena.json` rather than silently overwriting gameplay layout.

## Stable bindings

- Character IDs: `miner`, `swordsman`, `archer`; lower-detail variants append `_lod`.
- First-person assets: `fp_swordsman`, `fp_archer`; the swordsman carries only a sword.
- Clips: `idle`, `walk`, `carry`, `mine`, `melee_attack`, `bow_attack`, `hit`, `death`. The shared clip set simplifies substitution; only applicable clips are selected for each role.
- Rig: root, pelvis, spine, head, paired upper arms/forearms/hands/thighs/shins/feet.
- Preserved sockets: `eye_socket`, `weapon_socket`, `bow_socket`, `arrow_socket`.
- Materials: `Painted palette` and `Team cloth`. Only the latter is recolored at runtime. Keep material names distinct through optimization.
- Arena markers: `blue_statue`, `blue_mine`, `blue_spawn`, and red counterparts.

Character geometry uses segmented limbs weighted to the shared skeleton, with facial features, layered equipment and in-place animation. Ambient shading is painted into atlas swatches; simple directional lighting adds readable shadows. The runtime reuses one atlas GPU texture across the GLBs. First-person swing/recoil is cosmetic; simulation timing alone decides hits.

The optimizer deduplicates while retaining unique names, welds, resamples animation, prunes while retaining leaf sockets, and uses Meshopt compression. Validation checks all required bindings, clips, joints, material separation, arena anchors, triangle budgets, texture dimensions, manifest byte counts and Khronos GLB errors. `docs/asset-validation.json` records a checked export.

Close models are 3,256–3,624 triangles; commander variants are 1,296–1,429. This leaves headroom below the 8,000-triangle target for later sculpting and animation refinement. The current style is a deliberately economical tabletop treatment, not the full detail of the concept paintings.

# Stick War production art direction

## Visual contract

- High-resolution 2D raster sprites with a clean vector-illustration finish.
- Hand-inked stick figures with subtly imperfect curves and variable line weight.
- Restrained medieval equipment detail so silhouettes remain readable at gameplay scale.
- Blue and red team variants are derived from one approved master design per class.
- No direct copying of an existing game's characters, costumes, UI, or branded artwork.
- Character animation strips use transparent 384 x 384 cells, bottom-center anchoring,
  linear texture filtering, and whole-strip generation to minimize visual drift.
- Backgrounds use broad gradient fields, blocky parallax silhouettes, and hand-drawn edges.
- Time-of-day grading controls tint, saturation, and brightness independently of source art.

## Seed approval checkpoint

Before animation strips are generated, approve these four references:

1. Miner master character
2. Swordsman master character
3. Archer master character
4. Battlefield environment and medieval UI mood

Selected references are stored in `art/reference/`. Production-ready sheets will live
under `assets/characters/`, with previews under `docs/animation-previews/`.

## Animation plan

| Character | Clip | Frames | FPS | Gameplay event |
| --- | --- | ---: | ---: | --- |
| All | Idle | 8 | 12 | — |
| All | Walk | 10 | 16 | — |
| Miner | Walk loaded | 12 | 16 | — |
| Miner | Mine | 12 | 16 | Pickaxe contact: 7 |
| Combat units | Backpedal | 10 | 14 | — |
| Swordsman | Attack | 12 | 20 | Sword contact: 7 |
| Archer | Attack | 14 | 20 | Arrow release: 9 |
| All | Hit | 6 | 20 | — |
| All | Death | 12 | 16 | Despawn after final frame |

## Reference prompts

All character references use built-in image generation with a flat `#00ff00`
chroma-key background, followed by local soft-matte removal and normalization.

### Miner

> Original full-body side-view medieval stick-figure miner facing right. Hunched,
> work-worn silhouette; black hand-inked limbs with variable curved line weight;
> simple warm leather cap and belt; restrained blue cloth sash; heavy iron pickaxe;
> rope-tied ore sack designed to be dragged behind him. High-resolution clean
> vector-like 2D game art with subtle ink imperfections and limited cel shading.

### Swordsman

> Original full-body side-view medieval stick-figure swordsman facing right.
> Athletic black hand-inked figure with curved variable line weight; weathered iron
> helmet, practical broad sword, small round shield, and restrained blue cloth sash.
> Strong readable silhouette, high-resolution vector-like 2D game art, subtle ink
> imperfections, limited cel shading, and equipment that can animate with weight.

### Archer

> Original full-body side-view medieval stick-figure archer facing right. Lean black
> hand-inked figure with curved variable line weight; simple leather hood and bracer;
> restrained blue scarf; longbow, visible bowstring, and compact back quiver. Clear
> draw-and-release silhouette, high-resolution vector-like 2D game art, subtle ink
> imperfections, and limited cel shading.

### Battlefield

> Original polished side-view 2D medieval battlefield environment for a strategy
> game. Wide horizontal composition with layered parallax mountains, distant forest,
> blocky grassy ridge, ochre soil cutaway, winding clouds, and a warm late-afternoon
> gradient sky. Vector-like shapes with hand-drawn inked edges and restrained texture;
> enough contrast for black stick figures and persistent red/blue accents. Include
> visual motifs suitable for carved dark-wood and hammered-iron UI framing, but no
> actual text, HUD, characters, logos, or copied game artwork.

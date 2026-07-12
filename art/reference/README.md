# Approved-reference candidates

These images are the seed-approval checkpoint for the production art pipeline.
They are not yet runtime sprite sheets.

- `characters/miner-seed.png`
- `characters/swordsman-seed.png`
- `characters/archer-seed.png`
- `environment/battlefield-key-art.png`

The references were generated with Codex's built-in image-generation path using
the prompts recorded in `../ART_DIRECTION.md`. Character sources were generated
against a flat green chroma key, processed with the installed `remove_chroma_key.py`
soft-matte/despill workflow, and validated to contain an alpha channel. Disposable
chroma-key intermediates and rejected generations are intentionally not tracked.

After approval, each character seed becomes the identity anchor for whole-strip
generation. Production strips are normalized to fixed 384 x 384 cells with one
shared scale and a bottom-center anchor before red-team palette variants are derived.

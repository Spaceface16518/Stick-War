import source from "../../config/animation.json";

// Shared with Blender authoring and export validation. Contact is a visual
// marker; the simulation's captured windup always decides when damage occurs.
export const animationLibrary = source;
export type AttackClip = keyof typeof source.attacks;

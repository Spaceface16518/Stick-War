import { z } from "zod";
import source from "../../config/game.json";
import arenaSource from "../../config/arena.json";
import { animationLibrary } from "./animation";
const positive = z.number().finite().positive();
const nonnegative = z.number().finite().nonnegative();
const count = z.number().int().nonnegative().max(1000000);
const point = z.object({ x: z.number().finite(), z: z.number().finite() });
const unit = z
  .object({
    asset: z.string(),
    cost: count,
    trainingSeconds: nonnegative,
    health: positive,
    speed: positive,
    damage: nonnegative,
    range: nonnegative,
    cooldown: nonnegative,
    windup: nonnegative,
    activationRange: nonnegative,
    capacity: count,
    miningSeconds: nonnegative,
    attackVariants: z.array(
      z.object({
        clip: z
          .string()
          .refine(
            (clip) => Object.hasOwn(animationLibrary.attacks, clip),
            "Unknown attack clip",
          ),
        weight: positive,
        windupScale: positive,
        cooldownScale: positive,
      }),
    ),
  })
  .refine((u) => u.windup <= u.cooldown, "Windup must fit within cooldown");
export const configSchema = z
  .object({
    seed: z
      .string()
      .regex(/^\d+$/)
      .refine(
        (s) => BigInt(s) > 0n && BigInt(s) < 1n << 64n,
        "Seed must be a nonzero unsigned 64-bit integer",
      ),
    economy: z.object({
      startingGold: count,
      startingMiners: count,
      populationCap: count.min(1),
      passiveGold: count,
      passiveSeconds: positive,
    }),
    units: z.object({ miner: unit, swordsman: unit, archer: unit }),
    statueHealth: positive,
    arrow: z.object({
      speed: positive,
      gravity: nonnegative,
      lifetime: positive,
      radius: positive,
      speedVariation: nonnegative.max(0.5),
      verticalVariation: nonnegative,
      velocitySmoothing: nonnegative.max(1),
    }),
    combat: z.object({
      cooldownJitter: nonnegative,
      cooldownExtra: nonnegative,
      meleeArcDegrees: positive.max(180),
      timingVariation: nonnegative.max(0.25),
    }),
    ai: z.object({
      recruitSeconds: positive,
      desiredMiners: count,
      swordsPerArcher: count.min(1),
      defenseRadius: positive,
      defenseOffset: nonnegative,
      archerPreferred: positive.max(1),
      archerBackpedal: positive.max(1),
    }),
    formation: z.object({
      columns: count.min(1),
      spacing: positive,
      swordOffset: nonnegative,
      archerOffset: nonnegative,
      retreatOffset: nonnegative,
      spawnOffset: nonnegative,
      mineReturnOffset: nonnegative,
    }),
    camera: z.object({
      panSpeed: positive,
      viewHeight: positive,
      minViewHeight: positive,
      maxViewHeight: positive,
      eyeHeight: positive,
      fov: positive.min(40).max(100),
    }),
    presentation: z.object({ corpseSeconds: positive, maxCorpses: count }),
  })
  .refine(
    (c) =>
      c.camera.minViewHeight <= c.camera.viewHeight &&
      c.camera.viewHeight <= c.camera.maxViewHeight,
    "Camera zoom bounds are inconsistent",
  )
  .refine(
    (c) => c.economy.startingMiners <= c.economy.populationCap,
    "Starting miners exceed population cap",
  )
  .refine(
    (c) =>
      (["swordsman", "archer"] as const).every((kind) => {
        const u = c.units[kind];
        return (
          u.attackVariants.length > 0 &&
          u.attackVariants.every(
            (variant) =>
              variant.clip.startsWith(kind === "archer" ? "bow_" : "melee_") &&
              u.windup * variant.windupScale <=
                u.cooldown * variant.cooldownScale,
          )
        );
      }),
    "Attack variants must match the unit and fit windup within cooldown",
  );
const anchors = z.object({ statue: point, mine: point, spawn: point });
export const arenaSchema = z
  .object({
    id: z.string(),
    name: z.string(),
    halfLength: positive,
    halfWidth: positive,
    teams: z.object({ blue: anchors, red: anchors }),
  })
  .refine(
    (a) =>
      Object.values(a.teams).every((t) =>
        Object.values(t).every(
          (p) => Math.abs(p.x) < a.halfLength && Math.abs(p.z) < a.halfWidth,
        ),
      ),
    "Arena anchors must be within bounds",
  );
export type GameConfig = z.infer<typeof configSchema>;
export type UnitDefinition = GameConfig["units"]["miner"];
export type ArenaDefinition = z.infer<typeof arenaSchema>;
export const defaultConfig = configSchema.parse(source);
export const defaultArena = arenaSchema.parse(arenaSource);
export function parseConfig(value: unknown): GameConfig {
  return configSchema.parse(value);
}
export function watchConfig(
  onValid: (config: GameConfig) => void,
  onError: (message: string) => void,
): void {
  if (import.meta.hot)
    import.meta.hot.accept("../../config/game.json", (module) => {
      try {
        onValid(parseConfig(module?.default));
        onError("");
      } catch (error) {
        onError(`Balance reload rejected: ${String(error)}`);
      }
    });
}

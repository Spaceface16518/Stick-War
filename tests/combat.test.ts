import { beforeAll, describe, expect, it } from "vitest";
import { defaultArena, defaultConfig } from "../src/content/config";
import { tickCombat } from "../src/simulation/combat";
import { acquireTargets, tickMovement } from "../src/simulation/movement";
import { spawnUnit, tickEconomy } from "../src/simulation/economy";
import { CombatRandom } from "../src/simulation/random";
import {
  neutralInput,
  type Team,
  type UnitKind,
} from "../src/simulation/types";
import type { BattleWorld } from "../src/simulation/world";
import { initializePhysics, RapierSpatial } from "../src/spatial/rapier";
beforeAll(initializePhysics);
function world(): BattleWorld {
  return {
    config: structuredClone(defaultConfig),
    arena: defaultArena,
    spatial: new RapierSpatial(defaultArena),
    random: new CombatRandom(defaultConfig.seed),
    mode: "skirmish",
    units: new Map(),
    statues: new Map(),
    projectiles: new Map(),
    training: [],
    teams: {
      blue: { gold: 10000, order: "attack" },
      red: { gold: 10000, order: "attack" },
    },
    settings: { costs: true, trainingTime: false, populationCap: 20 },
    controlledId: null,
    paused: false,
    outcome: null,
    elapsed: 0,
    tick: 0,
    nextId: 10,
    incomeTime: 0,
    recruitTime: 0,
    events: [],
    input: { ...neutralInput },
  };
}
function soldier(w: BattleWorld, team: Team, kind: UnitKind, x: number) {
  const id = spawnUnit(w, team, kind),
    u = w.units.get(id)!;
  u.x = x;
  u.z = 0;
  u.previous = { x, z: 0 };
  return u;
}
const sync = (w: BattleWorld) =>
  w.spatial.sync([...w.units.values()], [...w.statues.values()]);
function battle(w: BattleWorld, frames: number) {
  for (let i = 0; i < frames; i++) {
    sync(w);
    tickCombat(w, 1 / 60);
  }
}
describe("combat resolution", () => {
  it("keeps a backpedaling archer facing its target", () => {
    const w = world();
    w.mode = "sandbox";
    const archer = soldier(w, "blue", "archer", 0);
    soldier(w, "red", "swordsman", 2);
    sync(w);
    acquireTargets(w);
    tickMovement(w, 1 / 60);
    expect(archer.x).toBeLessThan(0);
    expect(archer.yaw).toBeCloseTo(Math.PI / 2);
    w.spatial.dispose();
  });
  it.each(["skirmish", "sandbox"] as const)(
    "aggregates simultaneous statue damage in %s",
    (mode) => {
      const w = world();
      w.mode = mode;
      for (const [id, team, x] of [
        [1, "blue", -4],
        [2, "red", 4],
      ] as const)
        w.statues.set(id, { id, team, x, z: 0, health: 15, maxHealth: 15 });
      for (const [id, team, vx] of [
        [3, "blue", 300],
        [4, "red", -300],
      ] as const)
        w.projectiles.set(id, {
          id,
          team,
          owner: 99,
          x: 0,
          y: 1.4,
          z: 0,
          previous: { x: 0, y: 1.4, z: 0 },
          velocity: { x: vx, y: 0, z: 0 },
          gravity: 0,
          remaining: 1,
          radius: 0.1,
          damage: 15,
        });
      battle(w, 1);
      expect([...w.statues.values()].map((s) => s.health)).toEqual([0, 0]);
      expect(w.outcome).toBe(mode === "skirmish" ? "draw" : null);
      w.spatial.dispose();
    },
  );
  it("manual swords respect facing and hit only the closest hostile", () => {
    const w = world();
    w.mode = "sandbox";
    const blue = soldier(w, "blue", "swordsman", 0);
    const near = soldier(w, "red", "swordsman", 0.8),
      far = soldier(w, "red", "swordsman", 1.2);
    w.controlledId = blue.id;
    w.input = { ...neutralInput, attack: true };
    blue.yaw = -Math.PI / 2;
    battle(w, 50);
    expect(near.health).toBe(100);
    blue.yaw = Math.PI / 2;
    battle(w, 50);
    expect(near.health).toBe(80);
    expect(far.health).toBe(100);
    w.spatial.dispose();
  });
  it("manual bows fire ballistic projectiles and retain attack parameters on reload", () => {
    const w = world();
    w.mode = "sandbox";
    const blue = soldier(w, "blue", "archer", 0),
      red = soldier(w, "red", "swordsman", 4);
    w.controlledId = blue.id;
    w.input = { ...neutralInput, attack: true, pitch: 0.1 };
    battle(w, 1);
    w.config.units.archer.damage = 999;
    w.config.arrow.speed = 1000;
    w.config.arrow.gravity = 1000;
    w.input.attack = false;
    battle(w, 40);
    expect(red.health).toBe(85);
    w.spatial.dispose();
  });
  it("clears possession when a controlled unit dies", () => {
    const w = world();
    w.mode = "sandbox";
    const blue = soldier(w, "blue", "swordsman", 0);
    w.controlledId = blue.id;
    w.projectiles.set(90, {
      id: 90,
      team: "red",
      owner: 91,
      x: -2,
      y: 1,
      z: 0,
      previous: { x: -2, y: 1, z: 0 },
      velocity: { x: 300, y: 0, z: 0 },
      gravity: 0,
      remaining: 1,
      radius: 0.1,
      damage: 100,
    });
    battle(w, 1);
    expect(w.controlledId).toBeNull();
    expect(w.units.has(blue.id)).toBe(false);
    w.spatial.dispose();
  });
  it("recruits two swords per archer and replaces missing miners first", () => {
    const w = world();
    for (let n = 0; n < 5; n++) tickEconomy(w, 2);
    expect(
      w.events.filter((e) => e.type === "spawn").map((e) => e.kind),
    ).toEqual(["miner", "miner", "swordsman", "swordsman", "archer"]);
    const miner = [...w.units.values()].find((u) => u.kind === "miner")!;
    w.units.delete(miner.id);
    tickEconomy(w, 2);
    expect(
      [...w.units.values()].filter((u) => u.kind === "miner"),
    ).toHaveLength(2);
    expect(w.events.at(-1)).toMatchObject({ type: "spawn", kind: "miner" });
    w.spatial.dispose();
  });
});

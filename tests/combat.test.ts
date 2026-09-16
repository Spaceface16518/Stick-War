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
    w.elapsed += 1 / 60;
    w.tick++;
    sync(w);
    tickCombat(w, 1 / 60);
  }
}
describe("combat resolution", () => {
  it.each(["blue", "red"] as const)(
    "%s defenders turn back to protect the statue beyond their personal activation range",
    (team) => {
      const w = world();
      const sign = team === "blue" ? 1 : -1,
        enemy = team === "blue" ? "red" : "blue";
      w.teams[team].order = "defend";
      const defender = soldier(w, team, "swordsman", -17.4 * sign);
      const attacker = soldier(w, enemy, "swordsman", -26.5 * sign);
      w.statues.set(1, {
        id: 1,
        team,
        x: -28 * sign,
        z: 0,
        health: 100,
        maxHealth: 500,
      });
      attacker.target = 1;
      sync(w);
      acquireTargets(w);
      expect(defender.target).toBe(attacker.id);
      tickMovement(w, 1 / 60);
      expect(defender.x * sign).toBeLessThan(-17.4);
      w.spatial.dispose();
    },
  );
  it("defenders drop a fleeing target beyond their half and return to formation", () => {
    const w = world();
    w.teams.blue.order = "defend";
    const defender = soldier(w, "blue", "swordsman", -0.2);
    const enemy = soldier(w, "red", "archer", 3);
    defender.target = enemy.id;
    sync(w);
    acquireTargets(w);
    tickMovement(w, 1 / 60);
    expect(defender.target).toBeNull();
    expect(defender.x).toBeLessThan(-0.2);
    w.spatial.dispose();
  });
  it("defend recalls an archer from the enemy half instead of continuing its assault", () => {
    const w = world();
    w.teams.blue.order = "defend";
    const defender = soldier(w, "blue", "archer", 5);
    const enemy = soldier(w, "red", "archer", 12);
    defender.target = enemy.id;
    sync(w);
    acquireTargets(w);
    tickMovement(w, 1 / 60);
    expect(defender.target).toBeNull();
    expect(defender.x).toBeLessThan(5);
    w.spatial.dispose();
  });
  it("archers react to nearby melee pressure instead of keeping a distant ranged target", () => {
    const w = world();
    const archer = soldier(w, "blue", "archer", 0);
    const distant = soldier(w, "red", "archer", 12);
    const sword = soldier(w, "red", "swordsman", 3);
    archer.target = distant.id;
    acquireTargets(w);
    expect(archer.target).toBe(sword.id);
    w.spatial.dispose();
  });
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

describe("varied attack timing and physical reactions", () => {
  it.each(["swordsman", "archer"] as const)(
    "reproduces %s styles and timing from the full seed",
    (kind) => {
      const sequence = (seed: string) => {
        const w = world();
        w.mode = "sandbox";
        w.random = new CombatRandom(seed);
        const u = soldier(w, "blue", kind, 0);
        w.controlledId = u.id;
        w.input.attack = true;
        const motions = [];
        let stamp = -1;
        for (let tick = 0; tick < 3000; tick++) {
          battle(w, 1);
          if (u.attackMotion && u.attackMotion.startedAt !== stamp) {
            stamp = u.attackMotion.startedAt;
            motions.push({ ...u.attackMotion });
          }
        }
        w.spatial.dispose();
        return motions;
      };
      const first = sequence("18446744073709551601");
      expect(first).toEqual(sequence("18446744073709551601"));
      expect(first).not.toEqual(sequence("18446744073709551602"));
      expect(new Set(first.map((m) => m.clip)).size).toBe(3);
      expect(new Set(first.map((m) => m.windup)).size).toBeGreaterThan(10);
    },
  );
  for (const kind of ["swordsman", "archer"] as const)
    for (const variant of defaultConfig.units[kind].attackVariants) {
      it(`${variant.clip} resolves at its captured windup and retains timing after reload`, () => {
        const w = world();
        w.mode = "sandbox";
        w.config.combat.timingVariation = 0;
        w.config.combat.cooldownJitter = 0;
        w.config.units[kind].attackVariants = [variant];
        const u = soldier(w, "blue", kind, 0),
          enemy = soldier(w, "red", "swordsman", 0.8);
        u.yaw = Math.PI / 2;
        w.controlledId = u.id;
        w.input = { ...neutralInput, yaw: Math.PI / 2, attack: true };
        battle(w, 1);
        w.input.attack = false;
        const captured = { ...u.attackMotion! };
        expect(captured.windup).toBeCloseTo(
          defaultConfig.units[kind].windup * variant.windupScale,
        );
        expect(captured.duration).toBeCloseTo(
          defaultConfig.units[kind].cooldown * variant.cooldownScale +
            w.config.combat.cooldownExtra,
        );
        w.config.units[kind].windup = 8;
        w.config.units[kind].cooldown = 10;
        const beforeContact = Math.ceil(captured.windup * 60) - 1;
        battle(w, beforeContact);
        expect(u.attack).not.toBeNull();
        expect(enemy.health).toBe(100);
        expect(w.events.filter((e) => e.type === "shot")).toHaveLength(0);
        battle(w, 1);
        expect(u.attack).toBeNull();
        expect(u.attackMotion).toEqual(captured);
        expect(w.elapsed - captured.startedAt).toBeGreaterThanOrEqual(
          captured.windup - 1e-8,
        );
        expect(w.elapsed - captured.startedAt).toBeLessThan(
          captured.windup + 1 / 60 + 1e-8,
        );
        if (kind === "archer")
          expect(w.events.filter((e) => e.type === "shot")).toHaveLength(1);
        else expect(enemy.health).toBe(80);
        w.spatial.dispose();
      });
    }
  it("captures the weighted direction of simultaneous damage in the death event", () => {
    const w = world();
    w.mode = "sandbox";
    const u = soldier(w, "blue", "swordsman", 0);
    for (const [id, x, z, vx, vz, damage] of [
      [90, -2, 0, 300, 0, 70],
      [91, 0, -2, 0, 300, 30],
    ]) {
      w.projectiles.set(id, {
        id,
        team: "red",
        owner: 99,
        x,
        y: 1,
        z,
        previous: { x, y: 1, z },
        velocity: { x: vx, y: 0, z: vz },
        gravity: 0,
        remaining: 1,
        radius: 0.1,
        damage,
      });
    }
    battle(w, 1);
    const event = w.events.find((e) => e.type === "death");
    expect(event?.type).toBe("death");
    if (event?.type === "death") {
      expect(event.unit.hitMotion!.direction.x).toBeCloseTo(
        70 / Math.hypot(70, 30),
      );
      expect(event.unit.hitMotion!.direction.z).toBeCloseTo(
        30 / Math.hypot(70, 30),
      );
      expect(event.unit.hitMotion!.strength).toBe(1);
    }
    expect(w.units.has(u.id)).toBe(false);
    w.spatial.dispose();
  });
});

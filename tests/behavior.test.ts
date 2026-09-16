import { beforeAll, describe, expect, it } from "vitest";
import { defaultArena, defaultConfig } from "../src/content/config";
import { BattleSimulation } from "../src/simulation/battle";
import { initializePhysics, RapierSpatial } from "../src/spatial/rapier";
import {
  direction,
  opponent,
  type ArmyOrder,
  type Team,
  type UnitKind,
} from "../src/simulation/types";

beforeAll(initializePhysics);
function sandbox() {
  const config = structuredClone(defaultConfig);
  config.economy.startingMiners = 0;
  return new BattleSimulation(
    config,
    defaultArena,
    "sandbox",
    new RapierSpatial(defaultArena),
  );
}
function train(s: BattleSimulation, team: Team, kind: UnitKind, count = 1) {
  for (let n = 0; n < count; n++)
    expect(s.command({ type: "train", team, kind }).ok).toBe(true);
}
function order(s: BattleSimulation, team: Team, order: ArmyOrder) {
  s.command({ type: "order", team, order });
}
function advance(s: BattleSimulation, seconds: number) {
  for (let n = 0; n < seconds * 60; n++) s.step();
}

describe("sandbox battle scenarios", () => {
  it.each(["blue", "red"] as const)(
    "three %s defending archers return fire against three attacking archers",
    (defender) => {
      const s = sandbox(),
        attacker = opponent(defender);
      train(s, defender, "archer", 3);
      train(s, attacker, "archer", 3);
      order(s, defender, "defend");
      order(s, attacker, "attack");
      s.command({ type: "pause", paused: false });
      const initial = s.snapshot();
      const teams = new Map(initial.units.map((u) => [u.id, u.team]));
      s.events();
      const shots = { blue: 0, red: 0 };
      for (let n = 0; n < 60; n++) {
        advance(s, 1);
        for (const e of s.events())
          if (e.type === "shot") shots[teams.get(e.id)!]++;
        if (shots.blue && shots.red) break;
      }
      expect(shots[attacker]).toBeGreaterThan(0);
      expect(shots[defender]).toBeGreaterThan(0);
      s.dispose();
    },
  );

  it.each(["blue", "red"] as const)(
    "new %s swordsmen relieve a statue under siege",
    (defender) => {
      const s = sandbox(),
        attacker = opponent(defender);
      train(s, attacker, "swordsman", 3);
      order(s, attacker, "attack");
      s.command({ type: "pause", paused: false });
      for (let n = 0; n < 40; n++) {
        advance(s, 1);
        if (s.snapshot().statues.find((t) => t.team === defender)!.health < 500)
          break;
      }
      expect(
        s.snapshot().statues.find((t) => t.team === defender)!.health,
      ).toBeLessThan(500);
      const invaders = s
        .snapshot()
        .units.filter((u) => u.team === attacker)
        .map((u) => u.id);
      order(s, defender, "defend");
      train(s, defender, "swordsman", 3);
      const reinforcements = s
        .snapshot()
        .units.filter((u) => u.team === defender);
      expect(
        reinforcements.every(
          (u) =>
            (u.x - defaultArena.teams[defender].statue.x) *
              direction(defender) <
            0,
        ),
      ).toBe(true);
      s.events();
      advance(s, 9);
      expect(
        s.events().some((e) => e.type === "damage" && invaders.includes(e.id)),
      ).toBe(true);
      s.dispose();
    },
  );

  it.each(["archer", "swordsman"] as const)(
    "defending %ss hold home when neither army advances",
    (kind) => {
      const s = sandbox();
      for (const team of ["blue", "red"] as const) {
        train(s, team, kind, 3);
        order(s, team, "defend");
      }
      s.command({ type: "pause", paused: false });
      advance(s, 60);
      expect(s.events().some((e) => e.type === "attack")).toBe(false);
      for (const u of s.snapshot().units)
        expect(u.x * direction(u.team)).toBeLessThan(-13);
      s.dispose();
    },
  );
});

describe("orders and deployment under pressure", () => {
  it.each([
    ["swordsman", "swordsman"],
    ["archer", "swordsman"],
    ["swordsman", "archer"],
    ["archer", "archer"],
  ] as const)(
    "%s attackers engage %s defenders instead of idling",
    (attackingKind, defendingKind) => {
      const s = sandbox();
      train(s, "blue", defendingKind, 3);
      train(s, "red", attackingKind, 3);
      order(s, "blue", "defend");
      order(s, "red", "attack");
      s.command({ type: "pause", paused: false });
      advance(s, 10);
      const home = s.snapshot().units.filter((u) => u.team === "blue");
      let intercepted = false;
      for (let n = 0; n < 38; n++) {
        advance(s, 1);
        intercepted ||= s
          .snapshot()
          .units.some(
            (u) =>
              u.team === "blue" &&
              u.target !== null &&
              u.x > home.find((h) => h.id === u.id)!.x + 2,
          );
      }
      const events = s.events();
      const blueIds = home.map((u) => u.id);
      expect(
        events.some((e) => e.type === "attack" && blueIds.includes(e.id)) ||
          intercepted,
      ).toBe(true);
      expect(events.some((e) => e.type === "damage")).toBe(true);
      s.dispose();
    },
  );

  it("mixed defending armies disengage, retreat and regroup without attacking during withdrawal", () => {
    const s = sandbox();
    for (const team of ["blue", "red"] as const) {
      train(s, team, "swordsman", 4);
      train(s, team, "archer", 2);
      order(s, team, "attack");
    }
    s.command({ type: "pause", paused: false });
    advance(s, 17);
    const ids = s
      .snapshot()
      .units.filter((u) => u.team === "blue")
      .map((u) => u.id);
    order(s, "blue", "retreat");
    expect(
      s
        .snapshot()
        .units.filter((u) => u.team === "blue")
        .every((u) => u.attack === null && u.target === null),
    ).toBe(true);
    s.events();
    advance(s, 2);
    expect(
      s.events().some((e) => e.type === "attack" && ids.includes(e.id)),
    ).toBe(false);
    order(s, "red", "retreat");
    advance(s, 25);
    for (const u of s.snapshot().units)
      expect(u.x * direction(u.team)).toBeLessThan(-29);
    order(s, "blue", "defend");
    order(s, "red", "defend");
    advance(s, 25);
    for (const u of s.snapshot().units) {
      expect(u.target).toBeNull();
      expect(u.x * direction(u.team)).toBeGreaterThan(-27);
      expect(u.x * direction(u.team)).toBeLessThan(-16);
    }
    s.dispose();
  });

  it("100-unit paused deployment avoids statues and overlapping spawn slots", () => {
    const s = sandbox();
    s.command({ type: "sandbox", populationCap: 50 });
    for (const team of ["blue", "red"] as const)
      train(s, team, "swordsman", 50);
    const state = s.snapshot();
    for (const u of state.units) {
      for (const statue of state.statues)
        expect(
          Math.abs(u.x - statue.x) >= 1.4 || Math.abs(u.z - statue.z) >= 1.4,
        ).toBe(true);
      for (const other of state.units)
        if (u.id !== other.id)
          expect(
            Math.hypot(u.x - other.x, u.z - other.z),
          ).toBeGreaterThanOrEqual(0.749);
    }
    s.dispose();
  });
});

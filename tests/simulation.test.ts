import { beforeAll, describe, expect, it } from "vitest";
import {
  defaultArena,
  defaultConfig,
  parseConfig,
  type ArenaDefinition,
} from "../src/content/config";
import { BattleSimulation } from "../src/simulation/battle";
import { CombatRandom } from "../src/simulation/random";
import { initializePhysics, RapierSpatial } from "../src/spatial/rapier";
import { neutralInput, type UnitState } from "../src/simulation/types";
beforeAll(initializePhysics);
const advance = (sim: BattleSimulation, seconds: number) => {
  for (let i = 0; i < seconds * 60; i++) sim.step();
};
function sandbox() {
  return new BattleSimulation(
    defaultConfig,
    defaultArena,
    "sandbox",
    new RapierSpatial(defaultArena),
  );
}
function tinyArena(): ArenaDefinition {
  return {
    ...defaultArena,
    halfLength: 8,
    teams: {
      blue: {
        statue: { x: -5, z: 0 },
        mine: { x: -2, z: 1.6 },
        spawn: { x: -3.3, z: 0 },
      },
      red: {
        statue: { x: 5, z: 0 },
        mine: { x: 2, z: -1.6 },
        spawn: { x: 3.3, z: 0 },
      },
    },
  };
}
describe("battle rules", () => {
  it("returns an interrupted miner to the deposit before restarting work", () => {
    const s = sandbox();
    s.command({ type: "pause", paused: false });
    advance(s, 5);
    expect(s.snapshot().units.find((u) => u.team === "blue")!.minerState).toBe(
      "mining",
    );
    s.command({ type: "order", team: "blue", order: "retreat" });
    advance(s, 2);
    s.command({ type: "order", team: "blue", order: "defend" });
    advance(s, 1.2);
    const miner = s.snapshot().units.find((u) => u.team === "blue")!;
    expect(miner.minerState).toBe("outbound");
    expect(miner.carried).toBe(0);
    s.dispose();
  });
  it("can win with unchanged balance through economy, defense and a counterattack", () => {
    const s = new BattleSimulation(
      defaultConfig,
      defaultArena,
      "skirmish",
      new RapierSpatial(defaultArena),
    );
    let attacking = false;
    for (let n = 0; n < 240 && !s.snapshot().outcome; n++) {
      const state = s.snapshot();
      const count = (kind: "miner" | "swordsman" | "archer") =>
        state.units.filter((u) => u.team === "blue" && u.kind === kind).length +
        state.training.filter((t) => t.team === "blue" && t.kind === kind)
          .length;
      const kind =
        count("miner") < 2
          ? "miner"
          : count("swordsman") >= (count("archer") + 1) * 2
            ? "archer"
            : "swordsman";
      s.command({ type: "train", team: "blue", kind });
      if (count("swordsman") + count("archer") >= 7) attacking = true;
      s.command({
        type: "order",
        team: "blue",
        order: attacking ? "attack" : "defend",
      });
      advance(s, 2);
    }
    expect(s.snapshot().outcome).toBe("victory");
    s.dispose();
  });
  it("mirrors miners and carried resources throughout their first cycle", () => {
    const s = sandbox();
    s.command({ type: "pause", paused: false });
    for (let n = 0; n < 720; n++) {
      s.step();
      const [blue, red] = s.snapshot().units;
      expect(blue.x).toBeCloseTo(-red.x, 4);
      expect(blue.z).toBeCloseTo(-red.z, 4);
      expect(blue.carried).toBe(red.carried);
    }
    s.dispose();
  });
  it("retreats all four formation columns around the home statue", () => {
    const s = sandbox();
    for (let n = 0; n < 4; n++)
      s.command({ type: "train", team: "blue", kind: "swordsman" });
    s.command({ type: "order", team: "blue", order: "retreat" });
    s.command({ type: "pause", paused: false });
    advance(s, 20);
    for (const u of s.snapshot().units.filter((u) => u.team === "blue"))
      expect(u.x).toBeLessThan(-29.1);
    s.dispose();
  });
  it("starts paused, trains immediately and keeps slots/costs symmetric", () => {
    const s = sandbox();
    expect(s.snapshot().paused).toBe(true);
    for (const team of ["blue", "red"] as const) {
      expect(s.command({ type: "train", team, kind: "swordsman" }).ok).toBe(
        true,
      );
      expect(s.population(team)).toBe(2);
      expect(s.snapshot().teams[team].gold).toBe(150);
    }
    advance(s, 3);
    expect(s.snapshot().elapsed).toBe(0);
    s.dispose();
  });
  it("reserves population and money and rejects duplicate training", () => {
    const s = sandbox();
    s.command({
      type: "sandbox",
      costs: true,
      trainingTime: true,
      populationCap: 2,
    });
    expect(
      s.command({ type: "train", team: "blue", kind: "swordsman" }).ok,
    ).toBe(true);
    expect(s.snapshot().teams.blue.gold).toBe(50);
    expect(s.population("blue")).toBe(2);
    expect(
      s.command({ type: "train", team: "blue", kind: "swordsman" }).ok,
    ).toBe(false);
    expect(s.command({ type: "train", team: "blue", kind: "miner" }).ok).toBe(
      false,
    );
    advance(s, 6);
    expect(s.snapshot().training[0].remaining).toBe(5);
    s.command({ type: "pause", paused: false });
    advance(s, 5);
    expect(s.snapshot().training).toHaveLength(0);
    expect(s.population("blue")).toBe(2);
    s.dispose();
  });
  it("rejects unaffordable purchases without changing population", () => {
    const s = sandbox();
    s.command({ type: "sandbox", costs: true });
    s.command({ type: "train", team: "blue", kind: "archer" });
    expect(s.command({ type: "train", team: "blue", kind: "miner" }).ok).toBe(
      false,
    );
    expect(s.population("blue")).toBe(2);
    s.dispose();
  });
  it("pays passive income to both teams", () => {
    const s = sandbox();
    s.command({ type: "pause", paused: false });
    advance(s, 2);
    expect(s.snapshot().teams.blue.gold).toBe(155);
    expect(s.snapshot().teams.red.gold).toBe(155);
    s.dispose();
  });
  it("miners bring gold home symmetrically", () => {
    const s = sandbox();
    s.command({ type: "pause", paused: false });
    advance(s, 20);
    expect(s.snapshot().teams.blue.gold).toBeGreaterThan(200);
    expect(s.snapshot().teams.blue.gold).toBe(s.snapshot().teams.red.gold);
    s.dispose();
  });
  it("retreat interrupts mining and retains carried gold", () => {
    const s = sandbox();
    s.command({ type: "pause", paused: false });
    advance(s, 6);
    const u = s.snapshot().units.find((u) => u.team === "blue")!;
    expect(u.carried).toBe(25);
    s.command({ type: "order", team: "blue", order: "retreat" });
    advance(s, 1);
    expect(s.snapshot().units.find((x) => x.id === u.id)!.carried).toBe(25);
    s.dispose();
  });
  it("keeps already reserved training when cap is lowered and toggles change", () => {
    const s = sandbox();
    s.command({ type: "sandbox", trainingTime: true });
    s.command({ type: "train", team: "blue", kind: "archer" });
    s.command({ type: "sandbox", trainingTime: false, populationCap: 1 });
    expect(s.snapshot().training[0].duration).toBe(7);
    s.command({ type: "pause", paused: false });
    advance(s, 7);
    expect(s.population("blue")).toBe(2);
    expect(s.command({ type: "train", team: "blue", kind: "miner" }).ok).toBe(
      false,
    );
    s.dispose();
  });
  it("possession suppresses autonomous orders and preserves cooldown through switching", () => {
    const s = sandbox();
    s.command({ type: "train", team: "blue", kind: "archer" });
    const id = s.snapshot().units.find((u) => u.kind === "archer")!.id;
    s.command({ type: "possess", id });
    s.command({ type: "order", team: "blue", order: "attack" });
    s.command({ type: "pause", paused: false });
    const start = s.snapshot().units.find((u) => u.id === id)!;
    advance(s, 1);
    expect(s.snapshot().units.find((u) => u.id === id)!.x).toBe(start.x);
    s.setInput({ ...neutralInput, attack: true });
    s.step();
    const cooldown = s.snapshot().units.find((u) => u.id === id)!.cooldown;
    expect(cooldown).toBeGreaterThan(1);
    s.command({ type: "release" });
    s.command({ type: "possess", id });
    expect(s.snapshot().units.find((u) => u.id === id)!.cooldown).toBe(
      cooldown,
    );
    expect(s.snapshot().units.find((u) => u.id === id)!.attack).toBeNull();
    s.dispose();
  });
  it("rejects miners and enemy possession in skirmish", () => {
    const s = new BattleSimulation(
      defaultConfig,
      defaultArena,
      "skirmish",
      new RapierSpatial(defaultArena),
    );
    expect(
      s.command({ type: "possess", id: s.snapshot().units[0].id }).ok,
    ).toBe(false);
    expect(
      s.command({ type: "train", team: "red", kind: "swordsman" }).ok,
    ).toBe(false);
    s.dispose();
  });
  it("AI recruits miners then swordsmen", () => {
    const s = new BattleSimulation(
      defaultConfig,
      defaultArena,
      "skirmish",
      new RapierSpatial(defaultArena),
    );
    advance(s, 8);
    expect(
      s.snapshot().units.filter((u) => u.team === "red" && u.kind === "miner"),
    ).toHaveLength(2);
    expect(
      s
        .snapshot()
        .training.some((t) => t.team === "red" && t.kind === "swordsman") ||
        s
          .snapshot()
          .units.some((u) => u.team === "red" && u.kind === "swordsman"),
    ).toBe(true);
    s.dispose();
  });
  it("attack advances, defend holds, retreat moves home", () => {
    const s = sandbox();
    s.command({ type: "train", team: "blue", kind: "swordsman" });
    s.command({ type: "pause", paused: false });
    advance(s, 5);
    let u = s.snapshot().units.find((u) => u.kind === "swordsman")!;
    const defended = u.x;
    s.command({ type: "order", team: "blue", order: "attack" });
    advance(s, 2);
    u = s.snapshot().units.find((x) => x.id === u.id)!;
    expect(u.x).toBeGreaterThan(defended + 3);
    s.command({ type: "order", team: "blue", order: "retreat" });
    advance(s, 2);
    expect(s.snapshot().units.find((x) => x.id === u.id)!.x).toBeLessThan(
      u.x - 3,
    );
    s.dispose();
  });
  it("runs melee, ranged combat, deaths and a complete victory", () => {
    const c = structuredClone(defaultConfig);
    c.economy.startingMiners = 0;
    c.economy.startingGold = 1000;
    c.ai.recruitSeconds = 999;
    c.units.swordsman.trainingSeconds = 0;
    c.units.archer.trainingSeconds = 0;
    c.statueHealth = 80;
    const a = tinyArena();
    const s = new BattleSimulation(c, a, "skirmish", new RapierSpatial(a));
    s.command({ type: "train", team: "blue", kind: "swordsman" });
    s.command({ type: "train", team: "blue", kind: "archer" });
    s.command({ type: "order", team: "blue", order: "attack" });
    advance(s, 35);
    expect(s.snapshot().outcome).toBe("victory");
    expect(s.events().some((e) => e.type === "shot")).toBe(true);
    s.dispose();
  });
  it("retains active training parameters during a valid balance reload", () => {
    const s = sandbox();
    s.command({ type: "sandbox", trainingTime: true });
    s.command({ type: "train", team: "blue", kind: "swordsman" });
    const c = structuredClone(defaultConfig);
    c.units.swordsman.trainingSeconds = 1;
    s.reloadConfig(c);
    expect(s.snapshot().training[0].duration).toBe(5);
    s.dispose();
  });
  it("disposes every collider and permits repeated clean sessions", () => {
    for (let n = 0; n < 5; n++) {
      const s = sandbox();
      expect(s.diagnostics().colliders).toBe(9);
      s.command({ type: "train", team: "blue", kind: "swordsman" });
      s.dispose();
      expect(s.diagnostics().colliders).toBe(0);
      expect(s.diagnostics().units).toBe(0);
      s.dispose();
    }
  });
});
describe("spatial invariants", () => {
  const unit = (id: number, x: number, team: "blue" | "red"): UnitState => ({
    id,
    x,
    z: 0,
    previous: { x, z: 0 },
    team,
    kind: "swordsman",
    health: 100,
    maxHealth: 100,
    yaw: 0,
    cooldown: 0,
    attack: null,
    phase: "idle",
    carried: 0,
    miningRemaining: 0,
    minerState: "outbound",
    target: null,
    velocity: { x: 0, z: 0 },
    hitRemaining: 0,
  });
  it("sweeps fast arrows past friendlies to the first hostile impact", () => {
    const p = new RapierSpatial(defaultArena);
    p.sync(
      [
        unit(1, 0, "blue"),
        unit(2, 2, "blue"),
        unit(3, 4, "red"),
        unit(4, 7, "red"),
      ],
      [],
    );
    const hit = p.sweep(
      { x: 0, y: 1, z: 0 },
      { x: 20, y: 1, z: 0 },
      0.1,
      "blue",
      1,
    );
    expect(hit?.id).toBe(3);
    expect(hit!.fraction).toBeLessThan(0.2);
    p.dispose();
  });
  it("world obstacles stop projectiles before units behind them", () => {
    const p = new RapierSpatial(defaultArena);
    p.sync([unit(3, 4, "red")], []);
    expect(
      p.sweep({ x: 0, y: 1, z: 0 }, { x: 10, y: -10, z: 0 }, 0.1, "blue", 1)
        ?.id,
    ).toBeNull();
    p.dispose();
  });
});
describe("configuration", () => {
  it("rejects invalid reloads and imprecise numeric seeds", () => {
    expect(() =>
      parseConfig({ ...defaultConfig, seed: Number(defaultConfig.seed) }),
    ).toThrow();
    expect(() =>
      parseConfig({
        ...defaultConfig,
        economy: { ...defaultConfig.economy, passiveSeconds: 0 },
      }),
    ).toThrow();
  });
  it("uses reproducible bounded 64-bit randomness", () => {
    const a = new CombatRandom(defaultConfig.seed),
      b = new CombatRandom(defaultConfig.seed);
    for (let i = 0; i < 100; i++) {
      const v = a.next();
      expect(v).toBe(b.next());
      expect(v).toBeGreaterThanOrEqual(0);
      expect(v).toBeLessThan(1);
    }
  });
});

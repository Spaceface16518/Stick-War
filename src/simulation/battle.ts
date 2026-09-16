import type { ArenaDefinition, GameConfig } from "../content/config";
import type { SpatialWorld } from "../spatial/types";
import { acquireTargets, tickMovement } from "./movement";
import { tickCombat } from "./combat";
import { spawnUnit, tickEconomy, train, population } from "./economy";
import { CombatRandom } from "./random";
import {
  neutralInput,
  teams,
  type BattleCommand,
  type BattleEvent,
  type BattleMode,
  type BattleSnapshot,
  type CommandResult,
  type ControlInput,
  type Team,
} from "./types";
import type { BattleWorld } from "./world";
export const FIXED_DT = 1 / 60;
export class BattleSimulation {
  private w: BattleWorld;
  private disposed = false;
  constructor(
    config: GameConfig,
    arena: ArenaDefinition,
    mode: BattleMode,
    spatial: SpatialWorld,
  ) {
    this.w = {
      config: structuredClone(config),
      arena: structuredClone(arena),
      mode,
      spatial,
      random: new CombatRandom(config.seed),
      units: new Map(),
      statues: new Map(),
      projectiles: new Map(),
      training: [],
      teams: {
        blue: { gold: config.economy.startingGold, order: "defend" },
        red: { gold: config.economy.startingGold, order: "attack" },
      },
      settings: {
        costs: mode !== "sandbox",
        trainingTime: mode !== "sandbox",
        populationCap: config.economy.populationCap,
      },
      controlledId: null,
      paused: mode === "sandbox",
      outcome: null,
      elapsed: 0,
      tick: 0,
      nextId: 1,
      incomeTime: 0,
      recruitTime: 0,
      events: [],
      input: { ...neutralInput },
    };
    for (const team of teams) {
      const id = this.w.nextId++;
      this.w.statues.set(id, {
        id,
        team,
        ...arena.teams[team].statue,
        health: config.statueHealth,
        maxHealth: config.statueHealth,
      });
      for (let n = 0; n < config.economy.startingMiners; n++)
        spawnUnit(this.w, team, "miner");
    }
    this.sync();
  }
  command(command: BattleCommand): CommandResult {
    const w = this.w;
    if (this.disposed || w.outcome)
      return { ok: false, reason: "Battle has ended" };
    switch (command.type) {
      case "train": {
        const result = train(w, command.team, command.kind);
        this.sync();
        return result;
      }
      case "order":
        if (w.mode === "skirmish" && command.team !== "blue")
          return { ok: false, reason: "Enemy army is controlled by AI" };
        w.teams[command.team].order = command.order;
        return { ok: true };
      case "possess": {
        const u = w.units.get(command.id);
        if (
          !u ||
          u.kind === "miner" ||
          (w.mode === "skirmish" && u.team !== "blue")
        )
          return { ok: false, reason: "Choose a friendly swordsman or archer" };
        if (w.controlledId === u.id) return { ok: true };
        if (w.controlledId !== null) {
          const old = w.units.get(w.controlledId);
          if (old) old.attack = null;
        }
        w.controlledId = u.id;
        u.target = null;
        u.attack = null;
        w.input = { ...neutralInput, yaw: u.yaw };
        return { ok: true };
      }
      case "release": {
        const u =
          w.controlledId === null ? undefined : w.units.get(w.controlledId);
        if (u) u.attack = null;
        w.controlledId = null;
        w.input = { ...neutralInput };
        return { ok: true };
      }
      case "pause":
        w.paused = command.paused;
        w.input = { ...w.input, forward: 0, strafe: 0, attack: false };
        return { ok: true };
      case "sandbox":
        if (w.mode !== "sandbox")
          return { ok: false, reason: "Sandbox controls are unavailable" };
        if (
          command.populationCap !== undefined &&
          (!Number.isSafeInteger(command.populationCap) ||
            command.populationCap < 1)
        )
          return {
            ok: false,
            reason: "Population cap must be a positive integer",
          };
        if (command.costs !== undefined) w.settings.costs = command.costs;
        if (command.trainingTime !== undefined)
          w.settings.trainingTime = command.trainingTime;
        if (command.populationCap !== undefined)
          w.settings.populationCap = command.populationCap;
        return { ok: true };
    }
  }
  setInput(input: ControlInput): void {
    this.w.input = { ...input };
  }
  reloadConfig(config: GameConfig): void {
    this.w.config = structuredClone(config);
  }
  step(): void {
    const w = this.w;
    if (this.disposed || w.paused || w.outcome) return;
    w.elapsed += FIXED_DT;
    w.tick++;
    tickEconomy(w, FIXED_DT);
    this.sync();
    acquireTargets(w);
    tickMovement(w, FIXED_DT);
    this.sync();
    tickCombat(w, FIXED_DT);
    this.sync();
  }
  private sync(): void {
    this.w.spatial.sync(
      [...this.w.units.values()],
      [...this.w.statues.values()],
    );
  }
  snapshot(): BattleSnapshot {
    const w = this.w;
    return {
      mode: w.mode,
      elapsed: w.elapsed,
      paused: w.paused,
      outcome: w.outcome,
      controlledId: w.controlledId,
      units: structuredClone([...w.units.values()]),
      statues: structuredClone([...w.statues.values()]),
      projectiles: structuredClone([...w.projectiles.values()]),
      training: structuredClone(w.training),
      teams: structuredClone(w.teams),
      settings: { ...w.settings },
      tick: w.tick,
    };
  }
  population(team: Team): number {
    return population(this.w, team);
  }
  events(): BattleEvent[] {
    const events = this.w.events;
    this.w.events = [];
    return events;
  }
  diagnostics() {
    return {
      units: this.w.units.size,
      projectiles: this.w.projectiles.size,
      colliders: this.w.spatial.count(),
      tick: this.w.tick,
    };
  }
  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.w.spatial.dispose();
    this.w.units.clear();
    this.w.projectiles.clear();
    this.w.statues.clear();
    this.w.training = [];
    this.w.events = [];
  }
}

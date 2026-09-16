import type { BattleWorld } from "./world";
import {
  direction,
  kinds,
  teams,
  type CommandResult,
  type Team,
  type UnitKind,
} from "./types";
export function population(w: BattleWorld, team: Team): number {
  return (
    [...w.units.values()].filter((u) => u.team === team).length +
    w.training.filter((t) => t.team === team).length
  );
}
export function spawnUnit(w: BattleWorld, team: Team, kind: UnitKind): number {
  const def = w.config.units[kind];
  const id = w.nextId++;
  const anchor = w.arena.teams[team].spawn;
  const slot = [...w.units.values()].filter(
    (u) => u.team === team && Math.abs(u.x - anchor.x) < 3,
  ).length;
  const z = Math.max(
    -w.arena.halfWidth + 0.4,
    Math.min(w.arena.halfWidth - 0.4, ((slot % 4) - 1.5) * 1.05),
  );
  const x =
    anchor.x - direction(team) * Math.min(1.1, Math.floor(slot / 4) * 0.5);
  w.units.set(id, {
    id,
    team,
    kind,
    x,
    z,
    previous: { x, z },
    health: def.health,
    maxHealth: def.health,
    yaw: (direction(team) * Math.PI) / 2,
    cooldown: 0,
    attack: null,
    phase: "idle",
    carried: 0,
    miningRemaining: def.miningSeconds,
    minerState: "outbound",
    target: null,
    velocity: { x: 0, z: 0 },
    hitRemaining: 0,
  });
  w.events.push({ type: "spawn", id, team, kind });
  return id;
}
export function train(
  w: BattleWorld,
  team: Team,
  kind: UnitKind,
): CommandResult {
  if (w.mode === "skirmish" && team !== "blue")
    return { ok: false, reason: "Enemy army is controlled by AI" };
  return recruit(w, team, kind);
}
export function recruit(
  w: BattleWorld,
  team: Team,
  kind: UnitKind,
): CommandResult {
  if (w.training.some((t) => t.team === team && t.kind === kind))
    return { ok: false, reason: "Already training this unit" };
  if (population(w, team) >= w.settings.populationCap)
    return { ok: false, reason: "Population limit reached" };
  const def = w.config.units[kind];
  if (w.settings.costs && w.teams[team].gold < def.cost)
    return { ok: false, reason: "Not enough gold" };
  if (w.settings.costs) w.teams[team].gold -= def.cost;
  const duration = w.settings.trainingTime ? def.trainingSeconds : 0;
  if (duration === 0) spawnUnit(w, team, kind);
  else w.training.push({ team, kind, remaining: duration, duration });
  return { ok: true };
}
export function tickEconomy(w: BattleWorld, dt: number): void {
  w.incomeTime += dt;
  while (w.incomeTime + 1e-9 >= w.config.economy.passiveSeconds) {
    w.incomeTime -= w.config.economy.passiveSeconds;
    for (const team of teams)
      w.teams[team].gold += w.config.economy.passiveGold;
  }
  for (const t of w.training) t.remaining = Math.max(0, t.remaining - dt);
  const completed = w.training.filter((t) => t.remaining < 1e-8);
  w.training = w.training.filter((t) => t.remaining >= 1e-8);
  for (const t of completed) spawnUnit(w, t.team, t.kind);
  if (w.mode === "sandbox") return;
  w.recruitTime += dt;
  while (w.recruitTime + 1e-9 >= w.config.ai.recruitSeconds) {
    w.recruitTime -= w.config.ai.recruitSeconds;
    const counts = Object.fromEntries(
      kinds.map((kind) => [
        kind,
        [...w.units.values()].filter((u) => u.team === "red" && u.kind === kind)
          .length +
          w.training.filter((t) => t.team === "red" && t.kind === kind).length,
      ]),
    ) as Record<UnitKind, number>;
    const kind =
      counts.miner < w.config.ai.desiredMiners
        ? "miner"
        : counts.swordsman >= (counts.archer + 1) * w.config.ai.swordsPerArcher
          ? "archer"
          : "swordsman";
    recruit(w, "red", kind);
  }
}

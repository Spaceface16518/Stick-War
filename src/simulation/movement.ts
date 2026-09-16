import {
  clamp,
  direction,
  distance,
  opponent,
  type Point,
  type UnitState,
} from "./types";
import type { BattleWorld } from "./world";
export function formationPosition(w: BattleWorld, unit: UnitState): Point {
  const order = w.teams[unit.team].order;
  const dir = direction(unit.team);
  const base = w.arena.teams[unit.team];
  const f = w.config.formation;
  const siblings = [...w.units.values()]
    .filter(
      (u) =>
        u.team === unit.team && (order === "retreat" || u.kind === unit.kind),
    )
    .sort((a, b) => a.id - b.id);
  const slot = Math.max(
    0,
    siblings.findIndex((u) => u.id === unit.id),
  );
  const row = Math.floor(slot / f.columns);
  const z =
    dir *
    clamp(
      ((slot % f.columns) - (f.columns - 1) / 2) * f.spacing,
      -w.arena.halfWidth + 0.4,
      w.arena.halfWidth - 0.4,
    );
  if (order === "retreat")
    return {
      x: clamp(
        base.statue.x - dir * Math.max(1.55, f.retreatOffset) - dir * row * 0.7,
        -w.arena.halfLength + 0.5,
        w.arena.halfLength - 0.5,
      ),
      z,
    };
  return {
    x:
      base.mine.x +
      dir * (unit.kind === "swordsman" ? f.swordOffset : f.archerOffset) -
      dir * row * f.spacing,
    z,
  };
}
function moveToward(
  w: BattleWorld,
  u: UnitState,
  destination: Point,
  dt: number,
  avoid = true,
): void {
  // Route around the stationary statue footprint when the destination lies beyond it.
  // This keeps inner formation columns from walking forever into their own base.
  for (const statue of w.statues.values()) {
    const dir = Math.sign(destination.x - u.x);
    if (
      statue.health <= 0 ||
      !dir ||
      (statue.x - u.x) * dir < -1.5 ||
      (destination.x - statue.x) * dir < 1.5 ||
      Math.abs(destination.z - statue.z) >= 1.5
    )
      continue;
    const side = Math.sign(u.z - statue.z) || (u.id % 2 ? 1 : -1);
    const z = statue.z + side * 1.7;
    destination =
      Math.abs(u.x - statue.x) < 1.55 && Math.abs(u.z - statue.z) < 1.55
        ? { x: u.x, z }
        : { x: statue.x + dir * 1.65, z };
    break;
  }
  let dx = destination.x - u.x,
    dz = destination.z - u.z;
  const length = Math.hypot(dx, dz);
  const step = w.config.units[u.kind].speed * dt;
  if (length < 0.03) return;
  dx = (dx / length) * Math.min(step, length);
  dz = (dz / length) * Math.min(step, length);
  if (avoid)
    for (const other of w.units.values()) {
      if (other.id === u.id || other.team !== u.team) continue;
      const ax = u.x - other.x,
        az = u.z - other.z,
        d = Math.hypot(ax, az);
      if (d < 0.8) {
        const sign = az === 0 ? (u.id > other.id ? 1 : -1) : Math.sign(az);
        dz += sign * (0.8 - d) * step * 0.6;
      }
    }
  const limit = Math.hypot(dx, dz);
  if (limit > step) {
    dx = (dx / limit) * step;
    dz = (dz / limit) * step;
  }
  const next = w.spatial.move(u.id, u.team, u, { x: dx, z: dz });
  u.x = clamp(next.x, -w.arena.halfLength + 0.4, w.arena.halfLength - 0.4);
  u.z = clamp(next.z, -w.arena.halfWidth + 0.4, w.arena.halfWidth - 0.4);
  if (Math.hypot(u.x - u.previous.x, u.z - u.previous.z) > 0.0005) {
    u.yaw = Math.atan2(dx, dz);
    u.phase = u.carried > 0 ? "carry" : "walk";
  }
}
function moveMiner(w: BattleWorld, u: UnitState, dt: number): void {
  const base = w.arena.teams[u.team];
  const def = w.config.units.miner;
  if (w.teams[u.team].order === "retreat") {
    moveToward(w, u, formationPosition(w, u), dt);
    return;
  }
  if (u.minerState === "outbound") {
    moveToward(w, u, base.mine, dt, false);
    if (distance(u, base.mine) < 0.12) {
      u.minerState = "mining";
      u.miningRemaining = def.miningSeconds;
    }
  } else if (u.minerState === "mining") {
    u.phase = "mine";
    u.yaw = (direction(u.team) * Math.PI) / 2;
    u.miningRemaining -= dt;
    if (u.miningRemaining <= 0) {
      u.carried = def.capacity;
      u.minerState = "returning";
    }
  } else {
    const target = {
      x:
        base.statue.x + direction(u.team) * w.config.formation.mineReturnOffset,
      z: base.mine.z,
    };
    moveToward(w, u, target, dt, false);
    if (distance(u, target) < 0.12) {
      w.teams[u.team].gold += u.carried;
      w.events.push({ type: "gold", team: u.team, amount: u.carried });
      u.carried = 0;
      u.minerState = "outbound";
    }
  }
}
export function acquireTargets(w: BattleWorld): void {
  for (const u of w.units.values()) {
    if (
      u.kind === "miner" ||
      u.id === w.controlledId ||
      w.teams[u.team].order === "retreat"
    ) {
      u.target = null;
      continue;
    }
    const def = w.config.units[u.kind];
    const defense =
      w.arena.teams[u.team].mine.x +
      direction(u.team) * w.config.ai.defenseOffset;
    const candidates = [...w.units.values(), ...w.statues.values()].filter(
      (t) =>
        t.team !== u.team &&
        t.health > 0 &&
        distance(u, t) <= def.activationRange &&
        (w.teams[u.team].order !== "defend" ||
          Math.abs(t.x - defense) <= w.config.ai.defenseRadius),
    );
    const priority = (t: (typeof candidates)[number]) =>
      "target" in t && t.target === u.id ? 0 : t.id === u.target ? 1 : 2;
    candidates.sort(
      (a, b) =>
        priority(a) - priority(b) ||
        distance(u, a) - distance(u, b) ||
        a.id - b.id,
    );
    u.target = candidates[0]?.id ?? null;
  }
}
export function tickMovement(w: BattleWorld, dt: number): void {
  for (const u of w.units.values()) {
    u.previous = { x: u.x, z: u.z };
    u.phase = u.attack ? "attack" : "idle";
    if (u.id === w.controlledId) {
      const i = w.input;
      u.yaw = i.yaw;
      const length = Math.hypot(i.forward, i.strafe);
      const s = (w.config.units[u.kind].speed * dt) / Math.max(1, length);
      const delta = {
        x: (Math.sin(i.yaw) * i.forward - Math.cos(i.yaw) * i.strafe) * s,
        z: (Math.cos(i.yaw) * i.forward + Math.sin(i.yaw) * i.strafe) * s,
      };
      const next = w.spatial.move(u.id, u.team, u, delta);
      u.x = clamp(next.x, -w.arena.halfLength + 0.4, w.arena.halfLength - 0.4);
      u.z = clamp(next.z, -w.arena.halfWidth + 0.4, w.arena.halfWidth - 0.4);
      if (length > 0) u.phase = "walk";
    } else if (u.kind === "miner") moveMiner(w, u, dt);
    else if (!u.attack) {
      const order = w.teams[u.team].order;
      const target =
        u.target === null
          ? undefined
          : (w.units.get(u.target) ?? w.statues.get(u.target));
      const def = w.config.units[u.kind];
      if (target) {
        const d = distance(u, target);
        u.yaw = Math.atan2(target.x - u.x, target.z - u.z);
        const reach = def.range + ("kind" in target ? 0.3 : 1.0);
        if (
          u.kind === "archer" &&
          d < def.range * w.config.ai.archerBackpedal
        ) {
          const scale = 1 / Math.max(d, 0.01);
          moveToward(
            w,
            u,
            {
              x: u.x + (u.x - target.x) * scale * 2,
              z: u.z + (u.z - target.z) * scale,
            },
            dt,
          );
        } else if (
          d >
          (u.kind === "archer"
            ? def.range * w.config.ai.archerPreferred
            : reach * 0.92)
        )
          moveToward(w, u, target, dt);
      } else if (order === "attack")
        moveToward(w, u, w.arena.teams[opponent(u.team)].statue, dt);
      else moveToward(w, u, formationPosition(w, u), dt);
    }
    const smoothing = w.config.arrow.velocitySmoothing;
    u.velocity = {
      x:
        u.velocity.x * (1 - smoothing) +
        ((u.x - u.previous.x) / dt) * smoothing,
      z:
        u.velocity.z * (1 - smoothing) +
        ((u.z - u.previous.z) / dt) * smoothing,
    };
    u.hitRemaining = Math.max(0, u.hitRemaining - dt);
    if (u.attack) u.phase = "attack";
    else if (u.hitRemaining > 0) u.phase = "hit";
    else if (
      u.kind !== "miner" &&
      u.cooldown > w.config.units[u.kind].cooldown * 0.5
    )
      u.phase = "attack";
  }
}

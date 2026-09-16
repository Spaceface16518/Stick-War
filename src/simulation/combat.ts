import { distance, type AttackState, type UnitState, type Vec3 } from "./types";
import { targetById, type BattleWorld } from "./world";
function aimDirection(yaw: number, pitch: number): Vec3 {
  return {
    x: Math.sin(yaw) * Math.cos(pitch),
    y: Math.sin(pitch),
    z: Math.cos(yaw) * Math.cos(pitch),
  };
}
function startAttack(w: BattleWorld, u: UnitState, automatic: boolean): void {
  const def = w.config.units[u.kind];
  u.attack = {
    remaining: def.windup,
    duration: def.windup,
    damage: def.damage,
    range: def.range,
    aim: aimDirection(u.yaw, automatic ? 0 : w.input.pitch),
    target: u.target,
    automatic,
    arrowSpeed:
      w.config.arrow.speed *
      (1 + w.random.signed() * w.config.arrow.speedVariation),
    arrowGravity: w.config.arrow.gravity,
    arrowLifetime: w.config.arrow.lifetime,
    arrowRadius: w.config.arrow.radius,
    arrowVerticalVariation: w.config.arrow.verticalVariation,
    meleeArcDegrees: w.config.combat.meleeArcDegrees,
  };
  u.cooldown =
    def.cooldown +
    w.config.combat.cooldownExtra +
    w.random.signed() * w.config.combat.cooldownJitter;
  u.phase = "attack";
  w.events.push({ type: "attack", id: u.id, kind: u.kind });
}
function fire(w: BattleWorld, u: UnitState, attack: AttackState): void {
  const origin = {
    x: u.x + Math.sin(u.yaw) * 0.48,
    y: 1.4,
    z: u.z + Math.cos(u.yaw) * 0.48,
  };
  let velocity: Vec3;
  const target =
    attack.target === null ? undefined : targetById(w, attack.target);
  if (attack.automatic && target && target.health > 0) {
    const t = Math.max(0.05, distance(origin, target) / attack.arrowSpeed);
    const v = w.units.get(target.id)?.velocity ?? { x: 0, z: 0 };
    const dx = target.x + v.x * t - origin.x,
      dz = target.z + v.z * t - origin.z;
    const d = Math.max(0.01, Math.hypot(dx, dz));
    const flight = d / attack.arrowSpeed;
    velocity = {
      x: (dx / d) * attack.arrowSpeed,
      z: (dz / d) * attack.arrowSpeed,
      y:
        (("kind" in target ? 1.15 : 2.3) -
          origin.y +
          0.5 * attack.arrowGravity * flight * flight) /
          flight +
        w.random.signed() * attack.arrowVerticalVariation,
    };
  } else if (attack.automatic) return;
  else {
    const aim = aimDirection(w.input.yaw, w.input.pitch);
    velocity = {
      x: aim.x * attack.arrowSpeed,
      y: aim.y * attack.arrowSpeed,
      z: aim.z * attack.arrowSpeed,
    };
  }
  const id = w.nextId++;
  w.projectiles.set(id, {
    id,
    team: u.team,
    owner: u.id,
    ...origin,
    previous: { ...origin },
    velocity,
    gravity: attack.arrowGravity,
    remaining: attack.arrowLifetime,
    radius: attack.arrowRadius,
    damage: attack.damage,
  });
  w.events.push({ type: "shot", id: u.id, position: origin });
}
export function tickCombat(w: BattleWorld, dt: number): void {
  const damage = new Map<number, number>();
  const addDamage = (id: number, amount: number) =>
    damage.set(id, (damage.get(id) ?? 0) + amount);
  for (const u of w.units.values()) {
    if (u.kind === "miner") continue;
    u.cooldown = Math.max(0, u.cooldown - dt);
    const controlled = u.id === w.controlledId;
    const target = u.target === null ? undefined : targetById(w, u.target);
    const def = w.config.units[u.kind];
    if (!u.attack && u.cooldown <= 1e-8) {
      if (controlled && w.input.attack) startAttack(w, u, false);
      else if (
        !controlled &&
        w.teams[u.team].order !== "retreat" &&
        target &&
        target.health > 0 &&
        distance(u, target) <= def.range + ("kind" in target ? 0.3 : 1)
      )
        startAttack(w, u, true);
    }
    const attack = u.attack;
    if (!attack) continue;
    attack.remaining -= dt;
    if (attack.remaining > 1e-8) continue;
    if (u.kind === "archer") fire(w, u, attack);
    else {
      const candidates = w.spatial
        .nearby({ x: u.x, y: 0.9, z: u.z }, attack.range, u.team)
        .map((id) => targetById(w, id))
        .filter((t) => t && t.health > 0);
      const arc = Math.cos((attack.meleeArcDegrees * Math.PI) / 360);
      const eligible = candidates.filter((t) => {
        if (!t) return false;
        const d = Math.max(0.001, distance(u, t));
        return (
          (Math.sin(u.yaw) * (t.x - u.x) + Math.cos(u.yaw) * (t.z - u.z)) / d >=
            arc &&
          (!attack.automatic || t.id === attack.target)
        );
      });
      eligible.sort(
        (a, b) => distance(u, a!) - distance(u, b!) || a!.id - b!.id,
      );
      if (eligible[0]) addDamage(eligible[0].id, attack.damage);
    }
    u.attack = null;
  }
  for (const p of w.projectiles.values()) {
    const step = Math.min(dt, p.remaining);
    const from = { x: p.x, y: p.y, z: p.z };
    const to = {
      x: p.x + p.velocity.x * step,
      y: p.y + p.velocity.y * step - 0.5 * p.gravity * step * step,
      z: p.z + p.velocity.z * step,
    };
    p.previous = from;
    const hit = w.spatial.sweep(from, to, p.radius, p.team, p.owner);
    p.velocity.y -= p.gravity * step;
    p.remaining -= step;
    Object.assign(p, to);
    if (hit) {
      if (hit.id !== null) addDamage(hit.id, p.damage);
      w.projectiles.delete(p.id);
    } else if (p.remaining <= 1e-8 || p.y < 0) w.projectiles.delete(p.id);
  }
  // Aggregate all hits before resolving death: outcome cannot depend on team iteration order.
  for (const [id, amount] of damage) {
    const target = targetById(w, id);
    if (!target) continue;
    target.health = Math.max(0, target.health - amount);
    if ("hitRemaining" in target) target.hitRemaining = 0.12;
    w.events.push({
      type: "damage",
      id,
      amount,
      position: { x: target.x, y: 1.2, z: target.z },
    });
  }
  for (const u of w.units.values())
    if (u.health <= 0) {
      w.events.push({ type: "death", unit: structuredClone(u) });
      w.units.delete(u.id);
      if (w.controlledId === u.id) {
        w.controlledId = null;
        w.input.attack = false;
      }
    }
  if (w.mode === "skirmish") {
    const blue =
      [...w.statues.values()].find((s) => s.team === "blue")!.health <= 0;
    const red =
      [...w.statues.values()].find((s) => s.team === "red")!.health <= 0;
    if (blue || red) {
      w.outcome = blue && red ? "draw" : red ? "victory" : "defeat";
      w.events.push({ type: "outcome", outcome: w.outcome });
    }
  }
}

import RAPIER from "@dimforge/rapier3d-compat";
import type { ArenaDefinition } from "../content/config";
import type {
  Point,
  StatueState,
  Team,
  UnitState,
  Vec3,
} from "../simulation/types";
import type { SpatialHit, SpatialWorld } from "./types";
let initialization: Promise<void> | undefined;
export function initializePhysics(): Promise<void> {
  return (initialization ??= RAPIER.init());
}
const rotation = { x: 0, y: 0, z: 0, w: 1 };
export class RapierSpatial implements SpatialWorld {
  private world = new RAPIER.World({ x: 0, y: 0, z: 0 });
  private controller = this.world.createCharacterController(0.01);
  private colliders = new Map<number, RAPIER.Collider>();
  private metadata = new Map<number, { id: number; team: Team }>();
  private disposed = false;
  constructor(arena: ArenaDefinition) {
    this.world.timestep = 1 / 60;
    this.world.createCollider(
      RAPIER.ColliderDesc.cuboid(
        arena.halfLength + 2,
        0.1,
        arena.halfWidth + 2,
      ).setTranslation(0, -0.15, 0),
    );
    for (const sign of [-1, 1]) {
      this.world.createCollider(
        RAPIER.ColliderDesc.cuboid(arena.halfLength + 1, 2, 0.1).setTranslation(
          0,
          1,
          sign * (arena.halfWidth + 0.1),
        ),
      );
      this.world.createCollider(
        RAPIER.ColliderDesc.cuboid(0.1, 2, arena.halfWidth + 1).setTranslation(
          sign * (arena.halfLength + 0.1),
          1,
          0,
        ),
      );
    }
  }
  sync(units: readonly UnitState[], statues: readonly StatueState[]): void {
    const ids = new Set<number>();
    for (const object of [...units, ...statues]) {
      if (object.health <= 0) continue;
      ids.add(object.id);
      const isUnit = "kind" in object;
      let collider = this.colliders.get(object.id);
      if (!collider) {
        collider = this.world.createCollider(
          isUnit
            ? RAPIER.ColliderDesc.capsule(0.55, 0.35)
            : RAPIER.ColliderDesc.cuboid(1, 2.2, 1),
        );
        this.colliders.set(object.id, collider);
        this.metadata.set(collider.handle, {
          id: object.id,
          team: object.team,
        });
      }
      collider.setTranslation({
        x: object.x,
        y: isUnit ? 0.9 : 2.2,
        z: object.z,
      });
    }
    for (const [id, collider] of this.colliders)
      if (!ids.has(id)) {
        this.metadata.delete(collider.handle);
        this.world.removeCollider(collider, true);
        this.colliders.delete(id);
      }
    this.world.step();
  }
  move(id: number, team: Team, position: Point, delta: Point): Point {
    const collider = this.colliders.get(id);
    if (!collider) return { ...position };
    this.controller.computeColliderMovement(
      collider,
      { x: delta.x, y: 0, z: delta.z },
      undefined,
      undefined,
      (c) => {
        const data = this.metadata.get(c.handle);
        return (
          !data ||
          (data.id !== id &&
            (data.team !== team || c.shape.type === RAPIER.ShapeType.Cuboid))
        );
      },
    );
    const change = this.controller.computedMovement();
    const next = { x: position.x + change.x, z: position.z + change.z };
    collider.setTranslation({ ...next, y: 0.9 });
    return next;
  }
  sweep(
    from: Vec3,
    to: Vec3,
    radius: number,
    team: Team,
    owner: number,
  ): SpatialHit | null {
    const hit = this.world.castShape(
      from,
      rotation,
      { x: to.x - from.x, y: to.y - from.y, z: to.z - from.z },
      new RAPIER.Ball(radius),
      0,
      1,
      true,
      undefined,
      undefined,
      undefined,
      undefined,
      (c) => {
        const data = this.metadata.get(c.handle);
        return !data || (data.id !== owner && data.team !== team);
      },
    );
    return hit
      ? {
          id: this.metadata.get(hit.collider.handle)?.id ?? null,
          fraction: hit.time_of_impact,
        }
      : null;
  }
  nearby(center: Vec3, radius: number, team: Team): number[] {
    const ids: number[] = [];
    this.world.intersectionsWithShape(
      center,
      rotation,
      new RAPIER.Ball(radius),
      (c) => {
        const data = this.metadata.get(c.handle);
        if (data && data.team !== team) ids.push(data.id);
        return true;
      },
    );
    return ids.sort((a, b) => a - b);
  }
  count(): number {
    return this.disposed ? 0 : this.world.colliders.len();
  }
  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.world.free();
    this.colliders.clear();
    this.metadata.clear();
  }
}

import type {
  Point,
  StatueState,
  Team,
  UnitState,
  Vec3,
} from "../simulation/types";
export interface SpatialHit {
  id: number | null;
  fraction: number;
}
export interface SpatialWorld {
  sync(units: readonly UnitState[], statues: readonly StatueState[]): void;
  move(id: number, team: Team, position: Point, delta: Point): Point;
  sweep(
    from: Vec3,
    to: Vec3,
    radius: number,
    team: Team,
    owner: number,
  ): SpatialHit | null;
  nearby(center: Vec3, radius: number, team: Team): number[];
  dispose(): void;
  count(): number;
}

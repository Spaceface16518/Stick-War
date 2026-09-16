export const teams = ["blue", "red"] as const;
export type Team = (typeof teams)[number];
export const kinds = ["miner", "swordsman", "archer"] as const;
export type UnitKind = (typeof kinds)[number];
export type ArmyOrder = "attack" | "defend" | "retreat";
export type BattleMode = "skirmish" | "sandbox";
export type Outcome = "victory" | "defeat" | "draw";
export interface Vec3 {
  x: number;
  y: number;
  z: number;
}
export interface Point {
  x: number;
  z: number;
}
export interface ControlInput {
  forward: number;
  strafe: number;
  yaw: number;
  pitch: number;
  attack: boolean;
}
export const neutralInput: ControlInput = {
  forward: 0,
  strafe: 0,
  yaw: Math.PI / 2,
  pitch: 0,
  attack: false,
};
export interface AttackState {
  remaining: number;
  duration: number;
  damage: number;
  range: number;
  aim: Vec3;
  target: number | null;
  automatic: boolean;
  arrowSpeed: number;
  arrowGravity: number;
  arrowLifetime: number;
  arrowRadius: number;
  arrowVerticalVariation: number;
  meleeArcDegrees: number;
}
export type UnitPhase = "idle" | "walk" | "mine" | "carry" | "attack" | "hit";
export interface AttackMotion {
  clip: string;
  contact: number;
  startedAt: number;
  duration: number;
  windup: number;
}
export interface HitMotion {
  startedAt: number;
  duration: number;
  direction: Point;
  strength: number;
}
export interface UnitState extends Point {
  id: number;
  team: Team;
  kind: UnitKind;
  previous: Point;
  health: number;
  maxHealth: number;
  yaw: number;
  cooldown: number;
  attack: AttackState | null;
  attackMotion: AttackMotion | null;
  hitMotion: HitMotion | null;
  distanceTravelled: number;
  phase: UnitPhase;
  carried: number;
  miningRemaining: number;
  minerState: "outbound" | "mining" | "returning";
  target: number | null;
  velocity: Point;
  hitRemaining: number;
}
export interface StatueState extends Point {
  id: number;
  team: Team;
  health: number;
  maxHealth: number;
}
export interface ProjectileState extends Vec3 {
  id: number;
  team: Team;
  owner: number;
  previous: Vec3;
  velocity: Vec3;
  gravity: number;
  remaining: number;
  radius: number;
  damage: number;
}
export interface TrainingState {
  team: Team;
  kind: UnitKind;
  remaining: number;
  duration: number;
}
export interface TeamState {
  gold: number;
  order: ArmyOrder;
}
export interface SandboxSettings {
  costs: boolean;
  trainingTime: boolean;
  populationCap: number;
}
export interface BattleSnapshot {
  mode: BattleMode;
  elapsed: number;
  paused: boolean;
  outcome: Outcome | null;
  controlledId: number | null;
  units: readonly UnitState[];
  statues: readonly StatueState[];
  projectiles: readonly ProjectileState[];
  training: readonly TrainingState[];
  teams: Record<Team, TeamState>;
  settings: SandboxSettings;
  tick: number;
}
export type BattleCommand =
  | { type: "train"; team: Team; kind: UnitKind }
  | { type: "order"; team: Team; order: ArmyOrder }
  | { type: "possess"; id: number }
  | { type: "release" }
  | { type: "pause"; paused: boolean }
  | {
      type: "sandbox";
      costs?: boolean;
      trainingTime?: boolean;
      populationCap?: number;
    };
export interface CommandResult {
  ok: boolean;
  reason?: string;
}
export type BattleEvent =
  | { type: "spawn"; id: number; kind: UnitKind; team: Team }
  | { type: "attack"; id: number; kind: UnitKind }
  | { type: "shot"; id: number; position: Vec3 }
  | { type: "damage"; id: number; amount: number; position: Vec3 }
  | { type: "death"; unit: UnitState }
  | { type: "gold"; team: Team; amount: number }
  | { type: "outcome"; outcome: Outcome };
export const direction = (team: Team): number => (team === "blue" ? 1 : -1);
export const opponent = (team: Team): Team =>
  team === "blue" ? "red" : "blue";
export const distance = (a: Point, b: Point): number =>
  Math.hypot(a.x - b.x, a.z - b.z);
export const clamp = (value: number, min: number, max: number): number =>
  Math.max(min, Math.min(max, value));

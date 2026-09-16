import type { ArenaDefinition, GameConfig } from "../content/config";
import type { SpatialWorld } from "../spatial/types";
import type {
  BattleEvent,
  BattleMode,
  ControlInput,
  Outcome,
  ProjectileState,
  SandboxSettings,
  StatueState,
  Team,
  TeamState,
  TrainingState,
  UnitState,
} from "./types";
import type { CombatRandom } from "./random";
export interface BattleWorld {
  config: GameConfig;
  arena: ArenaDefinition;
  spatial: SpatialWorld;
  random: CombatRandom;
  mode: BattleMode;
  units: Map<number, UnitState>;
  statues: Map<number, StatueState>;
  projectiles: Map<number, ProjectileState>;
  training: TrainingState[];
  teams: Record<Team, TeamState>;
  settings: SandboxSettings;
  controlledId: number | null;
  paused: boolean;
  outcome: Outcome | null;
  elapsed: number;
  tick: number;
  nextId: number;
  incomeTime: number;
  recruitTime: number;
  events: BattleEvent[];
  input: ControlInput;
}
export const targetById = (
  w: BattleWorld,
  id: number,
): UnitState | StatueState | undefined => w.units.get(id) ?? w.statues.get(id);

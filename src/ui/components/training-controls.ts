import { html, LitElement } from "lit";
import type { GameConfig } from "../../content/config";
import {
  kinds,
  type BattleSnapshot,
  type Team,
  type UnitKind,
} from "../../simulation/types";
import type { UIAction } from "../actions";

interface TrainingOption {
  kind: UnitKind;
  label: string;
  disabled: boolean;
}

const unitLabels = {
  miner: { name: "Miner", icon: "⛏", key: "M" },
  swordsman: { name: "Swordsman", icon: "⚔", key: "S" },
  archer: { name: "Archer", icon: "➶", key: "R" },
};

export function trainingOptions(
  s: BattleSnapshot,
  config: GameConfig,
  team: Team,
): TrainingOption[] {
  const population =
    s.units.filter((u) => u.team === team).length +
    s.training.filter((t) => t.team === team).length;
  return kinds.map((kind) => {
    const definition = config.units[kind];
    const pending = s.training.find((t) => t.team === team && t.kind === kind);
    const cost = s.settings.costs ? `${definition.cost} gold` : "Free";
    const time = s.settings.trainingTime
      ? `${definition.trainingSeconds}s`
      : "instant";
    return {
      kind,
      label: pending
        ? `Training · ${pending.remaining.toFixed(1)}s`
        : `${cost} · ${time}`,
      disabled:
        !!pending ||
        population >= s.settings.populationCap ||
        (s.settings.costs && s.teams[team].gold < definition.cost),
    };
  });
}

export class TrainingControls extends LitElement {
  static properties = {
    options: {
      attribute: false,
      // Snapshots arrive at render speed. Only update when a button's display changes.
      hasChanged: (next: TrainingOption[], previous: TrainingOption[] = []) =>
        next.length !== previous.length ||
        next.some(
          (option, i) =>
            option.kind !== previous[i]?.kind ||
            option.label !== previous[i]?.label ||
            option.disabled !== previous[i]?.disabled,
        ),
    },
    onAction: { attribute: false },
  };

  declare options: TrainingOption[];
  declare onAction?: UIAction;

  constructor() {
    super();
    this.options = [];
  }

  protected createRenderRoot() {
    return this;
  }

  protected render() {
    return this.options.map(({ kind, label, disabled }) => {
      const unit = unitLabels[kind];
      return html`
        <button
          id="train-${kind}"
          ?disabled=${disabled}
          @click=${() => this.onAction?.("train", kind)}
        >
          <span class="unit-icon">${unit.icon}</span>
          <span>
            <b>${unit.name}</b>
            <small id="train-label-${kind}">${label}</small>
          </span>
          <kbd>${unit.key}</kbd>
        </button>
      `;
    });
  }
}

customElements.define("training-controls", TrainingControls);

import { nothing, render } from "lit";
import { battleScreen, menuScreen } from "./screens";
import type { UIAction } from "./actions";
import type { DialogKind, GameDialog } from "./components/game-dialog";
import type { SandboxControls } from "./components/sandbox-controls";
import {
  trainingOptions,
  type TrainingControls,
} from "./components/training-controls";
import type { Settings } from "../platform/settings";
import type { BattleMode, BattleSnapshot, Team } from "../simulation/types";
import type { GameConfig } from "../content/config";
export class GameUI {
  team: Team = "blue";
  private cache = new Map<string, string>();
  constructor(
    private root: HTMLElement,
    private action: UIAction,
  ) {
    root.addEventListener("click", (e) => {
      const element = (e.target as Element).closest<HTMLElement>(
        "[data-action]",
      );
      if (element && !element.hasAttribute("disabled"))
        this.action(element.dataset.action!, element.dataset.value);
    });
  }
  menu() {
    this.cache.clear();
    render(menuScreen(this.action), this.root);
  }
  battle(mode: BattleMode) {
    this.cache.clear();
    this.team = "blue";
    // A restart also resets component-local state, such as collapsed sandbox tools.
    render(nothing, this.root);
    render(battleScreen(mode, this.action), this.root);
  }
  update(s: BattleSnapshot, config: GameConfig, selectedId: number | null) {
    for (const team of ["blue", "red"] as Team[]) {
      const statue = s.statues.find((x) => x.team === team)!;
      this.text(`gold-${team}`, `◆ ${s.teams[team].gold}`);
      this.text(
        `pop-${team}`,
        `${s.units.filter((u) => u.team === team).length + s.training.filter((t) => t.team === team).length}/${s.settings.populationCap}`,
      );
      this.text(
        `health-${team}`,
        `${Math.ceil(statue.health)} / ${statue.maxHealth}`,
      );
      this.style(
        `statue-${team}`,
        "width",
        `${Math.max(0, statue.health / statue.maxHealth) * 100}%`,
      );
    }
    this.text(
      "clock",
      `${Math.floor(s.elapsed / 60)
        .toString()
        .padStart(2, "0")}:${Math.floor(s.elapsed % 60)
        .toString()
        .padStart(2, "0")}`,
    );
    const possessed = s.units.find((u) => u.id === s.controlledId);
    this.hidden("pov", !possessed);
    this.hidden("training", !!possessed);
    this.hidden("camera-hint", !!possessed);
    this.hidden("sandbox-panel", s.mode !== "sandbox" || !!possessed);
    this.hidden("paused-label", !s.paused);
    this.root.classList.toggle("possessing", !!possessed);
    if (possessed) {
      this.text("pov-kind", possessed.kind.toUpperCase());
      this.text("pov-health", `${Math.ceil(possessed.health)} HP`);
      this.text(
        "pov-cooldown",
        possessed.cooldown > 0 ? `${possessed.cooldown.toFixed(1)}s` : "READY",
      );
    }
    const selected = s.units.find((u) => u.id === selectedId);
    const allowed =
      selected &&
      selected.kind !== "miner" &&
      (s.mode === "sandbox" || selected.team === "blue");
    this.hidden("selection", !allowed || !!possessed);
    if (selected)
      this.text(
        "selection-label",
        `${selected.team.toUpperCase()} ${selected.kind} · ${Math.ceil(selected.health)} HP`,
      );
    this.root.querySelector<TrainingControls>("#training")!.options =
      trainingOptions(s, config, this.team);
    for (const order of ["attack", "defend", "retreat"])
      this.root
        .querySelector(`#order-${order}`)
        ?.classList.toggle(
          "active",
          s.teams[possessed?.team ?? this.team].order === order,
        );
    const sandbox = this.root.querySelector<SandboxControls>("#sandbox-panel")!;
    sandbox.paused = s.paused;
    sandbox.team = this.team;
    sandbox.costs = s.settings.costs;
    sandbox.trainingTime = s.settings.trainingTime;
    sandbox.populationCap = s.settings.populationCap;
  }
  private text(id: string, value: string) {
    if (this.cache.get(id) === value) return;
    const el = this.root.querySelector<HTMLElement>("#" + id);
    if (el) el.textContent = value;
    this.cache.set(id, value);
  }
  private style(id: string, key: string, value: string) {
    const token = id + key;
    if (this.cache.get(token) === value) return;
    this.root
      .querySelector<HTMLElement>("#" + id)
      ?.style.setProperty(key, value);
    this.cache.set(token, value);
  }
  private hidden(id: string, hidden: boolean) {
    const el = this.root.querySelector<HTMLElement>("#" + id);
    if (el) el.hidden = hidden;
  }
  notice(message: string) {
    this.text("notice", message);
  }
  error(message: string) {
    this.text("dev-error", message);
    this.hidden("dev-error", !message);
  }
  portrait(value: boolean) {
    this.hidden("rotate", !value);
  }
  dialog(kind: DialogKind, settings: Settings, inBattle: boolean) {
    const dialog = this.root.querySelector<GameDialog>("#dialog")!;
    dialog.kind = kind;
    dialog.settings = { ...settings };
    dialog.inBattle = inBattle;
    dialog.hidden = false;
  }
  closeDialog() {
    this.hidden("dialog", true);
  }
}

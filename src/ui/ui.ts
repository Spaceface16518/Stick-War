import type { Settings } from "../platform/settings";
import {
  kinds,
  type BattleMode,
  type BattleSnapshot,
  type Team,
} from "../simulation/types";
import type { GameConfig } from "../content/config";
export class GameUI {
  team: Team = "blue";
  private cache = new Map<string, string>();
  constructor(
    private root: HTMLElement,
    private action: (action: string, value?: string) => void,
  ) {
    root.addEventListener("click", (e) => {
      const element = (e.target as Element).closest<HTMLElement>(
        "[data-action]",
      );
      if (element?.dataset.action === "toggle-tools") {
        const collapsed = this.root
          .querySelector("#sandbox-panel")!
          .classList.toggle("collapsed");
        element.setAttribute("aria-expanded", String(!collapsed));
        element.textContent = collapsed ? "Sandbox ▸" : "Sandbox ▾";
      } else if (element && !element.hasAttribute("disabled"))
        this.action(element.dataset.action!, element.dataset.value);
    });
    root.addEventListener("change", (e) => {
      const element = e.target as HTMLInputElement;
      if (element.dataset.setting)
        this.action(
          "setting",
          `${element.dataset.setting}:${element.type === "checkbox" ? element.checked : element.value}`,
        );
    });
  }
  menu() {
    this.cache.clear();
    this.root.innerHTML = `<main class="menu"><div class="eyebrow">THE NARROW KINGDOM</div><h1>STICK<br><span>WAR</span><sup>3D</sup></h1><p class="intro">Command the battle.<br>Join the front line.</p><div class="menu-actions"><button class="primary" data-action="start" data-value="skirmish">Begin battle <span>→</span></button><button data-action="start" data-value="sandbox">Open sandbox <span>◇</span></button><button class="quiet" data-action="settings">Settings & controls</button></div><p class="menu-note">Build an economy. Raise an army.<br>Possess a soldier and fight for your statue.</p><footer>AN ORIGINAL BROWSER STRATEGY GAME</footer></main><div class="menu-art"><div class="sun-disc"></div><div class="crest">⚔</div><div class="art-caption">ONE FIELD.<br>TWO KINGDOMS.<br><em>Your command.</em></div></div><div id="dialog" class="dialog-layer" hidden></div>`;
  }
  battle(mode: BattleMode, config: GameConfig) {
    this.cache.clear();
    this.team = "blue";
    this.root.innerHTML = `<div class="battle-ui"><header class="status-bar"><div class="team-status blue"><div><b>BLUE</b><span id="gold-blue"></span><small id="pop-blue"></small></div><div class="statue-meter"><i id="statue-blue"></i></div><small id="health-blue"></small></div><div class="battle-center"><span id="clock">00:00</span><small>${mode === "sandbox" ? "SANDBOX" : "DESTROY THE ENEMY STATUE"}</small></div><div class="team-status red"><div><b>RED</b><span id="gold-red"></span><small id="pop-red"></small></div><div class="statue-meter"><i id="statue-red"></i></div><small id="health-red"></small></div><button class="icon-button" aria-label="Pause menu" data-action="pause">Ⅱ</button></header><div id="sandbox-panel" class="sandbox-panel" ${mode === "sandbox" ? "" : "hidden"}><button data-action="toggle-tools" aria-expanded="true">Sandbox ▾</button><button data-action="toggle-pause" id="sandbox-pause">▶ Resume</button><button data-action="team" data-value="blue" id="team-blue">Blue army</button><button data-action="team" data-value="red" id="team-red">Red army</button><button data-action="costs" id="costs">Costs off</button><button data-action="training-time" id="training-time">Time off</button><label>Cap <input id="cap" type="number" min="1" value="12" data-setting="cap" aria-label="Population cap"></label></div><div id="notice" class="notice" role="status"></div><div id="dev-error" class="dev-error" hidden></div><div id="selection" class="selection" hidden><span id="selection-label"></span><button data-action="possess-selected">Enter POV <kbd>Tab</kbd></button></div><div id="training" class="training">${kinds.map((kind, i) => `<button data-action="train" data-value="${kind}" id="train-${kind}"><span class="unit-icon">${["⛏", "⚔", "➶"][i]}</span><span><b>${kind === "swordsman" ? "Swordsman" : kind[0].toUpperCase() + kind.slice(1)}</b><small id="train-label-${kind}">${config.units[kind].cost} gold</small></span><kbd>${["M", "S", "R"][i]}</kbd></button>`).join("")}</div><div class="orders" id="orders"><button data-action="order" data-value="attack" id="order-attack"><span>⚔</span> Attack <kbd>1</kbd></button><button data-action="order" data-value="defend" id="order-defend"><span>◇</span> Defend <kbd>2</kbd></button><button data-action="order" data-value="retreat" id="order-retreat"><span>↶</span> Retreat <kbd>3</kbd></button></div><div class="camera-hint" id="camera-hint">← → pan · scroll to zoom · select a soldier or press Tab</div><div id="pov" class="pov-ui" hidden><div class="crosshair">+</div><div class="pov-status"><b id="pov-kind"></b><span id="pov-health"></span><small id="pov-cooldown"></small></div><div class="pov-actions"><button data-action="release">↑ Command <kbd>Esc</kbd></button><button data-action="cycle">Next soldier <kbd>Tab</kbd></button></div><div class="touch-controls"><div class="joystick" data-touch="move" aria-label="Move"><i></i></div><div class="look-zone" data-touch="look" aria-label="Look"></div><button class="attack-touch" data-touch="attack" aria-label="Attack; drag to aim">⚔<small>ATTACK</small></button></div><div class="pov-hint">WASD move · mouse / drag to aim · click / Space attack</div></div><div id="paused-label" class="paused-label" hidden>SIMULATION PAUSED</div></div><div id="dialog" class="dialog-layer" hidden></div><div id="rotate" class="rotate" hidden><span>↻</span><h2>Turn to landscape</h2><p>Your battle is paused.</p></div>`;
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
    for (const kind of kinds) {
      const def = config.units[kind];
      const pending = s.training.find(
        (t) => t.team === this.team && t.kind === kind,
      );
      this.text(
        `train-label-${kind}`,
        pending
          ? `Training · ${pending.remaining.toFixed(1)}s`
          : s.settings.costs
            ? `${def.cost} gold · ${def.trainingSeconds}s`
            : "Free · " +
              (s.settings.trainingTime ? `${def.trainingSeconds}s` : "instant"),
      );
      const pop =
        s.units.filter((u) => u.team === this.team).length +
        s.training.filter((t) => t.team === this.team).length;
      const button = this.root.querySelector<HTMLButtonElement>(
        `#train-${kind}`,
      )!;
      button.disabled =
        !!pending ||
        pop >= s.settings.populationCap ||
        (s.settings.costs && s.teams[this.team].gold < def.cost);
    }
    for (const order of ["attack", "defend", "retreat"])
      this.root
        .querySelector(`#order-${order}`)
        ?.classList.toggle(
          "active",
          s.teams[possessed?.team ?? this.team].order === order,
        );
    this.text("sandbox-pause", s.paused ? "▶ Resume" : "Ⅱ Pause");
    this.text("costs", `Costs ${s.settings.costs ? "on" : "off"}`);
    this.text(
      "training-time",
      `Time ${s.settings.trainingTime ? "on" : "off"}`,
    );
    for (const team of ["blue", "red"])
      this.root
        .querySelector(`#team-${team}`)
        ?.classList.toggle("active", this.team === team);
    const cap = this.root.querySelector<HTMLInputElement>("#cap");
    if (cap && document.activeElement !== cap)
      cap.value = String(s.settings.populationCap);
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
  dialog(
    type: "pause" | "settings" | "victory" | "defeat" | "draw",
    settings: Settings,
    inBattle: boolean,
  ) {
    const el = this.root.querySelector<HTMLElement>("#dialog")!;
    el.hidden = false;
    const title =
      type === "pause"
        ? "Take a breath."
        : type === "victory"
          ? "The field is yours."
          : type === "defeat"
            ? "Your statue has fallen."
            : type === "draw"
              ? "Both kingdoms fall."
              : "Make it yours.";
    el.innerHTML = `<section class="dialog"><div class="eyebrow">${type === "settings" ? "SETTINGS & CONTROLS" : type.toUpperCase()}</div><h2>${title}</h2>${type === "settings" ? `<label>Look sensitivity <input type="range" min="0.2" max="3" step="0.1" value="${settings.sensitivity}" data-setting="sensitivity"></label><label>Volume <input type="range" min="0" max="1" step="0.05" value="${settings.volume}" data-setting="volume"></label><label class="check">Reduced motion <input type="checkbox" ${settings.reducedMotion ? "checked" : ""} data-setting="reducedMotion"></label><p class="control-copy">COMMAND · M/S/R train · 1/2/3 orders · arrows or drag to pan · wheel/pinch to zoom<br><br>POV · Tab to enter/cycle · WASD move · mouse or drag to aim · click/Space attack · Escape to command<br><br>TOUCH · select a soldier to enter POV · left stick moves · right drag aims · drag the attack button to aim and fire together.</p>` : ""}<div class="dialog-buttons">${type === "pause" || type === "settings" ? `<button class="primary" data-action="resume">${inBattle ? "Return to battle" : "Back"}</button>` : ""}${inBattle ? '<button data-action="restart">Restart battle</button><button data-action="menu">Main menu</button>' : ""}${type === "pause" ? '<button class="quiet" data-action="settings">Settings & controls</button>' : ""}</div></section>`;
  }
  closeDialog() {
    const el = this.root.querySelector<HTMLElement>("#dialog");
    if (el) el.hidden = true;
  }
}

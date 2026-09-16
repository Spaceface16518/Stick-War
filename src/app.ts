import {
  defaultArena,
  defaultConfig,
  parseConfig,
  watchConfig,
  type GameConfig,
} from "./content/config";
import { BattleSimulation, FIXED_DT } from "./simulation/battle";
import {
  type BattleCommand,
  type BattleMode,
  type BattleSnapshot,
  type UnitKind,
  type ArmyOrder,
} from "./simulation/types";
import { initializePhysics, RapierSpatial } from "./spatial/rapier";
import { AssetStore } from "./presentation/assets";
import { BattleView } from "./presentation/view";
import { InputController } from "./platform/input";
import { BattleAudio } from "./platform/audio";
import { loadSettings, saveSettings, type Settings } from "./platform/settings";
import { GameUI } from "./ui/ui";
export class GameApp {
  private config = defaultConfig;
  private settings: Settings = loadSettings();
  private simulation: BattleSimulation | null = null;
  private snapshot: BattleSnapshot | null = null;
  private view!: BattleView;
  private input!: InputController;
  private ui: GameUI;
  private audio = new BattleAudio();
  private frameHandle: number | null = null;
  private lastFrame = 0;
  private accumulator = 0;
  private mode: BattleMode = "skirmish";
  private modal = false;
  private pausedBeforeModal = false;
  private frameTimes: number[] = [];
  private loadToken = 0;
  private canvas: HTMLCanvasElement;
  private uiRoot: HTMLElement;
  constructor(root: HTMLElement) {
    this.canvas = document.createElement("canvas");
    this.canvas.id = "battlefield";
    this.canvas.tabIndex = 0;
    this.canvas.setAttribute("aria-label", "Battlefield");
    this.uiRoot = document.createElement("div");
    this.uiRoot.id = "interface";
    root.append(this.canvas, this.uiRoot);
    this.ui = new GameUI(this.uiRoot, (action, value) =>
      this.action(action, value),
    );
  }
  async boot(): Promise<void> {
    const token = ++this.loadToken;
    const boot = document.querySelector<HTMLElement>("#boot")!;
    boot.hidden = false;
    boot.innerHTML =
      '<div class="eyebrow">THE NARROW KINGDOM</div><h1>STICK WAR</h1><p id="load-status">Preparing physics…</p><progress id="load-progress"></progress>';
    try {
      await initializePhysics();
      const assets = new AssetStore();
      await assets.load((fraction, message) => {
        const progress =
          document.querySelector<HTMLProgressElement>("#load-progress");
        if (progress) {
          progress.max = 1;
          progress.value = fraction;
        }
        const status = document.querySelector("#load-status");
        if (status) status.textContent = message;
      });
      if (token !== this.loadToken) return;
      this.view = new BattleView(
        this.canvas,
        assets,
        this.config,
        defaultArena,
      );
      this.input = new InputController(this.canvas, {
        possessed: () =>
          this.snapshot?.controlledId !== null &&
          this.snapshot?.controlledId !== undefined,
        paused: () => !!this.snapshot?.paused || this.modal,
        active: () =>
          !!this.simulation && !this.snapshot?.outcome && !this.modal,
        train: (k) => this.train(k),
        order: (o) => this.order(o),
        cycle: () => this.cycle(),
        release: () => this.release(),
        pause: () => this.pauseMenu(),
        pick: (x, y) => {
          this.view.selectedId = this.view.pick(x, y);
          this.invalidate();
        },
        pan: (p) => {
          this.view.pan((p / this.canvas.clientHeight) * this.view.viewHeight);
          this.invalidate();
        },
        zoom: (d) => {
          this.view.zoom(d);
          this.invalidate();
        },
        invalidate: () => this.invalidate(),
      });
      this.applySettings();
      this.ui.menu();
      boot.hidden = true;
      this.invalidate();
      window.addEventListener("resize", () => {
        this.view.resize();
        this.checkOrientation();
        this.invalidate();
      });
      window.addEventListener("blur", () => this.suspend());
      document.addEventListener("visibilitychange", () => {
        if (document.hidden) this.suspend();
        else this.invalidate();
      });
      this.canvas.addEventListener("webglcontextlost", (e) => {
        e.preventDefault();
        this.suspend();
        this.ui.notice(
          "Graphics paused. Waiting for the browser to restore the scene…",
        );
      });
      this.canvas.addEventListener("webglcontextrestored", () => {
        this.ui.notice("Graphics restored. Resume when ready.");
        this.invalidate();
      });
      watchConfig(
        (c) => this.reload(c),
        (message) => {
          this.ui.error(message);
          this.invalidate();
        },
      );
      if (import.meta.env.DEV) {
        Object.assign(window, {
          __stickWar: {
            snapshot: () => this.simulation?.snapshot() ?? null,
            command: (c: BattleCommand) => this.command(c),
            advance: (seconds: number) => {
              for (let i = 0; i < Math.min(600, Math.max(0, seconds)) * 60; i++)
                this.simulation?.step();
              this.refresh();
              this.invalidate();
            },
            configure: (c: unknown) => this.reload(parseConfig(c)),
            config: () => structuredClone(this.config),
            diagnostics: () => ({
              simulation: this.simulation?.diagnostics() ?? null,
              rendering: this.view.diagnostics(),
              frameCount: this.frameTimes.length,
              averageFrameMs:
                this.frameTimes.reduce((a, b) => a + b, 0) /
                Math.max(1, this.frameTimes.length),
            }),
          },
        });
      }
    } catch (error) {
      console.error(error);
      boot.innerHTML =
        '<h1>The battlefield couldn’t load.</h1><p id="load-error"></p><button id="retry" class="primary">Try again</button>';
      boot.querySelector("#load-error")!.textContent = String(error);
      boot
        .querySelector("#retry")!
        .addEventListener("click", () => void this.boot(), { once: true });
    }
  }
  private reload(c: GameConfig) {
    this.config = c;
    this.simulation?.reloadConfig(c);
    this.view?.reloadConfig(c);
    this.ui.error("");
    this.invalidate();
  }
  private refresh() {
    this.snapshot = this.simulation?.snapshot() ?? null;
  }
  private command(c: BattleCommand) {
    const result = this.simulation?.command(c);
    if (result && !result.ok)
      this.ui.notice(result.reason ?? "Unable to perform action");
    else this.ui.notice("");
    this.refresh();
    this.invalidate();
    return result;
  }
  private start(mode: BattleMode) {
    this.leave();
    this.mode = mode;
    this.modal = false;
    this.simulation = new BattleSimulation(
      this.config,
      defaultArena,
      mode,
      new RapierSpatial(defaultArena),
    );
    this.refresh();
    this.ui.battle(mode);
    this.accumulator = 0;
    this.lastFrame = 0;
    this.frameTimes = [];
    this.checkOrientation();
    void this.audio.unlock().catch(() => {});
    this.invalidate();
  }
  private leave() {
    if (this.simulation) {
      this.command({ type: "release" });
      this.input.release();
      this.simulation.dispose();
      this.simulation = null;
    }
    this.snapshot = null;
    this.view.clear();
    this.input?.clear();
    this.modal = false;
    this.accumulator = 0;
  }
  private train(kind: UnitKind) {
    if (this.snapshot?.controlledId !== null) return;
    this.command({ type: "train", team: this.ui.team, kind });
  }
  private order(order: ArmyOrder) {
    const controlled = this.snapshot?.units.find(
      (u) => u.id === this.snapshot?.controlledId,
    );
    this.command({
      type: "order",
      team: controlled?.team ?? this.ui.team,
      order,
    });
  }
  private possess(id: number) {
    const unit = this.snapshot?.units.find((u) => u.id === id);
    if (!unit) return;
    const result = this.command({ type: "possess", id });
    if (result?.ok) {
      this.input.enter(unit.yaw, matchMedia("(pointer:coarse)").matches);
      this.ui.notice("");
      this.invalidate();
    }
  }
  private cycle() {
    if (!this.snapshot) return;
    const units = this.snapshot.units
      .filter(
        (u) =>
          u.kind !== "miner" &&
          (this.mode === "sandbox"
            ? u.team === this.ui.team
            : u.team === "blue"),
      )
      .sort((a, b) => a.id - b.id);
    if (!units.length) {
      this.ui.notice("Train a swordsman or archer to enter their POV.");
      return;
    }
    const index = units.findIndex((u) => u.id === this.snapshot?.controlledId);
    this.possess(units[(index + 1) % units.length].id);
  }
  private release() {
    this.command({ type: "release" });
    this.input.release();
    this.invalidate();
  }
  private pauseMenu() {
    if (!this.simulation || this.snapshot?.outcome) return;
    this.pausedBeforeModal = !!this.snapshot?.paused;
    this.release();
    this.command({ type: "pause", paused: true });
    this.modal = true;
    this.input.clear();
    this.ui.dialog("pause", this.settings, true);
    this.invalidate();
  }
  private suspend() {
    if (!this.simulation || this.snapshot?.outcome || this.modal) return;
    this.pauseMenu();
  }
  private checkOrientation() {
    if (!this.simulation) return;
    const portrait =
      matchMedia("(pointer:coarse)").matches && innerHeight > innerWidth;
    if (portrait) {
      if (!this.modal) this.pauseMenu();
      this.ui.portrait(true);
    } else this.ui.portrait(false);
  }
  private action(action: string, value?: string) {
    if (action === "start") this.start(value as BattleMode);
    else if (action === "menu") {
      this.leave();
      this.ui.menu();
      this.invalidate();
    } else if (action === "restart") this.start(this.mode);
    else if (action === "train") this.train(value as UnitKind);
    else if (action === "order") this.order(value as ArmyOrder);
    else if (action === "cycle") this.cycle();
    else if (action === "possess-selected" && this.view.selectedId !== null)
      this.possess(this.view.selectedId);
    else if (action === "release") this.release();
    else if (action === "pause") this.pauseMenu();
    else if (action === "resume") {
      this.modal = false;
      this.ui.closeDialog();
      if (this.simulation)
        this.command({ type: "pause", paused: this.pausedBeforeModal });
      this.checkOrientation();
      this.invalidate();
    } else if (action === "settings") {
      if (this.simulation && !this.modal) {
        this.pausedBeforeModal = !!this.snapshot?.paused;
        this.release();
        this.command({ type: "pause", paused: true });
      }
      this.modal = true;
      this.ui.dialog("settings", this.settings, !!this.simulation);
      this.invalidate();
    } else if (action === "toggle-pause")
      this.command({ type: "pause", paused: !this.snapshot?.paused });
    else if (action === "team") {
      this.ui.team = value === "red" ? "red" : "blue";
      this.view.selectedId = null;
      this.invalidate();
    } else if (action === "costs")
      this.command({ type: "sandbox", costs: !this.snapshot?.settings.costs });
    else if (action === "training-time")
      this.command({
        type: "sandbox",
        trainingTime: !this.snapshot?.settings.trainingTime,
      });
    else if (action === "setting" && value) {
      const [key, raw] = value.split(":");
      if (key === "cap")
        this.command({ type: "sandbox", populationCap: Number(raw) });
      else {
        if (key === "reducedMotion")
          this.settings.reducedMotion = raw === "true";
        else if (key === "volume" || key === "sensitivity")
          this.settings[key] = Number(raw);
        saveSettings(this.settings);
        this.applySettings();
      }
    }
  }
  private applySettings() {
    this.view.reducedMotion = this.settings.reducedMotion;
    this.input.sensitivity = this.settings.sensitivity;
    this.audio.volume = this.settings.volume;
    document.body.classList.toggle(
      "reduced-motion",
      this.settings.reducedMotion,
    );
  }
  private invalidate() {
    if (!this.view || this.frameHandle !== null || document.hidden) return;
    this.frameHandle = requestAnimationFrame(this.frame);
  }
  private frame = (now: number) => {
    this.frameHandle = null;
    const dt = this.lastFrame
      ? Math.min(0.1, (now - this.lastFrame) / 1000)
      : 0;
    this.lastFrame = now;
    if (this.simulation) {
      this.simulation.setInput(this.input.sample(dt));
      if (!this.snapshot?.paused && !this.modal && !this.snapshot?.outcome) {
        this.accumulator += dt;
        let steps = 0;
        while (this.accumulator >= FIXED_DT && steps++ < 6) {
          this.simulation.step();
          this.accumulator -= FIXED_DT;
        }
      } else this.accumulator = 0;
      this.refresh();
      const events = this.simulation.events();
      if (
        this.snapshot?.controlledId === null &&
        document.pointerLockElement === this.canvas
      )
        this.input.release();
      this.view.consume(events);
      this.audio.play(events);
      if (this.snapshot)
        this.ui.update(this.snapshot, this.config, this.view.selectedId);
      if (this.snapshot?.outcome && !this.modal) {
        this.input.release();
        this.modal = true;
        this.ui.dialog(this.snapshot.outcome, this.settings, true);
      }
    }
    this.view.render(
      this.snapshot,
      this.snapshot?.paused ? 1 : this.accumulator / FIXED_DT,
      dt,
      { yaw: this.input.yaw, pitch: this.input.pitch },
    );
    if (this.snapshot && !this.snapshot.paused && !this.snapshot.outcome) {
      if (dt > 0) this.frameTimes.push(dt * 1000);
      if (this.frameTimes.length > 600) this.frameTimes.shift();
    }
    if (
      (this.snapshot && !this.snapshot.paused && !this.snapshot.outcome) ||
      this.input.navigating
    )
      this.invalidate();
    else this.lastFrame = 0;
  };
}

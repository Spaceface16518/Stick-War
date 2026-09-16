import {
  clamp,
  neutralInput,
  type ControlInput,
  type UnitKind,
  type ArmyOrder,
} from "../simulation/types";
interface InputActions {
  possessed: () => boolean;
  paused: () => boolean;
  active: () => boolean;
  train: (kind: UnitKind) => void;
  order: (order: ArmyOrder) => void;
  cycle: () => void;
  release: () => void;
  pause: () => void;
  pick: (x: number, y: number) => void;
  pan: (pixels: number) => void;
  zoom: (delta: number) => void;
  invalidate: () => void;
}
interface Gesture {
  role: string;
  startX: number;
  startY: number;
  x: number;
  y: number;
  distance: number;
  target: Element;
}
export class InputController {
  yaw = Math.PI / 2;
  pitch = 0;
  sensitivity = 1;
  private keys = new Set<string>();
  private pointers = new Map<number, Gesture>();
  private joystick = { x: 0, y: 0 };
  private mouseAttack = false;
  private ignoreUntil = 0;
  private abort = new AbortController();
  constructor(
    private canvas: HTMLCanvasElement,
    private actions: InputActions,
  ) {
    const options = { signal: this.abort.signal };
    window.addEventListener("keydown", this.keyDown, options);
    window.addEventListener(
      "keyup",
      (e) => {
        this.keys.delete(e.code);
        this.actions.invalidate();
      },
      options,
    );
    window.addEventListener("pointerdown", this.pointerDown, options);
    window.addEventListener("pointermove", this.pointerMove, options);
    window.addEventListener("pointerup", this.pointerUp, options);
    window.addEventListener("pointercancel", this.pointerCancel, options);
    window.addEventListener(
      "mousemove",
      (e) => {
        if (
          document.pointerLockElement === canvas &&
          actions.possessed() &&
          !actions.paused()
        )
          this.look(e.movementX, e.movementY);
      },
      options,
    );
    canvas.addEventListener(
      "wheel",
      (e) => {
        if (!actions.active() || actions.possessed()) return;
        e.preventDefault();
        if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) actions.pan(e.deltaX);
        else actions.zoom(e.deltaY * 0.001);
        actions.invalidate();
      },
      { ...options, passive: false },
    );
    canvas.addEventListener("contextmenu", (e) => e.preventDefault(), options);
    document.addEventListener(
      "pointerlockchange",
      () => {
        const locked = document.pointerLockElement === canvas;
        if (!locked && actions.possessed()) {
          this.clear();
          actions.release();
        }
        actions.invalidate();
      },
      options,
    );
  }
  private keyDown = (e: KeyboardEvent) => {
    if (!this.actions.active() || e.target instanceof HTMLInputElement) return;
    if (
      [
        "Tab",
        "Space",
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
      ].includes(e.code)
    )
      e.preventDefault();
    this.keys.add(e.code);
    if (!e.repeat) {
      if (e.code === "Escape") {
        if (this.actions.possessed()) this.actions.release();
        else this.actions.pause();
      } else if (e.code === "Tab") this.actions.cycle();
      else if (
        e.code.startsWith("Digit") &&
        ["Digit1", "Digit2", "Digit3"].includes(e.code)
      )
        this.actions.order(
          (["attack", "defend", "retreat"] as ArmyOrder[])[
            Number(e.code.at(-1)) - 1
          ],
        );
      else if (!this.actions.possessed()) {
        const kind = (
          { KeyM: "miner", KeyS: "swordsman", KeyR: "archer" } as Record<
            string,
            UnitKind
          >
        )[e.code];
        if (kind) this.actions.train(kind);
      }
    }
    this.actions.invalidate();
  };
  private pointerDown = (e: PointerEvent) => {
    if (!this.actions.active()) return;
    const target = e.target;
    if (!(target instanceof Element)) return;
    const touchTarget = target.closest<HTMLElement>("[data-touch]");
    if (target !== this.canvas && !touchTarget) return;
    if (this.actions.paused() && this.actions.possessed()) return;
    e.preventDefault();
    this.canvas.focus({ preventScroll: true });
    if (
      e.pointerType === "mouse" &&
      document.pointerLockElement === this.canvas
    ) {
      if (e.button === 0 && performance.now() > this.ignoreUntil)
        this.mouseAttack = true;
      return;
    }
    const role =
      touchTarget?.dataset.touch ?? (this.actions.possessed() ? "look" : "pan");
    const capture = touchTarget ?? this.canvas;
    try {
      capture.setPointerCapture(e.pointerId);
    } catch {
      /* Unsupported capture falls back to window listeners. */
    }
    this.pointers.set(e.pointerId, {
      role,
      startX: e.clientX,
      startY: e.clientY,
      x: e.clientX,
      y: e.clientY,
      distance: 0,
      target: capture,
    });
    this.actions.invalidate();
  };
  private pointerMove = (e: PointerEvent) => {
    const p = this.pointers.get(e.pointerId);
    if (!p) return;
    const dx = e.clientX - p.x,
      dy = e.clientY - p.y;
    const other = [...this.pointers.entries()].find(
      ([id, g]) => id !== e.pointerId && g.role === "pan",
    )?.[1];
    if (p.role === "pan" && other) {
      const before = Math.hypot(p.x - other.x, p.y - other.y),
        after = Math.hypot(e.clientX - other.x, e.clientY - other.y);
      if (before > 10 && after > 10)
        this.actions.zoom(Math.log(before / after));
    } else if (p.role === "pan") this.actions.pan(-dx);
    else if (p.role === "move") {
      this.joystick = {
        x: clamp((e.clientX - p.startX) / 45, -1, 1),
        y: clamp((p.startY - e.clientY) / 45, -1, 1),
      };
      (p.target as HTMLElement).style.setProperty(
        "--stick-x",
        `${this.joystick.x * 25}px`,
      );
      (p.target as HTMLElement).style.setProperty(
        "--stick-y",
        `${-this.joystick.y * 25}px`,
      );
    } else this.look(dx, dy);
    p.distance += Math.hypot(dx, dy);
    p.x = e.clientX;
    p.y = e.clientY;
    this.actions.invalidate();
  };
  private pointerUp = (e: PointerEvent) => {
    this.mouseAttack = false;
    const p = this.pointers.get(e.pointerId);
    if (p?.role === "pan" && p.distance < 7 && this.pointers.size === 1)
      this.actions.pick(e.clientX, e.clientY);
    this.finishPointer(e.pointerId);
  };
  private pointerCancel = (e: PointerEvent) => {
    this.mouseAttack = false;
    this.finishPointer(e.pointerId);
  };
  private finishPointer(id: number) {
    const p = this.pointers.get(id);
    if (p?.role === "move") {
      this.joystick = { x: 0, y: 0 };
      (p.target as HTMLElement).style.setProperty("--stick-x", "0px");
      (p.target as HTMLElement).style.setProperty("--stick-y", "0px");
    }
    this.pointers.delete(id);
    this.actions.invalidate();
  }
  private look(dx: number, dy: number) {
    if (this.actions.paused()) return;
    this.yaw -= dx * 0.003 * this.sensitivity;
    this.pitch = clamp(this.pitch - dy * 0.003 * this.sensitivity, -1.35, 1.35);
    this.actions.invalidate();
  }
  enter(yaw: number, touch = false): void {
    this.clear();
    this.yaw = yaw;
    this.pitch = 0;
    this.ignoreUntil = performance.now() + 200;
    this.canvas.focus({ preventScroll: true });
    if (!touch && this.canvas.requestPointerLock) {
      try {
        const result = this.canvas.requestPointerLock();
        if (result && typeof result.catch === "function")
          void result.catch(() => {
            /* Drag-to-look and Space remain available. */
          });
      } catch {
        /* Drag-to-look fallback. */
      }
    }
  }
  release(): void {
    this.clear();
    if (document.pointerLockElement === this.canvas) document.exitPointerLock();
  }
  clear(): void {
    this.keys.clear();
    this.mouseAttack = false;
    this.joystick = { x: 0, y: 0 };
    for (const p of this.pointers.values()) {
      if (p.role === "move") {
        (p.target as HTMLElement).style.setProperty("--stick-x", "0px");
        (p.target as HTMLElement).style.setProperty("--stick-y", "0px");
      }
    }
    this.pointers.clear();
  }
  sample(dt: number): ControlInput {
    if (!this.actions.possessed()) {
      const pan =
        (this.keys.has("ArrowRight") ? 1 : 0) -
        (this.keys.has("ArrowLeft") ? 1 : 0);
      if (pan) this.actions.pan(pan * dt * 500);
      return { ...neutralInput };
    }
    if (this.actions.paused())
      return { ...neutralInput, yaw: this.yaw, pitch: this.pitch };
    return {
      forward:
        (this.keys.has("KeyW") || this.keys.has("ArrowUp") ? 1 : 0) -
        (this.keys.has("KeyS") || this.keys.has("ArrowDown") ? 1 : 0) +
        this.joystick.y,
      strafe:
        (this.keys.has("KeyD") || this.keys.has("ArrowRight") ? 1 : 0) -
        (this.keys.has("KeyA") || this.keys.has("ArrowLeft") ? 1 : 0) +
        this.joystick.x,
      yaw: this.yaw,
      pitch: this.pitch,
      attack:
        this.mouseAttack ||
        this.keys.has("Space") ||
        [...this.pointers.values()].some((p) => p.role === "attack"),
    };
  }
  get navigating(): boolean {
    return this.keys.has("ArrowLeft") || this.keys.has("ArrowRight");
  }
  dispose(): void {
    this.release();
    this.abort.abort();
  }
}

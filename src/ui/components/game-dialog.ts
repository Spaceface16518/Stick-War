import { html, LitElement, nothing } from "lit";
import type { Settings } from "../../platform/settings";
import type { UIAction } from "../actions";

const titles = {
  pause: "Take a breath.",
  settings: "Make it yours.",
  victory: "The field is yours.",
  defeat: "Your statue has fallen.",
  draw: "Both kingdoms fall.",
};
export type DialogKind = keyof typeof titles;

export class GameDialog extends LitElement {
  static properties = {
    kind: { attribute: false },
    settings: { attribute: false },
    inBattle: { attribute: false },
    onAction: { attribute: false },
  };

  declare kind: DialogKind;
  declare settings?: Settings;
  declare inBattle: boolean;
  declare onAction?: UIAction;

  constructor() {
    super();
    this.kind = "pause";
    this.inBattle = false;
  }

  protected createRenderRoot() {
    return this;
  }

  protected render() {
    return html`
      <section
        class="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
      >
        <div class="eyebrow">
          ${this.kind === "settings" ? "SETTINGS & CONTROLS" : this.kind.toUpperCase()}
        </div>
        <h2 id="dialog-title">${titles[this.kind]}</h2>
        ${
          this.kind === "settings" && this.settings
            ? this.settingsForm(this.settings)
            : nothing
        }
        <div class="dialog-buttons">
          ${
            this.kind === "pause" || this.kind === "settings"
              ? html`<button
                  class="primary"
                  @click=${() => this.onAction?.("resume")}
                >
                  ${this.inBattle ? "Return to battle" : "Back"}
                </button>`
              : nothing
          }
          ${
            this.inBattle
              ? html`
                  <button @click=${() => this.onAction?.("restart")}>
                    Restart battle
                  </button>
                  <button @click=${() => this.onAction?.("menu")}>
                    Main menu
                  </button>
                `
              : nothing
          }
          ${
            this.kind === "pause"
              ? html`<button
                  class="quiet"
                  @click=${() => this.onAction?.("settings")}
                >
                  Settings & controls
                </button>`
              : nothing
          }
        </div>
      </section>
    `;
  }

  private settingsForm(settings: Settings) {
    return html`
      <label>
        Look sensitivity
        <input
          type="range"
          min="0.2"
          max="3"
          step="0.1"
          .value=${String(settings.sensitivity)}
          @change=${(event: Event) => this.changeSetting("sensitivity", event)}
        />
      </label>
      <label>
        Volume
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          .value=${String(settings.volume)}
          @change=${(event: Event) => this.changeSetting("volume", event)}
        />
      </label>
      <label class="check">
        Reduced motion
        <input
          type="checkbox"
          .checked=${settings.reducedMotion}
          @change=${(event: Event) => this.changeSetting("reducedMotion", event)}
        />
      </label>
      <p class="control-copy">
        COMMAND · M/S/R train · 1/2/3 orders · arrows or drag to pan ·
        wheel/pinch to zoom
        <br /><br />
        POV · Tab to enter/cycle · WASD move · mouse or drag to aim ·
        click/Space attack · Escape to command
        <br /><br />
        TOUCH · select a soldier to enter POV · left stick moves · right drag
        aims · drag the attack button to aim and fire together.
      </p>
    `;
  }

  private changeSetting(key: keyof Settings, event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    this.onAction?.(
      "setting",
      `${key}:${input.type === "checkbox" ? input.checked : input.value}`,
    );
  }
}

customElements.define("game-dialog", GameDialog);

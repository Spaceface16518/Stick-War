import { html, LitElement, noChange } from "lit";
import { live } from "lit/directives/live.js";
import type { Team } from "../../simulation/types";
import type { UIAction } from "../actions";

export class SandboxControls extends LitElement {
  static properties = {
    paused: { attribute: false },
    team: { attribute: false },
    costs: { attribute: false },
    trainingTime: { attribute: false },
    populationCap: { attribute: false },
    onAction: { attribute: false },
    collapsed: { state: true },
  };

  declare paused: boolean;
  declare team: Team;
  declare costs: boolean;
  declare trainingTime: boolean;
  declare populationCap: number;
  declare onAction?: UIAction;
  declare private collapsed: boolean;

  constructor() {
    super();
    this.paused = true;
    this.team = "blue";
    this.costs = false;
    this.trainingTime = false;
    this.populationCap = 12;
    this.collapsed = false;
  }

  protected createRenderRoot() {
    return this;
  }

  protected render() {
    // Preserve an unfinished edit while simulation props change; reconcile on blur.
    const editingCap = document.activeElement === this.querySelector("#cap");
    return html`
      <div class="sandbox-panel ${this.collapsed ? "collapsed" : ""}">
        <button
          aria-expanded=${String(!this.collapsed)}
          @click=${() => {
            this.collapsed = !this.collapsed;
          }}
        >
          Sandbox ${this.collapsed ? "▸" : "▾"}
        </button>
        <button
          id="sandbox-pause"
          @click=${() => this.onAction?.("toggle-pause")}
        >
          ${this.paused ? "▶ Resume" : "Ⅱ Pause"}
        </button>
        ${(["blue", "red"] as const).map(
          (team) => html`
            <button
              id="team-${team}"
              class=${this.team === team ? "active" : ""}
              aria-pressed=${String(this.team === team)}
              @click=${() => this.onAction?.("team", team)}
            >
              ${team === "blue" ? "Blue" : "Red"} army
            </button>
          `,
        )}
        <button
          id="costs"
          aria-pressed=${String(this.costs)}
          @click=${() => this.onAction?.("costs")}
        >
          Costs ${this.costs ? "on" : "off"}
        </button>
        <button
          id="training-time"
          aria-pressed=${String(this.trainingTime)}
          @click=${() => this.onAction?.("training-time")}
        >
          Time ${this.trainingTime ? "on" : "off"}
        </button>
        <label>
          Cap
          <input
            id="cap"
            type="number"
            min="1"
            aria-label="Population cap"
            .value=${editingCap ? noChange : live(String(this.populationCap))}
            @change=${this.changeCap}
            @blur=${() => this.requestUpdate()}
          />
        </label>
      </div>
    `;
  }

  private changeCap(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    this.onAction?.("setting", `cap:${input.value}`);
  }
}

customElements.define("sandbox-controls", SandboxControls);

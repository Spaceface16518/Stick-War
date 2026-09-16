import { html } from "lit";
import type { BattleMode } from "../simulation/types";
import type { UIAction } from "./actions";
import "./components/game-dialog";
import "./components/sandbox-controls";
import "./components/training-controls";

export function menuScreen(action: UIAction) {
  return html`
    <main class="menu">
      <div class="eyebrow">THE NARROW KINGDOM</div>
      <h1>STICK<br /><span>WAR</span><sup>3D</sup></h1>
      <p class="intro">Command the battle.<br />Join the front line.</p>
      <div class="menu-actions">
        <button class="primary" data-action="start" data-value="skirmish">
          Begin battle <span>→</span>
        </button>
        <button data-action="start" data-value="sandbox">
          Open sandbox <span>◇</span>
        </button>
        <button class="quiet" data-action="settings">
          Settings & controls
        </button>
      </div>
      <p class="menu-note">
        Build an economy. Raise an army.<br />Possess a soldier and fight for
        your statue.
      </p>
      <footer>AN ORIGINAL BROWSER STRATEGY GAME</footer>
    </main>
    <div class="menu-art">
      <div class="sun-disc"></div>
      <div class="crest">⚔</div>
      <div class="art-caption">
        ONE FIELD.<br />TWO KINGDOMS.<br /><em>Your command.</em>
      </div>
    </div>
    <game-dialog
      id="dialog"
      class="dialog-layer"
      .onAction=${action}
      hidden
    ></game-dialog>
  `;
}

export function battleScreen(mode: BattleMode, action: UIAction) {
  return html`
    <div class="battle-ui">
      <header class="status-bar">
        ${teamStatus("blue")}
        <div class="battle-center">
          <span id="clock">00:00</span>
          <small>
            ${mode === "sandbox" ? "SANDBOX" : "DESTROY THE ENEMY STATUE"}
          </small>
        </div>
        ${teamStatus("red")}
        <button class="icon-button" aria-label="Pause menu" data-action="pause">
          Ⅱ
        </button>
      </header>
      <sandbox-controls
        id="sandbox-panel"
        .onAction=${action}
        ?hidden=${mode !== "sandbox"}
      ></sandbox-controls>
      <div id="notice" class="notice" role="status"></div>
      <div id="dev-error" class="dev-error" hidden></div>
      <div id="selection" class="selection" hidden>
        <span id="selection-label"></span>
        <button data-action="possess-selected">Enter POV <kbd>Tab</kbd></button>
      </div>
      <training-controls
        id="training"
        class="training"
        .onAction=${action}
      ></training-controls>
      <div class="orders" id="orders">
        <button data-action="order" data-value="attack" id="order-attack">
          <span>⚔</span> Attack <kbd>1</kbd>
        </button>
        <button data-action="order" data-value="defend" id="order-defend">
          <span>◇</span> Defend <kbd>2</kbd>
        </button>
        <button data-action="order" data-value="retreat" id="order-retreat">
          <span>↶</span> Retreat <kbd>3</kbd>
        </button>
      </div>
      <div class="camera-hint" id="camera-hint">
        ← → pan · scroll to zoom · select a soldier or press Tab
      </div>
      ${possessionOverlay()}
      <div id="paused-label" class="paused-label" hidden>SIMULATION PAUSED</div>
    </div>
    <game-dialog
      id="dialog"
      class="dialog-layer"
      .onAction=${action}
      hidden
    ></game-dialog>
    <div id="rotate" class="rotate" hidden>
      <span>↻</span>
      <h2>Turn to landscape</h2>
      <p>Your battle is paused.</p>
    </div>
  `;
}

function teamStatus(team: "blue" | "red") {
  return html`
    <div class="team-status ${team}">
      <div>
        <b>${team.toUpperCase()}</b>
        <span id="gold-${team}"></span>
        <small id="pop-${team}"></small>
      </div>
      <div class="statue-meter"><i id="statue-${team}"></i></div>
      <small id="health-${team}"></small>
    </div>
  `;
}

function possessionOverlay() {
  return html`
    <div id="pov" class="pov-ui" hidden>
      <div class="crosshair">+</div>
      <div class="pov-status">
        <b id="pov-kind"></b>
        <span id="pov-health"></span>
        <small id="pov-cooldown"></small>
      </div>
      <div class="pov-actions">
        <button data-action="release">↑ Command <kbd>Esc</kbd></button>
        <button data-action="cycle">Next soldier <kbd>Tab</kbd></button>
      </div>
      <div class="touch-controls">
        <div class="joystick" data-touch="move" aria-label="Move"><i></i></div>
        <div class="look-zone" data-touch="look" aria-label="Look"></div>
        <button
          class="attack-touch"
          data-touch="attack"
          aria-label="Attack; drag to aim"
        >
          ⚔<small>ATTACK</small>
        </button>
      </div>
      <div class="pov-hint">
        WASD move · mouse / drag to aim · click / Space attack
      </div>
    </div>
  `;
}

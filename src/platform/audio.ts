import type { BattleEvent } from "../simulation/types";
export class BattleAudio {
  private context: AudioContext | null = null;
  volume = 0.35;
  async unlock() {
    this.context ??= new AudioContext();
    if (this.context.state === "suspended") await this.context.resume();
  }
  play(events: BattleEvent[]) {
    if (!this.context || this.context.state !== "running" || this.volume === 0)
      return;
    let budget = 4;
    for (const event of events) {
      if (budget <= 0) break;
      if (!["damage", "shot", "gold", "spawn"].includes(event.type)) continue;
      budget--;
      const context = this.context;
      const oscillator = context.createOscillator(),
        gain = context.createGain();
      oscillator.type = event.type === "damage" ? "triangle" : "sine";
      const frequency =
        event.type === "damage"
          ? 110
          : event.type === "shot"
            ? 450
            : event.type === "gold"
              ? 880
              : 330;
      oscillator.frequency.setValueAtTime(frequency, context.currentTime);
      oscillator.frequency.exponentialRampToValueAtTime(
        frequency * 0.6,
        context.currentTime + 0.1,
      );
      gain.gain.setValueAtTime(this.volume * 0.06, context.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, context.currentTime + 0.14);
      oscillator.connect(gain);
      gain.connect(context.destination);
      oscillator.start();
      oscillator.stop(context.currentTime + 0.15);
      oscillator.onended = () => {
        oscillator.disconnect();
        gain.disconnect();
      };
    }
  }
}

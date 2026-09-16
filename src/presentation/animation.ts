import * as THREE from "three";
import { animationLibrary } from "../content/animation";
import { clamp, type UnitState } from "../simulation/types";

export function attackClipProgress(
  unit: UnitState,
  time: number,
): number | null {
  const motion = unit.attackMotion;
  if (!motion || time >= motion.startedAt + motion.duration) return null;
  const elapsed = Math.max(0, time - motion.startedAt);
  return elapsed <= motion.windup && motion.windup > 0
    ? (motion.contact * elapsed) / motion.windup
    : motion.contact +
        (1 - motion.contact) *
          clamp(
            (elapsed - motion.windup) /
              Math.max(0.001, motion.duration - motion.windup),
            0,
            1,
          );
}

function localImpact(unit: UnitState) {
  const direction = unit.hitMotion?.direction ?? {
    x: -Math.sin(unit.yaw),
    z: -Math.cos(unit.yaw),
  };
  return {
    forward:
      direction.x * Math.sin(unit.yaw) + direction.z * Math.cos(unit.yaw),
    side: direction.x * Math.cos(unit.yaw) - direction.z * Math.sin(unit.yaw),
  };
}
export function deathClip(unit: UnitState): string {
  const { forward, side } = localImpact(unit);
  return Math.abs(forward) >= Math.abs(side)
    ? forward < 0
      ? "death_back"
      : "death_front"
    : side > 0
      ? "death_left"
      : "death_right";
}
function hitClip(unit: UnitState): string {
  const { forward, side } = localImpact(unit);
  return Math.abs(forward) >= Math.abs(side)
    ? forward < 0
      ? "hit_front"
      : "hit_back"
    : side > 0
      ? "hit_left"
      : "hit_right";
}

type Gait = Exclude<keyof typeof animationLibrary.locomotion, "runThreshold">;
interface AnimationLayer {
  actions: Map<string, THREE.AnimationAction>;
  active: string;
  motionStart: number;
  lowerBody: boolean;
}

/** Playback only. Distance drives footsteps; simulation timestamps drive contact. */
export class UnitAnimator {
  readonly mixer: THREE.AnimationMixer;
  private layers: AnimationLayer[] = [];
  private impacts = new Map<string, THREE.AnimationAction>();
  private activeHit: THREE.AnimationAction | null = null;
  private hitStarted = -1;
  private previousDistance: number | null = null;
  private phase = 0;
  private dying: string | null = null;

  constructor(
    private root: THREE.Object3D,
    clips: THREE.AnimationClip[],
  ) {
    this.mixer = new THREE.AnimationMixer(root);
    const split = !!root.getObjectByName("thigh_l");
    const isLeg = (track: THREE.KeyframeTrack) =>
      /^(root|pelvis|thigh_[lr]|shin_[lr]|foot_[lr]|toe_[lr])\./.test(
        track.name,
      );
    for (const source of clips.filter((clip) => clip.name.startsWith("hit_"))) {
      const clip = source.clone();
      clip.tracks = clip.tracks.filter((track) =>
        split
          ? /^(spine|head)\./.test(track.name)
          : /^(forearm_[lr]|hand_[lr])\./.test(track.name),
      );
      THREE.AnimationUtils.makeClipAdditive(clip);
      const action = this.mixer.clipAction(clip);
      action.setLoop(THREE.LoopOnce, 1);
      action.clampWhenFinished = true;
      this.impacts.set(source.name, action);
    }
    for (const lowerBody of split ? [true, false] : [false]) {
      const layer: AnimationLayer = {
        actions: new Map(),
        active: "",
        motionStart: -1,
        lowerBody,
      };
      for (const source of clips.filter(
        (clip) => !clip.name.startsWith("hit_"),
      )) {
        const clip = split
          ? new THREE.AnimationClip(
              source.name + (lowerBody ? ":legs" : ":upper"),
              source.duration,
              source.tracks.filter((track) => isLeg(track) === lowerBody),
            )
          : source;
        const action = this.mixer.clipAction(clip);
        if (/^(melee_|bow_|death_)/.test(source.name)) {
          action.setLoop(THREE.LoopOnce, 1);
          action.clampWhenFinished = true;
        }
        layer.actions.set(source.name, action);
      }
      this.layers.push(layer);
    }
  }

  get locomotionPhase(): number {
    return this.phase;
  }

  private select(
    layer: AnimationLayer,
    requested: string,
    stamp: number,
    paused: boolean,
  ) {
    const name = layer.actions.has(requested) ? requested : "idle";
    const action = layer.actions.get(name);
    if (!action) return null;
    if (name !== layer.active || stamp !== layer.motionStart) {
      const previous = layer.actions.get(layer.active);
      if (previous && previous !== action) {
        if (paused) previous.stop();
        else previous.fadeOut(0.12);
      }
      action.reset().setEffectiveWeight(1).play();
      if (previous && previous !== action && !paused) action.fadeIn(0.12);
      else action.stopFading();
      layer.active = name;
      layer.motionStart = stamp;
    }
    return action;
  }

  update(
    unit: UnitState,
    time: number,
    dt: number,
    paused: boolean,
    alpha = 1,
  ): void {
    const attack = attackClipProgress(unit, time);
    // Smoothed velocity is useful for aiming arrows, but footsteps need the
    // actual displacement after collisions, including stops against obstacles.
    const stepDistance = Math.hypot(
      unit.x - unit.previous.x,
      unit.z - unit.previous.z,
    );
    const speed = stepDistance * 60;
    let gait: Gait =
      unit.carried > 0
        ? "carry"
        : speed > animationLibrary.locomotion.runThreshold
          ? "run"
          : "walk";
    const dx = unit.x - unit.previous.x,
      dz = unit.z - unit.previous.z;
    const forward = dx * Math.sin(unit.yaw) + dz * Math.cos(unit.yaw);
    const sideways = dx * Math.cos(unit.yaw) - dz * Math.sin(unit.yaw);
    if (Math.abs(sideways) > Math.abs(forward) * 1.2)
      gait = sideways > 0 ? "strafe_left" : "strafe_right";
    else if (forward < -0.001) gait = "walk_back";
    const distance =
      unit.distanceTravelled - stepDistance * (paused ? 0 : 1 - alpha);
    if (this.previousDistance === null || distance < this.previousDistance) {
      this.phase =
        (unit.id * 0.61803398875 +
          distance / animationLibrary.locomotion[gait].distance) %
        1;
    } else {
      this.phase =
        (this.phase +
          (distance - this.previousDistance) /
            animationLibrary.locomotion[gait].distance) %
        1;
    }
    this.previousDistance = distance;
    const locomotion = speed > 0.04 ? gait : null;
    const desired =
      attack !== null
        ? unit.attackMotion!.clip
        : unit.kind === "miner" && unit.minerState === "mining" && !locomotion
          ? "mine"
          : (locomotion ?? "idle");
    for (const layer of this.layers) {
      const requested = layer.lowerBody && locomotion ? locomotion : desired;
      const striking = attack !== null && requested === unit.attackMotion?.clip;
      const action = this.select(
        layer,
        requested,
        striking ? unit.attackMotion!.startedAt : -1,
        paused,
      );
      if (!action) continue;
      const stepping = !!locomotion && requested === locomotion;
      action.paused = striking || stepping;
      if (striking) action.time = attack * action.getClip().duration;
      else if (stepping) action.time = this.phase * action.getClip().duration;
      else action.setEffectiveTimeScale(1);
    }
    const hit = unit.hitMotion;
    if (hit && time < hit.startedAt + hit.duration) {
      const action = this.impacts.get(hitClip(unit));
      if (action) {
        if (this.hitStarted !== hit.startedAt || action !== this.activeHit) {
          this.activeHit?.stop();
          action.reset().play();
          this.activeHit = action;
          this.hitStarted = hit.startedAt;
        }
        action.paused = true;
        action.time =
          clamp((time - hit.startedAt) / hit.duration, 0, 1) *
          action.getClip().duration;
        action.setEffectiveWeight(hit.strength);
      }
    } else {
      this.activeHit?.stop();
      this.activeHit = null;
    }
    this.mixer.update(paused ? 0 : dt);
  }

  die(unit: UnitState): void {
    this.dying = deathClip(unit);
    this.activeHit?.stop();
    this.activeHit = null;
  }

  updateDeath(age: number, dt: number, paused: boolean): void {
    for (const layer of this.layers) {
      const action = this.select(layer, this.dying ?? "death_back", -1, paused);
      if (!action) continue;
      action.paused = true;
      action.time = Math.min(age, action.getClip().duration);
    }
    this.mixer.update(paused ? 0 : dt);
  }

  dispose(): void {
    this.mixer.stopAllAction();
    this.mixer.uncacheRoot(this.root);
  }
}

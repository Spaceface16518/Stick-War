import * as THREE from "three";
import type { ArenaDefinition, GameConfig } from "../content/config";
import {
  clamp,
  type BattleEvent,
  type BattleSnapshot,
  type Team,
  type UnitState,
} from "../simulation/types";
import { AssetStore, teamColors } from "./assets";
import {
  box,
  material,
  primitiveCharacter,
  primitiveFirstPerson,
} from "./primitive-models";
interface Actor {
  assetKey: string;
  root: THREE.Group;
  model: THREE.Group;
  mixer: THREE.AnimationMixer | null;
  actions: Map<string, THREE.AnimationAction>;
  clip: string;
  bar: THREE.Mesh;
  ring: THREE.Mesh;
  phaseTime: number;
}
export class BattleView {
  readonly renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private commander = new THREE.OrthographicCamera();
  private firstPerson = new THREE.PerspectiveCamera();
  private actors = new Map<number, Actor>();
  private arrows = new Map<number, THREE.Mesh>();
  private arrowGeometry = new THREE.CylinderGeometry(0.02, 0.035, 0.65, 4);
  private arrowMaterial = material(0x715038);
  private healthGeometry = new THREE.PlaneGeometry(0.8, 0.045);
  private healthMaterials = {
    blue: new THREE.MeshBasicMaterial({ color: 0x9be1c2, depthTest: false }),
    red: new THREE.MeshBasicMaterial({ color: 0xf18b6c, depthTest: false }),
  };
  private ringGeometry = new THREE.RingGeometry(0.38, 0.43, 24);
  private ringMaterial = new THREE.MeshBasicMaterial({
    color: 0xe6d497,
    side: THREE.DoubleSide,
    transparent: true,
    opacity: 0.9,
  });
  private corpses: {
    root: THREE.Group;
    mixer: THREE.AnimationMixer | null;
    time: number;
  }[] = [];
  private fpsModel: THREE.Group | null = null;
  private fpsKey = "";
  private controlled: number | null = null;
  private lastControlledX = -20;
  private raycaster = new THREE.Raycaster();
  private modelCache = new Map<string, THREE.Group>();
  panX = -20;
  viewHeight: number;
  selectedId: number | null = null;
  reducedMotion = false;
  private width = 1;
  private height = 1;
  readonly mobile = matchMedia("(pointer:coarse)").matches;
  constructor(
    readonly canvas: HTMLCanvasElement,
    private assets: AssetStore,
    private config: GameConfig,
    readonly arena: ArenaDefinition,
  ) {
    this.viewHeight = config.camera.viewHeight;
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: !this.mobile,
      powerPreference: "high-performance",
    });
    this.renderer.setPixelRatio(
      Math.min(devicePixelRatio, this.mobile ? 1.25 : 2),
    );
    this.renderer.shadowMap.enabled = !this.mobile;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.scene.background = new THREE.Color(0xcddbd6);
    this.scene.fog = new THREE.Fog(0xcddbd6, 65, 130);
    this.scene.add(new THREE.HemisphereLight(0xe5f2fa, 0x6c664b, 1.5));
    const sun = new THREE.DirectionalLight(0xffe2a9, 2.6);
    sun.position.set(-15, 24, 16);
    sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048);
    Object.assign(sun.shadow.camera, {
      left: -38,
      right: 38,
      top: 20,
      bottom: -20,
      near: 1,
      far: 80,
    });
    sun.shadow.bias = -0.0005;
    this.scene.add(sun);
    this.firstPerson.fov = config.camera.fov;
    this.firstPerson.near = 0.04;
    this.firstPerson.far = 160;
    this.firstPerson.rotation.order = "YXZ";
    this.scene.add(this.firstPerson);
    this.createArena();
    this.resize();
  }
  reloadConfig(config: GameConfig) {
    this.config = config;
  }
  private createArena(): void {
    const authored = this.assets.instantiate("arena");
    if (authored) {
      this.scene.add(authored.root);
      authored.root.traverse((o) => {
        if (o instanceof THREE.Mesh) {
          o.receiveShadow = true;
          o.castShadow = !this.mobile;
        }
      });
      return;
    }
    const earth = material(0x8c9670),
      stone = material(0xbdb89f);
    box(this.scene, [64, 0.6, 6], [0, -0.35, 0], earth);
    box(this.scene, [64, 1.4, 5.8], [0, -1.25, 0], stone);
    for (const team of ["blue", "red"] as Team[]) {
      const base = this.arena.teams[team];
      box(this.scene, [2.2, 0.4, 2.2], [base.statue.x, 0.2, 0], stone);
      const statue = primitiveCharacter("swordsman", team);
      statue.scale.setScalar(2.2);
      statue.position.set(base.statue.x, 0.4, 0);
      statue.traverse((o) => {
        if (o instanceof THREE.Mesh) o.material = stone;
      });
      this.scene.add(statue);
      box(
        this.scene,
        [0.12, 5, 0.12],
        [base.statue.x, 2.5, -2],
        material(0x65553b),
      );
      box(
        this.scene,
        [1.5, 1.2, 0.035],
        [base.statue.x + 0.65, 4, -2],
        material(teamColors[team]),
      );
      const ore = new THREE.Mesh(
        new THREE.DodecahedronGeometry(0.65, 0),
        material(0xc7a955),
      );
      ore.position.set(base.mine.x, 0.4, base.mine.z);
      this.scene.add(ore);
    }
    for (let i = 0; i < 28; i++) {
      const hill = new THREE.Mesh(
        new THREE.ConeGeometry(5 + (i % 5), 12 + (i % 4) * 4, 5),
        material(i % 2 ? 0x85998b : 0x9aab9b),
      );
      hill.position.set((i - 14) * 7, -5, -20 - (i % 3) * 12);
      this.scene.add(hill);
    }
    for (let i = 0; i < 64; i++) {
      const rock = new THREE.Mesh(
        new THREE.DodecahedronGeometry(0.25 + (i % 4) * 0.08),
        stone,
      );
      rock.position.set(i - 32, -0.03, i % 2 ? 3.25 : -3.25);
      this.scene.add(rock);
    }
  }
  resize(): void {
    this.width = this.canvas.clientWidth || innerWidth;
    this.height = this.canvas.clientHeight || innerHeight;
    this.renderer.setSize(this.width, this.height, false);
    this.firstPerson.aspect = this.width / this.height;
    this.firstPerson.updateProjectionMatrix();
    this.updateCommander();
  }
  private updateCommander(): void {
    const aspect = this.width / this.height;
    const halfWidth = (this.viewHeight * aspect) / 2;
    this.panX = clamp(
      this.panX,
      -Math.max(0, this.arena.halfLength - halfWidth),
      Math.max(0, this.arena.halfLength - halfWidth),
    );
    Object.assign(this.commander, {
      left: -halfWidth,
      right: halfWidth,
      top: this.viewHeight / 2,
      bottom: -this.viewHeight / 2,
      near: 0.1,
      far: 200,
    });
    this.commander.position.set(this.panX, 14, 22);
    this.commander.lookAt(this.panX, 0.4, 0);
    this.commander.updateProjectionMatrix();
  }
  pan(delta: number): void {
    this.panX += delta;
    this.updateCommander();
  }
  zoom(delta: number): void {
    this.viewHeight = clamp(
      this.viewHeight * Math.exp(delta),
      this.config.camera.minViewHeight,
      this.config.camera.maxViewHeight,
    );
    this.updateCommander();
  }
  private spawnActor(u: UnitState, assetKey: string): Actor {
    const loaded = this.assets.instantiate(assetKey, u.team);
    const model = loaded?.root ?? primitiveCharacter(u.kind, u.team);
    const root = new THREE.Group();
    root.add(model);
    root.userData.unitId = u.id;
    model.traverse((o) => {
      o.userData.unitId = u.id;
      if (o instanceof THREE.Mesh) {
        o.castShadow = !this.mobile;
        o.receiveShadow = true;
      }
    });
    const mixer = loaded ? new THREE.AnimationMixer(model) : null;
    const actions = new Map<string, THREE.AnimationAction>();
    if (mixer)
      for (const clip of loaded!.clips) {
        const action = mixer.clipAction(clip);
        if (clip.name.endsWith("_attack")) {
          action.timeScale = 2;
          action.setLoop(THREE.LoopOnce, 1);
          action.clampWhenFinished = true;
        }
        actions.set(clip.name, action);
      }
    const bar = new THREE.Mesh(
      this.healthGeometry,
      this.healthMaterials[u.team],
    );
    bar.position.y = 2.1;
    bar.renderOrder = 10;
    root.add(bar);
    const ring = new THREE.Mesh(this.ringGeometry, this.ringMaterial);
    ring.rotation.x = -Math.PI / 2;
    ring.position.y = 0.015;
    root.add(ring);
    this.scene.add(root);
    const actor = {
      assetKey,
      root,
      model,
      mixer,
      actions,
      clip: "",
      bar,
      ring,
      phaseTime: 0,
    };
    this.actors.set(u.id, actor);
    return actor;
  }
  private disposeSkeletons(root: THREE.Object3D): void {
    const skeletons = new Set<THREE.Skeleton>();
    root.traverse((o) => {
      if (o instanceof THREE.SkinnedMesh) skeletons.add(o.skeleton);
    });
    for (const skeleton of skeletons) skeleton.dispose();
  }
  private removeActor(actor: Actor): void {
    this.scene.remove(actor.root);
    actor.mixer?.stopAllAction();
    actor.mixer?.uncacheRoot(actor.model);
    this.disposeSkeletons(actor.model);
  }
  consume(events: BattleEvent[]): void {
    if (this.reducedMotion) return;
    for (const event of events)
      if (event.type === "death") {
        const actor = this.actors.get(event.unit.id);
        if (!actor) continue;
        const loaded = this.assets.instantiate(actor.assetKey, event.unit.team);
        const clone = loaded?.root ?? actor.model.clone(true);
        const corpse = new THREE.Group();
        corpse.add(clone);
        corpse.position.set(event.unit.x, 0, event.unit.z);
        corpse.rotation.y = event.unit.yaw;
        this.scene.add(corpse);
        const mixer = loaded ? new THREE.AnimationMixer(clone) : null;
        const death = loaded?.clips.find((c) => c.name === "death");
        if (death && mixer) {
          const action = mixer.clipAction(death);
          action.setLoop(THREE.LoopOnce, 1);
          action.clampWhenFinished = true;
          action.play();
        }
        this.corpses.push({ root: corpse, mixer, time: 0 });
        while (this.corpses.length > this.config.presentation.maxCorpses) {
          const old = this.corpses.shift()!;
          old.mixer?.stopAllAction();
          this.disposeSkeletons(old.root);
          this.scene.remove(old.root);
        }
      }
  }
  render(
    snapshot: BattleSnapshot | null,
    alpha: number,
    dt: number,
    look: { yaw: number; pitch: number },
  ): void {
    const active = new Set<number>();
    const possessed = snapshot?.units.find(
      (u) => u.id === snapshot.controlledId,
    );
    if (snapshot)
      for (const u of snapshot.units) {
        active.add(u.id);
        const baseKey = this.config.units[u.kind].asset;
        const close =
          possessed && Math.hypot(u.x - possessed.x, u.z - possessed.z) < 12;
        const assetKey =
          !close && this.assets.has(baseKey + "_lod")
            ? baseKey + "_lod"
            : baseKey;
        let actor = this.actors.get(u.id);
        if (actor && actor.assetKey !== assetKey) {
          this.removeActor(actor);
          this.actors.delete(u.id);
          actor = undefined;
        }
        actor ??= this.spawnActor(u, assetKey);
        actor.root.position.set(
          THREE.MathUtils.lerp(u.previous.x, u.x, alpha),
          0,
          THREE.MathUtils.lerp(u.previous.z, u.z, alpha),
        );
        actor.root.rotation.y = u.yaw;
        actor.root.visible = u.id !== snapshot.controlledId;
        actor.bar.scale.x = Math.max(0, u.health / u.maxHealth);
        actor.bar.visible = u.health < u.maxHealth || u.id === this.selectedId;
        actor.bar.quaternion
          .copy(this.commander.quaternion)
          .premultiply(actor.root.quaternion.clone().invert());
        actor.ring.visible = u.id === this.selectedId;
        actor.phaseTime += snapshot.paused ? 0 : dt;
        const desired =
          u.phase === "attack"
            ? u.kind === "archer"
              ? "bow_attack"
              : "melee_attack"
            : u.phase === "carry"
              ? "carry"
              : u.phase === "mine"
                ? "mine"
                : u.phase === "hit"
                  ? "hit"
                  : u.phase;
        const clip = actor.actions.has(desired) ? desired : "idle";
        if (clip !== actor.clip) {
          actor.actions.get(actor.clip)?.fadeOut(0.12);
          actor.actions.get(clip)?.reset().fadeIn(0.12).play();
          actor.clip = clip;
        }
        if (!snapshot.paused) actor.mixer?.update(dt);
        if (!actor.mixer) {
          actor.model.rotation.z =
            u.phase === "walk" || u.phase === "carry"
              ? Math.sin(actor.phaseTime * 8) * 0.035
              : 0;
          actor.model.position.y =
            u.phase === "walk"
              ? Math.abs(Math.sin(actor.phaseTime * 8)) * 0.035
              : 0;
        }
      }
    for (const [id, actor] of this.actors)
      if (!active.has(id)) {
        this.removeActor(actor);
        this.actors.delete(id);
      }
    const projectileIds = new Set<number>();
    if (snapshot)
      for (const p of snapshot.projectiles) {
        projectileIds.add(p.id);
        let arrow = this.arrows.get(p.id);
        if (!arrow) {
          arrow = new THREE.Mesh(this.arrowGeometry, this.arrowMaterial);
          this.arrows.set(p.id, arrow);
          this.scene.add(arrow);
        }
        arrow.position.set(
          THREE.MathUtils.lerp(p.previous.x, p.x, alpha),
          THREE.MathUtils.lerp(p.previous.y, p.y, alpha),
          THREE.MathUtils.lerp(p.previous.z, p.z, alpha),
        );
        arrow.quaternion.setFromUnitVectors(
          new THREE.Vector3(0, 1, 0),
          new THREE.Vector3(
            p.velocity.x,
            p.velocity.y,
            p.velocity.z,
          ).normalize(),
        );
      }
    for (const [id, arrow] of this.arrows)
      if (!projectileIds.has(id)) {
        this.scene.remove(arrow);
        this.arrows.delete(id);
      }
    for (const corpse of this.corpses) {
      if (!snapshot?.paused) {
        corpse.time += dt;
        corpse.mixer?.update(dt);
      }
      if (!corpse.mixer)
        corpse.root.rotation.z = Math.min(Math.PI / 2, corpse.time * 3.5);
      corpse.root.position.y = -Math.max(0, corpse.time - 0.9) * 0.9;
    }
    this.corpses = this.corpses.filter((c) => {
      if (c.time > this.config.presentation.corpseSeconds) {
        c.mixer?.stopAllAction();
        this.disposeSkeletons(c.root);
        this.scene.remove(c.root);
        return false;
      }
      return true;
    });
    if (possessed) {
      this.controlled = possessed.id;
      this.lastControlledX = possessed.x;
      this.firstPerson.position.set(
        THREE.MathUtils.lerp(possessed.previous.x, possessed.x, alpha),
        this.config.camera.eyeHeight,
        THREE.MathUtils.lerp(possessed.previous.z, possessed.z, alpha),
      );
      this.firstPerson.rotation.set(look.pitch, look.yaw + Math.PI, 0);
      const key = possessed.kind + possessed.team;
      if (key !== this.fpsKey) {
        if (this.fpsModel) this.firstPerson.remove(this.fpsModel);
        let model = this.modelCache.get(key);
        if (!model) {
          model =
            this.assets.instantiate("fp_" + possessed.kind, possessed.team)
              ?.root ?? primitiveFirstPerson(possessed.kind, possessed.team);
          this.modelCache.set(key, model);
        }
        this.fpsModel = model;
        this.fpsKey = key;
        this.firstPerson.add(model);
        model.traverse((o) => {
          if (o instanceof THREE.Mesh) {
            o.castShadow = false;
            o.frustumCulled = false;
          }
        });
      }
      if (this.fpsModel) {
        this.fpsModel.visible = true;
        const def = this.config.units[possessed.kind];
        const swing =
          possessed.cooldown > 0
            ? Math.sin(
                (1 - clamp(possessed.cooldown / def.cooldown, 0, 1)) * Math.PI,
              )
            : 0;
        this.fpsModel.rotation.z =
          possessed.kind === "swordsman"
            ? -swing * (this.reducedMotion ? 0.15 : 0.65)
            : 0;
        this.fpsModel.position.z = this.reducedMotion ? 0 : swing * 0.08;
      }
    } else {
      if (this.controlled !== null) {
        this.panX = this.lastControlledX;
        this.controlled = null;
      }
      if (this.fpsModel) this.fpsModel.visible = false;
      this.updateCommander();
    }
    this.renderer.render(
      this.scene,
      possessed ? this.firstPerson : this.commander,
    );
  }
  pick(clientX: number, clientY: number): number | null {
    const rect = this.canvas.getBoundingClientRect();
    this.raycaster.setFromCamera(
      new THREE.Vector2(
        ((clientX - rect.left) / rect.width) * 2 - 1,
        (-(clientY - rect.top) / rect.height) * 2 + 1,
      ),
      this.commander,
    );
    const hits = this.raycaster.intersectObjects(
      [...this.actors.values()].map((a) => a.model),
      true,
    );
    return hits[0]?.object.userData.unitId ?? null;
  }
  clear(): void {
    for (const actor of this.actors.values()) {
      this.removeActor(actor);
    }
    this.actors.clear();
    for (const arrow of this.arrows.values()) this.scene.remove(arrow);
    this.arrows.clear();
    for (const corpse of this.corpses) {
      corpse.mixer?.stopAllAction();
      this.disposeSkeletons(corpse.root);
      this.scene.remove(corpse.root);
    }
    this.corpses = [];
    this.selectedId = null;
    this.controlled = null;
    if (this.fpsModel) this.fpsModel.visible = false;
    this.panX = -20;
    this.viewHeight = this.config.camera.viewHeight;
  }
  diagnostics() {
    return {
      actors: this.actors.size,
      arrows: this.arrows.size,
      corpses: this.corpses.length,
      geometries: this.renderer.info.memory.geometries,
      textures: this.renderer.info.memory.textures,
      drawCalls: this.renderer.info.render.calls,
      renderedFrames: this.renderer.info.render.frame,
    };
  }
}

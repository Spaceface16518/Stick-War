import { test, expect } from "@playwright/test";

test("exported bow variants keep the grip, drawing hand and arrow aligned", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open sandbox ◇" }).waitFor();
  const poses = await page.evaluate(async () => {
    const assetsPath = "/src/presentation/assets.ts",
      threePath = "/node_modules/.vite/deps/three.js";
    const { AssetStore } = (await import(
      assetsPath
    )) as typeof import("../src/presentation/assets");
    const T = (await import(threePath)) as typeof import("three");
    const library = (await (
      await fetch("/config/animation.json")
    ).json()) as typeof import("../config/animation.json");
    const store = new AssetStore();
    await store.load(() => {});
    const results = [];
    for (const asset of ["archer", "archer_lod", "fp_archer"])
      for (const name of ["bow_quick", "bow_draw", "bow_hold"] as const) {
        const { root, clips } = store.instantiate(asset)!;
        const mixer = new T.AnimationMixer(root),
          clip = clips.find((c) => c.name === name)!;
        mixer.clipAction(clip).play();
        for (const phase of [0.87, 0.94, 1].map(
          (p) => p * library.attacks[name].contact,
        )) {
          mixer.setTime(phase * clip.duration);
          root.updateMatrixWorld(true);
          const position = (name: string) =>
            root.getObjectByName(name)!.getWorldPosition(new T.Vector3());
          const grip = position("bow_socket"),
            hand = position("hand_l"),
            nock = position("string_nock"),
            drawHand = position("hand_r");
          results.push({
            asset,
            name,
            gripGap: grip.distanceTo(hand),
            draw: grip.distanceTo(nock),
            drawHandGap: nock.distanceTo(drawHand),
            verticalError: Math.abs(grip.y - nock.y),
            forward: (grip.z - nock.z) * (asset.startsWith("fp_") ? -1 : 1),
          });
        }
        mixer.stopAllAction();
        mixer.uncacheRoot(root);
      }
    return results;
  });
  for (const pose of poses) {
    const label = pose.asset + ":" + pose.name;
    expect(pose.gripGap, label + " grip").toBeLessThan(0.04);
    expect(pose.draw, label + " draw").toBeGreaterThan(0.25);
    expect(pose.drawHandGap, label + " string hand").toBeLessThan(0.04);
    expect(pose.verticalError, label + " level arrow").toBeLessThan(0.08);
    expect(pose.forward, label + " forward aim").toBeGreaterThan(0.25);
  }
});

test("exported footsteps plant in world space in forward, backward and lateral movement", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open sandbox ◇" }).waitFor();
  const feet = await page.evaluate(async () => {
    const assetsPath = "/src/presentation/assets.ts",
      threePath = "/node_modules/.vite/deps/three.js";
    const { AssetStore } = (await import(
      assetsPath
    )) as typeof import("../src/presentation/assets");
    const T = (await import(threePath)) as typeof import("three");
    const library = (await (
      await fetch("/config/animation.json")
    ).json()) as typeof import("../config/animation.json");
    const store = new AssetStore();
    await store.load(() => {});
    const results = [];
    for (const asset of ["miner", "swordsman", "archer", "swordsman_lod"])
      for (const name of [
        "walk",
        "run",
        "carry",
        "walk_back",
        "strafe_left",
        "strafe_right",
      ] as const) {
        const { root, clips } = store.instantiate(asset)!;
        const mixer = new T.AnimationMixer(root),
          clip = clips.find((c) => c.name === name)!;
        mixer.clipAction(clip).play();
        const points: import("three").Vector3[] = [];
        for (const phase of name === "run"
          ? [0.08, 0.2, 0.32]
          : [0.17, 0.3, 0.43]) {
          mixer.setTime(phase * clip.duration);
          const travel = phase * library.locomotion[name].distance;
          root.position.set(
            name === "strafe_left"
              ? travel
              : name === "strafe_right"
                ? -travel
                : 0,
            0,
            name.startsWith("strafe")
              ? 0
              : name === "walk_back"
                ? -travel
                : travel,
          );
          root.updateMatrixWorld(true);
          points.push(
            root.getObjectByName("foot_l")!.getWorldPosition(new T.Vector3()),
          );
        }
        results.push({
          asset,
          name,
          slide: Math.max(
            ...points.map((p) =>
              Math.hypot(p.x - points[0].x, p.z - points[0].z),
            ),
          ),
        });
        mixer.stopAllAction();
        mixer.uncacheRoot(root);
      }
    return results;
  });
  for (const foot of feet)
    expect(foot.slide, foot.asset + ":" + foot.name).toBeLessThan(0.015);
});

test("authored falls settle onto the floor without burying bodies or held weapons", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open sandbox ◇" }).waitFor();
  const falls = await page.evaluate(async () => {
    const assetsPath = "/src/presentation/assets.ts",
      threePath = "/node_modules/.vite/deps/three.js";
    const { AssetStore } = (await import(
      assetsPath
    )) as typeof import("../src/presentation/assets");
    const T = (await import(threePath)) as typeof import("three");
    const store = new AssetStore();
    await store.load(() => {});
    const results = [];
    for (const asset of ["miner", "swordsman", "archer"])
      for (const name of [
        "death_front",
        "death_back",
        "death_left",
        "death_right",
      ]) {
        const { root, clips } = store.instantiate(asset)!;
        const mixer = new T.AnimationMixer(root),
          clip = clips.find((c) => c.name === name)!;
        const action = mixer.clipAction(clip);
        action.setLoop(T.LoopOnce, 1);
        action.clampWhenFinished = true;
        action.play();
        let lowest = Infinity,
          highestContact = -Infinity;
        for (const phase of [0.0, 0.13, 0.28, 0.37, 0.56, 0.72, 0.88, 1]) {
          mixer.setTime(phase * clip.duration);
          root.updateMatrixWorld(true);
          let minY = Infinity;
          const p = new T.Vector3();
          root.traverse((o) => {
            if (o instanceof T.SkinnedMesh) {
              for (let i = 0; i < o.geometry.attributes.position.count; i++)
                minY = Math.min(
                  minY,
                  o.getVertexPosition(i, p).applyMatrix4(o.matrixWorld).y,
                );
            }
          });
          lowest = Math.min(lowest, minY);
          highestContact = Math.max(highestContact, minY);
        }
        results.push({ asset, name, lowest, highestContact });
        mixer.stopAllAction();
        mixer.uncacheRoot(root);
      }
    return results;
  });
  for (const fall of falls) {
    expect(
      fall.lowest,
      fall.asset + ":" + fall.name + " floor",
    ).toBeGreaterThan(-0.012);
    expect(
      fall.highestContact,
      fall.asset + ":" + fall.name + " support",
    ).toBeLessThan(0.025);
  }
});

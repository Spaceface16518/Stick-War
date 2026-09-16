import fs from "node:fs";
import assert from "node:assert/strict";
import validator from "gltf-validator";
import { NodeIO } from "@gltf-transform/core";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions";
import { MeshoptDecoder } from "meshoptimizer";
await MeshoptDecoder.ready;
const io = new NodeIO()
  .registerExtensions(ALL_EXTENSIONS)
  .registerDependencies({ "meshopt.decoder": MeshoptDecoder });
const manifest = JSON.parse(
  fs.readFileSync("public/assets/manifest.json", "utf8"),
);
const required = [
  "miner",
  "swordsman",
  "archer",
  "miner_lod",
  "swordsman_lod",
  "archer_lod",
  "fp_swordsman",
  "fp_archer",
  "arena",
];
assert.deepEqual(
  manifest.assets.map((a) => a.id).sort(),
  required.sort(),
  "Required asset bindings",
);
const arena = JSON.parse(fs.readFileSync("config/arena.json", "utf8"));
const animation = JSON.parse(fs.readFileSync("config/animation.json", "utf8"));
const report = [];
for (const asset of manifest.assets) {
  assert.equal(asset.file, asset.id + ".glb", "Local asset paths only");
  const bytes = fs.readFileSync("public/assets/" + asset.file);
  assert.equal(bytes.length, asset.bytes, `${asset.id}: stale byte size`);
  const result = await validator.validateBytes(new Uint8Array(bytes));
  assert.equal(
    result.issues.numErrors,
    0,
    `${asset.id}: ${JSON.stringify(result.issues.messages)}`,
  );
  const document = await io.readBinary(new Uint8Array(bytes));
  const root = document.getRoot();
  const triangles = root
    .listMeshes()
    .flatMap((m) => m.listPrimitives())
    .reduce(
      (n, p) =>
        n +
        (p.getIndices()?.getCount() ?? p.getAttribute("POSITION").getCount()) /
          3,
      0,
    );
  const clips = root.listAnimations().map((a) => a.getName());
  const joints = root
    .listSkins()
    .flatMap((s) => s.listJoints())
    .map((j) => j.getName());
  assert.ok(
    root.listTextures().every((t) => t.getSize()?.every((n) => n <= 1024)),
    `${asset.id}: texture budget`,
  );
  assert.ok(root.listMaterials().length <= 2, `${asset.id}: material budget`);
  if (!asset.id.startsWith("fp_") && asset.id !== "arena") {
    assert.ok(
      triangles <= (asset.id.endsWith("_lod") ? 2000 : 8000),
      `${asset.id}: triangle budget`,
    );
    const gaitClips = Object.keys(animation.locomotion).filter(
      (name) => name !== "runThreshold",
    );
    for (const clip of [
      ...gaitClips,
      "death_front",
      "death_back",
      "death_left",
      "death_right",
    ])
      assert.ok(clips.includes(clip), `${asset.id}: missing ${clip}`);
    if (asset.id.startsWith("miner"))
      assert.ok(clips.includes("mine"), "Miner must have mining action");
    for (const joint of [
      "root",
      "head",
      "hand_l",
      "hand_r",
      "weapon_socket",
      "bow_socket",
      "arrow_socket",
      "eye_socket",
      "toe_l",
      "toe_r",
    ])
      assert.ok(joints.includes(joint), `${asset.id}: missing ${joint}`);
    assert.ok(
      root.listMaterials().some((m) => m.getName() === "Team cloth"),
      "Team material must survive optimization",
    );
    assert.ok(
      root.listMaterials().some((m) => m.getName() === "Painted palette"),
      "Palette material must stay separate from team tint",
    );
  }
  if (asset.id === "arena") {
    for (const team of ["blue", "red"])
      for (const type of ["statue", "mine", "spawn"]) {
        const node = root
          .listNodes()
          .find((n) => n.getName() === `${team}_${type}`);
        assert.ok(node, `Missing arena anchor ${team}_${type}`);
        const p = node.getWorldTranslation(),
          expected = arena.teams[team][type];
        assert.ok(
          Math.abs(p[0] - expected.x) < 0.001 &&
            Math.abs(p[2] - expected.z) < 0.001,
          `Anchor mismatch ${team}_${type}`,
        );
      }
  }
  if (asset.id.startsWith("fp_")) {
    for (const name of [
      "hand_l",
      "hand_r",
      "weapon_socket",
      "bow_socket",
      "string_nock",
      "arrow_socket",
    ])
      assert.ok(joints.includes(name), `${asset.id}: missing ${name}`);
  }
  if (asset.id !== "arena") {
    const prefix = asset.id.includes("archer")
      ? "bow_"
      : asset.id.includes("swordsman")
        ? "melee_"
        : null;
    const attacks = Object.keys(animation.attacks).filter(
      (name) => prefix && name.startsWith(prefix),
    );
    for (const name of [
      "idle",
      "hit_front",
      "hit_back",
      "hit_left",
      "hit_right",
      ...attacks,
    ])
      assert.ok(clips.includes(name), `${asset.id}: missing ${name}`);
    for (const clip of root.listAnimations()) {
      const times = clip
        .listSamplers()
        .map((sampler) => sampler.getInput().getArray());
      const start = Math.min(...times.map((t) => t[0]));
      const end = Math.max(...times.map((t) => t[t.length - 1]));
      const expected =
        animation.attacks[clip.getName()]?.seconds ??
        animation.locomotion[clip.getName()]?.seconds ??
        (clip.getName().startsWith("hit_")
          ? animation.hitSeconds
          : clip.getName().startsWith("death_")
            ? animation.deathSeconds
            : null);
      assert.ok(
        Math.abs(start) < 1e-5,
        `${asset.id}: ${clip.getName()} must start at zero`,
      );
      if (expected !== null)
        assert.ok(
          Math.abs(end - expected) < 1e-5,
          `${asset.id}: ${clip.getName()} duration differs from animation contract`,
        );
      assert.ok(
        clip
          .listChannels()
          .every((channel) =>
            joints.includes(channel.getTargetNode()?.getName()),
          ),
        `${asset.id}: actions must target rig bones, never mesh geometry`,
      );
    }
    const nock = root
      .listNodes()
      .find((node) => node.getName() === "string_nock");
    assert.equal(
      nock?.getParentNode()?.getName(),
      "bow_socket",
      `${asset.id}: string follows the bow through layered motion`,
    );
  }
  const entry = {
    id: asset.id,
    bytes: bytes.length,
    triangles,
    materials: root.listMaterials().length,
    clips: clips.length,
    joints: joints.length,
    validatorErrors: result.issues.numErrors,
  };
  report.push(entry);
  console.log(
    `${asset.id}: ${triangles} triangles; ${entry.materials} materials; ${bytes.length} bytes; valid`,
  );
}
if (process.argv.includes("--report"))
  fs.writeFileSync(
    "docs/asset-validation.json",
    JSON.stringify(report, null, 2) + "\n",
  );

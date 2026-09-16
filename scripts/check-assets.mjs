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
    for (const clip of [
      "idle",
      "walk",
      "carry",
      "mine",
      "melee_attack",
      "bow_attack",
      "hit",
      "death",
    ])
      assert.ok(clips.includes(clip), `${asset.id}: missing ${clip}`);
    for (const joint of [
      "root",
      "head",
      "hand_l",
      "hand_r",
      "weapon_socket",
      "bow_socket",
      "arrow_socket",
      "eye_socket",
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
      "idle",
      asset.id === "fp_archer" ? "bow_attack" : "melee_attack",
    ])
      assert.ok(clips.includes(name), `${asset.id}: missing ${name}`);
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

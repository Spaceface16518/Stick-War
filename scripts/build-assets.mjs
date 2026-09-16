import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { NodeIO } from "@gltf-transform/core";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions";
import {
  dedup,
  resample,
  prune,
  weld,
  meshopt,
} from "@gltf-transform/functions";
import { MeshoptEncoder, MeshoptDecoder } from "meshoptimizer";
const selectedIndex = process.argv.indexOf("--asset");
const selected = selectedIndex < 0 ? null : process.argv[selectedIndex + 1];
if (!process.argv.includes("--exports-only")) {
  const blender =
    process.env.BLENDER_PATH ||
    (process.platform === "darwin"
      ? "/Applications/Blender.app/Contents/MacOS/Blender"
      : "blender");
  const result = spawnSync(
    blender,
    [
      "--background",
      "--factory-startup",
      "--python",
      "art/build_scene.py",
      "--",
      ...process.argv.slice(2).filter((arg) => arg !== "--exports-only"),
    ],
    { stdio: "inherit", env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" } },
  );
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`Blender failed: ${result.status ?? result.signal}`);
}
await Promise.all([MeshoptEncoder.ready, MeshoptDecoder.ready]);
const io = new NodeIO()
  .registerExtensions(ALL_EXTENSIONS)
  .registerDependencies({
    "meshopt.encoder": MeshoptEncoder,
    "meshopt.decoder": MeshoptDecoder,
  });
fs.mkdirSync("public/assets", { recursive: true });
for (const file of fs
  .readdirSync("art/export")
  .filter(
    (f) =>
      f.endsWith(".glb") &&
      (!selected || f === selected + ".glb" || f === selected + "_lod.glb"),
  )
  .sort()) {
  const document = await io.read("art/export/" + file);
  // Blender's unsampled action export can retain frame 1's offset even with
  // slide-to-zero enabled. Normalize the whole clip, preserving relative keys.
  for (const clip of document.getRoot().listAnimations()) {
    const samplers = clip.listSamplers();
    const start = Math.min(...samplers.map((s) => s.getInput().getArray()[0]));
    if (!(start > 0)) continue;
    const shifted = new Map();
    for (const sampler of samplers) {
      const input = sampler.getInput();
      if (!shifted.has(input))
        shifted.set(
          input,
          input
            .clone()
            .setArray(
              Float32Array.from(input.getArray(), (t) =>
                Math.max(0, t - start),
              ),
            ),
        );
      sampler.setInput(shifted.get(input));
    }
  }
  await document.transform(
    dedup({ keepUniqueNames: true }),
    weld(),
    resample(),
    prune({ keepLeaves: true }),
    meshopt({ encoder: MeshoptEncoder, level: "medium" }),
  );
  await io.write("public/assets/" + file, document);
  const bytes = fs.statSync("public/assets/" + file).size;
  console.log(`${file}: ${(bytes / 1024).toFixed(1)} KiB`);
}
// A single-source export must keep the other committed asset bindings, even
// in a fresh checkout with no previous scratch exports.
const assets = fs
  .readdirSync("public/assets")
  .filter((f) => f.endsWith(".glb"))
  .sort()
  .map((file) => ({
    id: file.slice(0, -4),
    file,
    bytes: fs.statSync("public/assets/" + file).size,
  }));
fs.writeFileSync(
  "public/assets/manifest.json",
  JSON.stringify({ version: 1, assets }, null, 2) + "\n",
);
if (!selected || selected === "arena") {
  const sourceArena = JSON.parse(
    fs.readFileSync("art/export/arena.json", "utf8"),
  );
  const runtimeArena = JSON.parse(fs.readFileSync("config/arena.json", "utf8"));
  if (JSON.stringify(sourceArena) !== JSON.stringify(runtimeArena))
    throw new Error(
      "Blender anchors differ from config/arena.json. Review and copy art/export/arena.json explicitly.",
    );
}

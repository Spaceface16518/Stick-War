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
if (!process.argv.includes("--exports-only")) {
  const blender =
    process.env.BLENDER_PATH ||
    "/Applications/Blender.app/Contents/MacOS/Blender";
  const result = spawnSync(
    blender,
    ["--background", "--factory-startup", "--python", "art/build_scene.py"],
    { stdio: "inherit" },
  );
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
const assets = [];
fs.mkdirSync("public/assets", { recursive: true });
for (const file of fs
  .readdirSync("art/export")
  .filter((f) => f.endsWith(".glb"))
  .sort()) {
  const document = await io.read("art/export/" + file);
  await document.transform(
    dedup({ keepUniqueNames: true }),
    weld(),
    resample(),
    prune({ keepLeaves: true }),
    meshopt({ encoder: MeshoptEncoder, level: "medium" }),
  );
  await io.write("public/assets/" + file, document);
  const bytes = fs.statSync("public/assets/" + file).size;
  assets.push({ id: file.slice(0, -4), file, bytes });
  console.log(`${file}: ${(bytes / 1024).toFixed(1)} KiB`);
}
fs.writeFileSync(
  "public/assets/manifest.json",
  JSON.stringify({ version: 1, assets }, null, 2) + "\n",
);
const sourceArena = JSON.parse(
  fs.readFileSync("art/export/arena.json", "utf8"),
);
const runtimeArena = JSON.parse(fs.readFileSync("config/arena.json", "utf8"));
if (JSON.stringify(sourceArena) !== JSON.stringify(runtimeArena))
  throw new Error(
    "Blender anchors differ from config/arena.json. Review and copy art/export/arena.json explicitly.",
  );

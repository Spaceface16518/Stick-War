import fs from "node:fs";
import validator from "gltf-validator";
const manifest = JSON.parse(
  fs.readFileSync("public/assets/manifest.json", "utf8"),
);
for (const asset of manifest.assets) {
  const bytes = fs.readFileSync("public/assets/" + asset.file);
  if (bytes.length !== asset.bytes)
    throw new Error(`${asset.id}: stale manifest byte size`);
  const result = await validator.validateBytes(new Uint8Array(bytes));
  if (result.issues.numErrors)
    throw new Error(`${asset.id}: ${JSON.stringify(result.issues.messages)}`);
  console.log(`${asset.id}: valid GLB, ${bytes.length} bytes`);
}

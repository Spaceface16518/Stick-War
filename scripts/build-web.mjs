import { cp, mkdir, readdir, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { join, relative } from "node:path";

const run = (command, args) =>
  new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: "inherit" });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} exited with status ${code}`));
    });
  });

const collectRuntimeAssets = async (directory) => {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await collectRuntimeAssets(path)));
    else if (/\.(png|ttf)$/i.test(entry.name)) files.push(relative("assets", path));
  }
  return files.sort();
};

await rm("dist", { recursive: true, force: true });
await mkdir("dist", { recursive: true });
await cp("web", "dist", { recursive: true });
await cp("assets", "dist/assets", { recursive: true });
await writeFile(
  "dist/assets/manifest.json",
  `${JSON.stringify(await collectRuntimeAssets("assets"), null, 2)}\n`,
);

await run("cargo", [
  "build",
  "--locked",
  "--profile",
  "wasm-release",
  "--target",
  "wasm32-unknown-unknown",
  "--no-default-features",
]);

await run("wasm-bindgen", [
  "--out-name",
  "stick_war",
  "--out-dir",
  "dist/pkg",
  "--target",
  "web",
  "target/wasm32-unknown-unknown/wasm-release/stick-war.wasm",
]);

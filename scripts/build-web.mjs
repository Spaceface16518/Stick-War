import { cp, mkdir, rm } from "node:fs/promises";
import { spawn } from "node:child_process";

const run = (command, args) =>
  new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: "inherit" });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} exited with status ${code}`));
    });
  });

await rm("dist", { recursive: true, force: true });
await mkdir("dist", { recursive: true });
await cp("web", "dist", { recursive: true });

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

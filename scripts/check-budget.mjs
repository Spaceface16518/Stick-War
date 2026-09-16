import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";
function files(dir) {
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .flatMap((e) =>
      e.isDirectory()
        ? files(path.join(dir, e.name))
        : [path.join(dir, e.name)],
    );
}
const entries = files("dist");
const raw = entries.reduce((n, p) => n + fs.statSync(p).size, 0);
const gzip = entries.reduce(
  (n, p) => n + gzipSync(fs.readFileSync(p)).byteLength,
  0,
);
console.log(
  `Complete site: ${(raw / 1048576).toFixed(2)} MiB raw, ${(gzip / 1048576).toFixed(2)} MiB gzip estimate`,
);
if (raw > 12 * 1024 * 1024)
  throw new Error("Complete site exceeds the 12 MiB raw download budget");

# Developing on amrit-max-n

The Linux checkout is `/home/amrit/Documents/projects/stick-war`, based on
`codex/3d-rewrite` at `542cc7d`. Workstation setup changes are on
`codex/amrit-max-n-setup`. The older Rust checkout on the Mac is separate.

## Open and edit

On the Linux desktop, the application menu includes **Stick War — Play**,
**Stick War — Blender**, and **Stick War — Codex**. Blender opens the swordsman;
the other editable models are in `art/sources/`.

From a terminal on that machine:

```sh
cd ~/Documents/projects/stick-war
codex
```

`stick-war-codex` also opens Codex directly in this checkout. The existing Codex
CLI is authenticated and has been checked in a read-only session against this
repository. Code, configuration, and Blender sources are ordinary editable
files owned by `amrit`. Git author details are configured only in this repo.

## Play and live reload

Open <http://127.0.0.1:5174> in the Linux browser. A user service runs Vite on
loopback; edits to code/configuration reload the page. The service starts with
the user's login session, restarts after a crash, and does not require keeping
a terminal open. It is not configured to run before login or after the last
user session ends.

```sh
systemctl --user status stick-war-dev
systemctl --user restart stick-war-dev
journalctl --user -u stick-war-dev -n 50 --no-pager
systemctl --user stop stick-war-dev
```

The service file is `~/.config/systemd/user/stick-war-dev.service`; application
shortcuts are in `~/.local/share/applications/stick-war-*.desktop`.

To play from your Mac over Tailscale, keep this command running in a Mac terminal:

```sh
ssh -N -o ExitOnForwardFailure=yes -L 127.0.0.1:5174:127.0.0.1:5174 amrit@100.73.1.97
```

Then open <http://127.0.0.1:5174> on the Mac. Stop the tunnel with Ctrl+C.
`amrit-max-n.shrimp-tegu.ts.net` can replace the IP when MagicDNS resolves.
Both ends listen only on localhost; no tailnet HTTP/HTTPS endpoint is enabled.

For a foreground server, use `npm run dev` (port 5173). Automated browser tests
use 5173 as well, separately from the persistent server on 5174.

## Blender round trip

```sh
cd ~/Documents/projects/stick-war
blender art/sources/swordsman.blend
# Save the Blender edit, then:
npm run assets:build -- --asset swordsman
npm run assets:check
```

Reload the browser to see the exported model. Export writes the runtime GLB,
its commander LOD, and the asset manifest; it does not save/regenerate the
Blender source. Commit the edited `.blend` and generated runtime assets together.
Use `assets:animate` or `assets:author` only when intentionally replacing authored
actions or geometry. See [the art guide](../art/README.md).

Blender 5.2.2 LTS is installed at `/snap/bin/blender`. Linux uses `blender` from
PATH automatically; `BLENDER_PATH` can override it.

## Verify changes

```sh
npm run check
npm run lint
npm test
npm run assets:check
npm run build:web
npm run test:browser
npx playwright show-report
```

Browser tests select installed Linux Chromium, including `/snap/bin/chromium`.
Use `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH` to override it. CI instead uses the
Playwright-managed browser. The browser download timed out during this setup;
the existing Chromium was used successfully. Screenshots and traces are in
ignored `test-results/`, and the HTML report is in `playwright-report/`.

Node 22.23.2 is isolated in `~/.local/share/stick-war/`, with `node`, `npm`, and
`npx` links in `~/.local/bin/`. Normal login shells include this directory.
For a non-login SSH command, explicitly set `PATH="$HOME/.local/bin:$PATH"`.
Dependencies are project-local and installed with `npm ci` from the lockfile.

## Save work

Create feature branches, review `git diff`, and commit specific changed files.
The origin is `https://github.com/Spaceface16518/Stick-War.git`. Public fetches
work without credentials; pushing from Linux needs your own GitHub authentication.
No GitHub credentials were copied from the Mac. The setup branch was pushed
using the Mac's existing authentication.

## Setup verification (2026-09-16)

- TypeScript, ESLint, production build, and all 62 unit tests passed on this host.
- Asset validation passed for all nine GLBs. A swordsman export in an isolated
  copy produced valid close/LOD assets, with all six source `.blend` hashes
  unchanged and output sizes matching the committed assets.
- Chromium browser tests: 34 passed, with two intentional platform-specific
  skips. Desktop and landscape touch screenshots were inspected. Touch
  emulation does not establish performance on a physical phone.
- The Blender source and live game were opened on the Linux desktop. Codex CLI
  0.154.0 successfully read the repo and ran development commands.
- The localhost server and Mac-to-Linux SSH tunnel returned the live app.

To disable the background server permanently:

```sh
systemctl --user disable --now stick-war-dev
```

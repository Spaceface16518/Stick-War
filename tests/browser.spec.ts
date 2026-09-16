import { test, expect, type Page } from "@playwright/test";
import type { BattleCommand, BattleSnapshot } from "../src/simulation/types";
import type { GameConfig } from "../src/content/config";
declare global {
  interface Window {
    __stickWar: {
      snapshot: () => BattleSnapshot | null;
      command: (c: BattleCommand) => unknown;
      advance: (seconds: number) => void;
      configure: (c: GameConfig) => void;
      config: () => GameConfig;
      diagnostics: () => {
        simulation: { units: number; colliders: number } | null;
        rendering: { actors: number; geometries: number; textures: number };
      };
    };
  }
}
const state = (page: Page) =>
  page.evaluate(() => window.__stickWar.snapshot()!);
const command = (page: Page, c: BattleCommand) =>
  page.evaluate((c) => window.__stickWar.command(c), c);
async function open(page: Page, mode = "sandbox") {
  await page.goto("/");
  await page
    .getByRole("button", {
      name: mode === "sandbox" ? "Open sandbox ◇" : "Begin battle →",
    })
    .click();
}
test("sandbox placement, orders, possession, pause, restart and menu", async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await open(page);
  await expect(
    page.getByText("SIMULATION PAUSED", { exact: true }),
  ).toBeVisible();
  await page.locator("#train-swordsman").click();
  await page.locator("#train-archer").click();
  expect((await state(page)).units).toHaveLength(4);
  for (const order of ["attack", "defend", "retreat"] as const) {
    await page.locator("#order-" + order).click();
    expect((await state(page)).teams.blue.order).toBe(order);
  }
  await page.locator("#team-red").click();
  await page.locator("#train-swordsman").click();
  expect(
    (await state(page)).units.filter((u) => u.team === "red"),
  ).toHaveLength(2);
  await page.locator("#team-blue").click();
  await page.locator("#order-attack").click();
  await page.screenshot({ path: info.outputPath("commander.png") });
  await page.locator("#sandbox-pause").click();
  await page.keyboard.press("Tab");
  await expect(page.locator("#pov")).toBeVisible();
  const id = (await state(page)).controlledId;
  await page.keyboard.down("KeyW");
  await expect
    .poll(async () => (await state(page)).units.find((u) => u.id === id)!.x)
    .toBeGreaterThan(-25.9);
  await page.keyboard.up("KeyW");
  await page.keyboard.press("Space");
  await page.screenshot({ path: info.outputPath("swordsman-pov.png") });
  await page.keyboard.press("Tab");
  await expect(page.locator("#pov-kind")).toHaveText("ARCHER");
  await page.screenshot({ path: info.outputPath("archer-pov.png") });
  await page.keyboard.press("Escape");
  await expect(page.locator("#pov")).toBeHidden();
  await page.getByRole("button", { name: "Pause menu" }).click();
  await page.getByRole("button", { name: "Restart battle" }).click();
  expect((await state(page)).units).toHaveLength(2);
  expect((await state(page)).paused).toBe(true);
  await page.getByRole("button", { name: "Pause menu" }).click();
  await page.getByRole("button", { name: "Main menu", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Begin battle →" }),
  ).toBeVisible();
  expect(errors).toEqual([]);
});
test("complete victory and defeat through the browser runtime", async ({
  page,
}) => {
  await open(page, "skirmish");
  await page.evaluate(() => {
    const c = window.__stickWar.config();
    c.units.swordsman.cost = 0;
    c.units.swordsman.trainingSeconds = 0;
    c.units.swordsman.damage = 200;
    c.ai.recruitSeconds = 999;
    window.__stickWar.configure(c);
    for (let i = 0; i < 8; i++)
      window.__stickWar.command({
        type: "train",
        team: "blue",
        kind: "swordsman",
      });
    window.__stickWar.command({ type: "order", team: "blue", order: "attack" });
    window.__stickWar.advance(90);
  });
  await expect(page.getByText("The field is yours.")).toBeVisible();
  await page.getByRole("button", { name: "Main menu", exact: true }).click();
  await page.reload();
  await page.getByRole("button", { name: "Begin battle →" }).click();
  await page.evaluate(() => window.__stickWar.advance(160));
  await expect(page.getByText("Your statue has fallen.")).toBeVisible();
});
test("restart does not accumulate battle objects or GPU assets", async ({
  page,
}) => {
  await open(page);
  let baseline = 0,
    textures = 0;
  for (let n = 0; n < 5; n++) {
    await page.locator("#train-swordsman").click();
    await page.locator("#train-archer").click();
    await expect
      .poll(
        async () =>
          (await page.evaluate(() => window.__stickWar.diagnostics())).rendering
            .actors,
      )
      .toBe(4);
    const d = await page.evaluate(() => window.__stickWar.diagnostics());
    if (n === 0) {
      baseline = d.rendering.geometries;
      textures = d.rendering.textures;
    } else {
      expect(d.rendering.geometries).toBe(baseline);
      expect(d.rendering.textures).toBe(textures);
    }
    await page.getByRole("button", { name: "Pause menu" }).click();
    await page.getByRole("button", { name: "Restart battle" }).click();
    await expect
      .poll(
        async () =>
          (await page.evaluate(() => window.__stickWar.diagnostics())).rendering
            .actors,
      )
      .toBe(2);
    expect(
      (await page.evaluate(() => window.__stickWar.diagnostics())).simulation!
        .colliders,
    ).toBe(9);
  }
});
test("asset-loading failures offer a working retry", async ({ page }) => {
  await page.route("**/assets/manifest.json", (route) =>
    route.fulfill({ status: 503, body: "Unavailable" }),
  );
  await page.goto("/");
  await expect(page.getByRole("button", { name: "Try again" })).toBeVisible();
  await page.unroute("**/assets/manifest.json");
  await page.getByRole("button", { name: "Try again" }).click();
  await expect(
    page.getByRole("button", { name: "Begin battle →" }),
  ).toBeVisible();
});
test("landscape touch supports movement, aiming and firing together", async ({
  page,
}, info) => {
  test.skip(info.project.name !== "touch", "Multi-touch device scenario");
  await open(page);
  await page.locator("#train-archer").click();
  await page.locator("#sandbox-pause").click();
  await page.keyboard.press("Tab");
  await expect(page.locator(".attack-touch")).toBeVisible();
  const initial = await state(page);
  const before = initial.units.find((u) => u.id === initial.controlledId);
  expect(before).toBeDefined();
  const move = page.locator(".joystick"),
    attack = page.locator(".attack-touch");
  const mb = await move.boundingBox(),
    ab = await attack.boundingBox();
  await move.dispatchEvent("pointerdown", {
    pointerId: 7,
    pointerType: "touch",
    clientX: mb!.x + 55,
    clientY: mb!.y + 55,
  });
  await move.dispatchEvent("pointermove", {
    pointerId: 7,
    pointerType: "touch",
    clientX: mb!.x + 55,
    clientY: mb!.y + 15,
  });
  await attack.dispatchEvent("pointerdown", {
    pointerId: 8,
    pointerType: "touch",
    clientX: ab!.x + 42,
    clientY: ab!.y + 42,
  });
  await attack.dispatchEvent("pointermove", {
    pointerId: 8,
    pointerType: "touch",
    clientX: ab!.x + 60,
    clientY: ab!.y + 35,
  });
  await expect
    .poll(async () => {
      const s = await state(page);
      return s.units.find((u) => u.id === s.controlledId)!.x;
    })
    .toBeGreaterThan(before!.x + 0.2);
  await expect
    .poll(async () => {
      const s = await state(page);
      return s.units.find((u) => u.id === s.controlledId)!.cooldown;
    })
    .toBeGreaterThan(0);
  await page.screenshot({ path: info.outputPath("touch-pov.png") });
  await move.dispatchEvent("pointercancel", {
    pointerId: 7,
    pointerType: "touch",
  });
  await attack.dispatchEvent("pointercancel", {
    pointerId: 8,
    pointerType: "touch",
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByText("Turn to landscape")).toBeVisible();
  expect((await state(page)).paused).toBe(true);
});
test("100-unit sandbox survives a simulation stress run", async ({ page }) => {
  await open(page);
  await command(page, { type: "sandbox", populationCap: 50 });
  await page.evaluate(() => {
    for (const team of ["blue", "red"] as const) {
      for (let i = 0; i < 49; i++)
        window.__stickWar.command({
          type: "train",
          team,
          kind: i % 3 ? "swordsman" : "archer",
        });
    }
    window.__stickWar.command({ type: "pause", paused: false });
    window.__stickWar.advance(15);
  });
  expect((await state(page)).units.length).toBeGreaterThan(50);
});

test("pointer-lock loss releases possession and entry cannot fire", async ({
  page,
}, info) => {
  test.skip(info.project.name !== "desktop", "Desktop pointer lock");
  await open(page);
  await page.locator("#train-swordsman").click();
  await page.locator("#sandbox-pause").click();
  await page.keyboard.press("Tab");
  await expect
    .poll(() => page.evaluate(() => document.pointerLockElement?.id))
    .toBe("battlefield");
  expect(
    (await state(page)).units.find((u) => u.kind === "swordsman")!.cooldown,
  ).toBe(0);
  await page.evaluate(() => document.exitPointerLock());
  await expect.poll(async () => (await state(page)).controlledId).toBeNull();
  await expect(page.locator("#pov")).toBeHidden();
});

test("valid balance reload updates future requests; invalid reload keeps prior values", async ({
  page,
}) => {
  const { readFile, writeFile } = await import("node:fs/promises");
  const configPath = new URL("../config/game.json", import.meta.url);
  const original = await readFile(configPath, "utf8");
  await open(page);
  try {
    const changed = JSON.parse(original);
    changed.units.swordsman.cost = 77;
    await writeFile(configPath, JSON.stringify(changed, null, 2) + "\n");
    await expect
      .poll(() =>
        page.evaluate(() => window.__stickWar.config().units.swordsman.cost),
      )
      .toBe(77);
    changed.economy.passiveSeconds = 0;
    await writeFile(configPath, JSON.stringify(changed, null, 2) + "\n");
    await expect(page.locator("#dev-error")).toContainText(
      "Balance reload rejected",
    );
    expect(
      await page.evaluate(
        () => window.__stickWar.config().economy.passiveSeconds,
      ),
    ).toBe(2);
    expect(
      await page.evaluate(
        () => window.__stickWar.config().units.swordsman.cost,
      ),
    ).toBe(77);
  } finally {
    await writeFile(configPath, original);
  }
  await expect
    .poll(() =>
      page.evaluate(() => window.__stickWar.config().units.swordsman.cost),
    )
    .toBe(100);
  await expect(page.locator("#dev-error")).toBeHidden();
});

test("rendered battle performance sample and static-menu idle", async ({
  page,
}, info) => {
  await open(page);
  const samples: unknown[] = [];
  const viewport = page.viewportSize()!;
  await page.mouse.move(viewport.width / 2, viewport.height * 0.6);
  await page.mouse.wheel(viewport.height, 0);
  await page.mouse.wheel(0, -350);
  await page.evaluate(() => {
    const c = window.__stickWar.config();
    c.units.swordsman.health = 100000;
    c.units.archer.health = 100000;
    window.__stickWar.configure(c);
  });
  for (const cap of [12, 50]) {
    await command(page, { type: "sandbox", populationCap: cap });
    await page.evaluate((cap) => {
      for (const team of ["blue", "red"] as const) {
        for (
          let i = window.__stickWar
            .snapshot()!
            .units.filter((u) => u.team === team).length;
          i < cap;
          i++
        )
          window.__stickWar.command({
            type: "train",
            team,
            kind: i % 3 ? "swordsman" : "archer",
          });
      }
      window.__stickWar.command({ type: "pause", paused: false });
      for (const team of ["blue", "red"] as const)
        window.__stickWar.command({ type: "order", team, order: "attack" });
      window.__stickWar.advance(10);
    }, cap);
    const timing = await page.evaluate(
      () =>
        new Promise<{ meanMs: number; p95Ms: number; samples: number }>(
          (resolve) => {
            const times: number[] = [];
            let last = 0;
            function frame(now: number) {
              if (last) times.push(now - last);
              last = now;
              if (times.length < 180) requestAnimationFrame(frame);
              else {
                const sorted = [...times].sort((a, b) => a - b);
                resolve({
                  meanMs: times.reduce((a, b) => a + b, 0) / times.length,
                  p95Ms: sorted[Math.floor(sorted.length * 0.95)],
                  samples: times.length,
                });
              }
            }
            requestAnimationFrame(frame);
          },
        ),
    );
    samples.push({
      requestedUnits: cap * 2,
      scenario:
        "Sustained combat centered in view; increased health prevents population decay",
      ...timing,
      diagnostics: await page.evaluate(() => window.__stickWar.diagnostics()),
    });
    await command(page, { type: "pause", paused: true });
    await page.screenshot({ path: info.outputPath(`combat-${cap * 2}.png`) });
  }
  const report = JSON.stringify(
    {
      project: info.project.name,
      viewport: page.viewportSize(),
      samples,
      physicalPhone: false,
    },
    null,
    2,
  );
  const { writeFile } = await import("node:fs/promises");
  await writeFile(info.outputPath("performance.json"), report);
  await info.attach("performance", {
    body: JSON.stringify(
      {
        project: info.project.name,
        viewport: page.viewportSize(),
        samples,
        physicalPhone: false,
      },
      null,
      2,
    ),
    contentType: "application/json",
  });
  await page.getByRole("button", { name: "Pause menu" }).click();
  await page.getByRole("button", { name: "Main menu", exact: true }).click();
  await expect
    .poll(
      async () =>
        (await page.evaluate(() => window.__stickWar.diagnostics())).rendering
          .actors,
    )
    .toBe(0);
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  );
  const before = await page.evaluate(() => window.__stickWar.diagnostics());
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        let n = 0;
        function wait() {
          if (++n === 20) resolve();
          else requestAnimationFrame(wait);
        }
        requestAnimationFrame(wait);
      }),
  );
  expect(await page.evaluate(() => window.__stickWar.diagnostics())).toEqual(
    before,
  );
});

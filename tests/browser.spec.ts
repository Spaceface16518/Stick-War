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
  let baseline = 0;
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
    if (n === 0) baseline = d.rendering.geometries;
    else expect(d.rendering.geometries).toBe(baseline);
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

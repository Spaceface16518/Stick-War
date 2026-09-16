import { expect, test, type Page } from "@playwright/test";
import type { BattleCommand } from "../src/simulation/types";

const state = (page: Page) =>
  page.evaluate(() => window.__stickWar.snapshot()!);
const command = (page: Page, command: BattleCommand) =>
  page.evaluate((command) => window.__stickWar.command(command), command);

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open sandbox ◇" }).waitFor();
});

test("settings persist across dialog, battle and menu transitions", async ({
  page,
}, info) => {
  await page.screenshot({ path: info.outputPath("menu.png") });
  await page.getByRole("button", { name: "Settings & controls" }).click();
  const sensitivity = page.getByLabel("Look sensitivity");
  await sensitivity.press("End");
  await sensitivity.press("ArrowLeft");
  await sensitivity.press("ArrowLeft");
  await page.getByLabel("Volume", { exact: true }).press("Home");
  await page.getByLabel("Volume", { exact: true }).press("ArrowRight");
  await page.getByLabel("Reduced motion").setChecked(true);
  await expect(sensitivity).toHaveValue("2.8");
  await expect(page.locator("body")).toHaveClass("reduced-motion");
  expect(
    await page.evaluate(() =>
      JSON.parse(localStorage.getItem("stick-war-settings-v1")!),
    ),
  ).toEqual({ sensitivity: 2.8, volume: 0.05, reducedMotion: true });
  await page.screenshot({ path: info.outputPath("settings.png") });
  await page.getByRole("button", { name: "Back", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeHidden();
  await page.reload();
  await page.getByRole("button", { name: "Settings & controls" }).click();
  await expect(sensitivity).toHaveValue("2.8");
  await expect(page.getByLabel("Volume", { exact: true })).toHaveValue("0.05");
  await expect(page.getByLabel("Reduced motion")).toBeChecked();
  await page.getByRole("button", { name: "Back", exact: true }).click();

  await page.getByRole("button", { name: "Open sandbox ◇" }).click();
  await page.locator("#sandbox-pause").click();
  await page.getByRole("button", { name: "Pause menu" }).click();
  await page.getByRole("button", { name: "Settings & controls" }).click();
  expect((await state(page)).paused).toBe(true);
  await expect(sensitivity).toHaveValue("2.8");
  await page.getByLabel("Reduced motion").uncheck();
  await page.getByRole("button", { name: "Return to battle" }).click();
  expect((await state(page)).paused).toBe(false);
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(page.locator("body")).not.toHaveClass("reduced-motion");
});

test("sandbox props preserve cap edits and collapsed tools, and reset on restart", async ({
  page,
}, info) => {
  await page.getByRole("button", { name: "Open sandbox ◇" }).click();
  const cap = page.getByLabel("Population cap");
  await cap.fill("2");
  await command(page, { type: "sandbox", costs: true });
  await expect(page.locator("#costs")).toHaveText("Costs on");
  await expect(cap).toBeFocused();
  await expect(cap).toHaveValue("2");
  expect((await state(page)).settings.populationCap).toBe(12);
  await cap.press("0");
  await page.locator("#team-red").click();
  await expect
    .poll(async () => (await state(page)).settings.populationCap)
    .toBe(20);
  await expect(cap).toHaveValue("20");
  await expect(page.locator("#team-red")).toHaveAttribute(
    "aria-pressed",
    "true",
  );

  await cap.fill("0");
  await cap.press("Tab");
  await expect(cap).toHaveValue("20");
  expect((await state(page)).settings.populationCap).toBe(20);
  expect((await state(page)).controlledId).toBeNull();

  const toggle = page.getByRole("button", { name: /^Sandbox/ });
  await toggle.click();
  await command(page, { type: "pause", paused: false });
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  await expect(cap).toBeHidden();
  await page.screenshot({ path: info.outputPath("sandbox-collapsed.png") });
  await toggle.click();
  await expect(page.locator("#sandbox-pause")).toHaveText("Ⅱ Pause");
  await expect(cap).toHaveValue("20");
  await page.locator("#sandbox-pause").click();
  await page.screenshot({ path: info.outputPath("sandbox-expanded.png") });
  await toggle.click();
  await page.getByRole("button", { name: "Pause menu" }).click();
  await page.getByRole("button", { name: "Restart battle" }).click();
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  await expect(cap).toHaveValue("12");
  await expect(page.locator("#team-blue")).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.locator("#costs")).toHaveText("Costs off");
});

test("training controls follow reservations, team, gold, time and cap changes", async ({
  page,
}, info) => {
  await page.getByRole("button", { name: "Open sandbox ◇" }).click();
  await page.getByLabel("Population cap").fill("2");
  await page.locator("#training-time").click();
  await expect(page.locator("#train-label-swordsman")).toHaveText("Free · 5s");
  await page.locator("#train-swordsman").click();
  await expect(page.locator("#train-label-swordsman")).toHaveText(
    "Training · 5.0s",
  );
  await expect(page.locator("#train-swordsman")).toBeDisabled();
  await expect(page.locator("#train-archer")).toBeDisabled();
  expect((await state(page)).training).toHaveLength(1);
  await page.locator("#costs").click();
  await page.locator("#training-time").click();
  await expect(page.locator("#train-label-swordsman")).toHaveText(
    "Training · 5.0s",
  );
  await page.locator("#team-red").click();
  await expect(page.locator("#train-archer")).toBeEnabled();
  await expect(page.locator("#train-label-archer")).toHaveText(
    "125 gold · instant",
  );
  await page.locator("#train-archer").click();
  expect((await state(page)).teams.red.gold).toBe(25);
  expect(
    (await state(page)).units.filter((u) => u.team === "red"),
  ).toHaveLength(2);
  await page.getByLabel("Population cap").fill("3");
  await page.getByLabel("Population cap").press("Tab");
  await expect(page.locator("#train-miner")).toBeDisabled();
  await page.locator("#costs").click();
  await expect(page.locator("#train-miner")).toBeEnabled();
  await page.locator("#team-blue").click();
  await expect(page.locator("#train-swordsman")).toBeDisabled();
  await page.evaluate(() => {
    window.__stickWar.command({ type: "pause", paused: false });
    window.__stickWar.advance(2);
    window.__stickWar.command({ type: "pause", paused: true });
  });
  await expect(page.locator("#train-label-swordsman")).toHaveText(
    "Training · 3.0s",
  );
  await page.screenshot({ path: info.outputPath("training-countdown.png") });
  await page.evaluate(() => {
    window.__stickWar.command({ type: "pause", paused: false });
    window.__stickWar.advance(3.1);
    window.__stickWar.command({ type: "pause", paused: true });
  });
  await expect(page.locator("#train-swordsman")).toBeEnabled();
  expect((await state(page)).training).toHaveLength(0);
  expect(
    (await state(page)).units.filter((u) => u.team === "blue"),
  ).toHaveLength(2);
});

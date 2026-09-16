import { defineConfig } from "@playwright/test";
import { existsSync } from "node:fs";

// Reuse a Linux desktop browser when available. CI keeps Playwright's browser.
const executablePath =
  process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH ||
  (!process.env.CI && process.platform === "linux"
    ? [
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
      ].find(existsSync)
    : undefined);
export default defineConfig({
  testDir: "tests",
  testMatch: "**/*.spec.ts",
  fullyParallel: false,
  workers: 1,
  timeout: process.env.CI ? 120000 : 45000,
  expect: { timeout: process.env.CI ? 15000 : 5000 },
  use: {
    baseURL: "http://127.0.0.1:5173",
    screenshot: "only-on-failure",
    trace: { mode: "retain-on-failure", screenshots: false, snapshots: true },
    launchOptions: {
      executablePath,
      channel:
        executablePath || process.env.CI || process.platform === "linux"
          ? undefined
          : "chrome",
      args: ["--enable-webgl", "--ignore-gpu-blocklist"],
    },
  },
  projects: [
    { name: "desktop", use: { viewport: { width: 1280, height: 720 } } },
    {
      name: "touch",
      use: {
        viewport: { width: 844, height: 390 },
        hasTouch: true,
        isMobile: true,
        deviceScaleFactor: 1,
      },
    },
  ],
  webServer: {
    command: "npm run dev",
    url: "http://127.0.0.1:5173",
    reuseExistingServer: !process.env.CI,
  },
  reporter: [["list"], ["html", { open: "never" }]],
});

export interface Settings {
  sensitivity: number;
  volume: number;
  reducedMotion: boolean;
}
export function loadSettings(): Settings {
  const defaults = {
    sensitivity: 1,
    volume: 0.35,
    reducedMotion: matchMedia("(prefers-reduced-motion: reduce)").matches,
  };
  try {
    const saved = JSON.parse(
      localStorage.getItem("stick-war-settings-v1") ?? "null",
    );
    return saved &&
      typeof saved.sensitivity === "number" &&
      typeof saved.volume === "number"
      ? {
          sensitivity: Math.max(0.2, Math.min(3, saved.sensitivity)),
          volume: Math.max(0, Math.min(1, saved.volume)),
          reducedMotion:
            typeof saved.reducedMotion === "boolean"
              ? saved.reducedMotion
              : defaults.reducedMotion,
        }
      : defaults;
  } catch {
    return defaults;
  }
}
export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem("stick-war-settings-v1", JSON.stringify(settings));
  } catch {
    /* Private browsing can deny storage. Settings still apply to this session. */
  }
}

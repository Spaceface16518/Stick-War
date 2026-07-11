import init from "./pkg/stick_war.js";

const status = document.querySelector("#status");

try {
  await init();
  status.remove();
  document.querySelector("#bevy").focus();
} catch (error) {
  console.error("Failed to start Stick War", error);
  status.textContent = "The game failed to load. Check the browser console for details.";
}

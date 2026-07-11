const status = document.querySelector("#status");
const statusMessage = document.querySelector("#status-message");
const loadingProgress = document.querySelector("#loading-progress");
const canvas = document.querySelector("#bevy");

const updateStatus = (message) => {
  statusMessage.textContent = message;
};

const downloadWasm = async (url) => {
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error(`Wasm download failed with status ${response.status}`);
  }

  const totalBytes = Number(response.headers.get("Content-Length"));
  const reader = response.body?.getReader();

  if (!reader) {
    return response.arrayBuffer();
  }

  if (totalBytes > 0) {
    loadingProgress.max = totalBytes;
    loadingProgress.value = 0;
  }

  const chunks = [];
  let receivedBytes = 0;
  let lastPercent = -1;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    chunks.push(value);
    receivedBytes += value.byteLength;

    if (totalBytes > 0) {
      loadingProgress.value = receivedBytes;
      const percent = Math.min(100, Math.round((receivedBytes / totalBytes) * 100));
      if (percent !== lastPercent) {
        updateStatus(`Loading game files… ${percent}%`);
        lastPercent = percent;
      }
    } else {
      updateStatus(`Loading game files… ${(receivedBytes / 1_048_576).toFixed(1)} MB`);
    }
  }

  const wasmBytes = new Uint8Array(receivedBytes);
  let offset = 0;
  for (const chunk of chunks) {
    wasmBytes.set(chunk, offset);
    offset += chunk.byteLength;
  }

  return wasmBytes;
};

try {
  updateStatus("Preparing game download…");
  const { default: init } = await import("./pkg/stick_war.js");

  updateStatus("Loading game files…");
  const wasmBytes = await downloadWasm(new URL("./pkg/stick_war_bg.wasm", import.meta.url));

  updateStatus("Starting Stick War…");
  loadingProgress.removeAttribute("value");
  await init(wasmBytes);
  status.hidden = true;
  canvas.focus();
} catch (error) {
  console.error("Failed to start Stick War", error);
  status.querySelector(".loading-spinner").hidden = true;
  loadingProgress.hidden = true;
  updateStatus("The game failed to load. Check the browser console for details.");
}

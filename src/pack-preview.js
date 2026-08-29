const invoke = window.__TAURI__.core.invoke;
const convertFileSrc = window.__TAURI__.core.convertFileSrc;
const canvas = document.querySelector("#canvas");
const ctx = canvas.getContext("2d", { alpha: true });
const packSelect = document.querySelector("#pack");
const behaviorSelect = document.querySelector("#behavior");
const directionSelect = document.querySelector("#direction");
const frameInput = document.querySelector("#frame");
const frameNumber = document.querySelector("#frame-number");
const details = document.querySelector("#details");
const message = document.querySelector("#message");
const sources = new Map();
let pack;
let image;

function setMessage(text, state = "") {
  message.textContent = text;
  message.dataset.state = state;
}

function direction() {
  const value = directionSelect.value;
  return {
    travel: value.startsWith("up")
      ? value.includes("-") ? "up-diagonal" : "up"
      : value.startsWith("down")
        ? value.includes("-") ? "down-diagonal" : "down"
        : "horizontal",
    facing: value.endsWith("left") ? "left" : "right",
  };
}

function selection() {
  return window.VirtualPetPack.resolveAnimation(pack, behaviorSelect.value, direction().travel);
}

function draw() {
  if (!pack || !image?.complete || !image.naturalWidth) return;
  const { atlas, animation } = selection();
  const frameIndex = Math.min(Number(frameInput.value), animation.frames.length - 1);
  const frame = animation.frames[frameIndex];
  const scale = Math.min(280 / atlas.frameWidth, 280 / atlas.frameHeight);
  const width = atlas.frameWidth * scale;
  const height = atlas.frameHeight * scale;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.save();
  if (direction().facing === "left") {
    ctx.translate(canvas.width, 0);
    ctx.scale(-1, 1);
  }
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(
    image,
    frame * atlas.frameWidth,
    animation.row * atlas.frameHeight,
    atlas.frameWidth,
    atlas.frameHeight,
    (canvas.width - width) / 2,
    canvas.height - height - 15,
    width,
    height,
  );
  ctx.restore();
  frameNumber.value = `${frameIndex + 1} / ${animation.frames.length}`;
  details.textContent = `${pack.name} · ${behaviorSelect.value} · row ${animation.row}, column ${frame}`;
}

function syncAnimation(resetFrame = true) {
  const { atlas, animation } = selection();
  if (resetFrame) frameInput.value = "0";
  frameInput.max = String(animation.frames.length - 1);
  image = new Image();
  image.addEventListener("load", draw, { once: true });
  image.addEventListener("error", () => setMessage("Could not load this atlas.", "error"), { once: true });
  image.src = atlas.src;
  draw();
}

async function loadPack() {
  setMessage("Loading validated manifest…");
  try {
    pack = await window.VirtualPetPack.load(sources.get(packSelect.value));
    behaviorSelect.replaceChildren(
      ...Object.keys(pack.animations).map((behavior) => {
        const option = document.createElement("option");
        option.value = behavior;
        option.textContent = behavior;
        return option;
      }),
    );
    setMessage("");
    syncAnimation();
  } catch (error) {
    setMessage(`Could not preview pack: ${String(error)}`, "error");
  }
}

async function loadSources() {
  try {
    const response = await fetch("pet-packs/catalog.json");
    if (!response.ok) throw new Error(`Pet catalog returned ${response.status}`);
    const bundled = window.VirtualPetPack.normalizeCatalog(await response.json());
    for (const entry of bundled) sources.set(`bundled:${entry.id}`, entry.manifest);
    for (const entry of await invoke("list_installed_pets")) {
      sources.set(`installed:${entry.id}`, convertFileSrc(entry.manifestPath));
    }
    packSelect.replaceChildren(
      ...[...sources].map(([id, url]) => {
        const option = document.createElement("option");
        option.value = id;
        option.textContent = `${id.startsWith("installed:") ? "Installed" : "Bundled"}: ${id.split(":")[1]}`;
        option.dataset.url = url;
        return option;
      }),
    );
    if (!sources.size) throw new Error("No pet packs are available");
    await loadPack();
  } catch (error) {
    setMessage(`Could not list packs: ${String(error)}`, "error");
  }
}

packSelect.addEventListener("change", loadPack);
behaviorSelect.addEventListener("change", () => syncAnimation());
directionSelect.addEventListener("change", () => syncAnimation());
frameInput.addEventListener("input", draw);
document.querySelector("#previous").addEventListener("click", () => {
  frameInput.value = String(Math.max(0, Number(frameInput.value) - 1));
  draw();
});
document.querySelector("#next").addEventListener("click", () => {
  frameInput.value = String(Math.min(Number(frameInput.max), Number(frameInput.value) + 1));
  draw();
});

loadSources();

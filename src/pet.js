const canvas = document.querySelector("#pet");
const ctx = canvas.getContext("2d", { alpha: true });
const invoke = window.__TAURI__.core.invoke;
const dogAtlas = new Image();
const PET_SIZE = 224;
const PET_SCALES = { small: 0.72, medium: 0.86, large: 1 };
const packCatalog = fetch("pet-packs/catalog.json")
  .then((response) => {
    if (!response.ok) throw new Error(`Pet catalog returned ${response.status}`);
    return response.json();
  })
  .then((entries) => window.VirtualPetPack.normalizeCatalog(entries))
  .then((entries) => new Map(entries.map((entry) => [entry.id, entry.manifest])));
let lastSnapshot;
let activeKind;
let activePack;

ctx.imageSmoothingEnabled = false;
const barkSound = new Audio();
barkSound.preload = "auto";
const snoreSound = new Audio();
snoreSound.preload = "auto";

async function activatePack(kind) {
  if (kind === activeKind) return;
  activeKind = kind;
  activePack = undefined;

  try {
    const manifestUrl = (await packCatalog).get(kind);
    if (!manifestUrl) return;
    const pack = await window.VirtualPetPack.load(manifestUrl);
    if (activeKind !== kind) return;
    activePack = pack;
    dogAtlas.src = pack.atlas.src;
    const bark = pack.sounds.bark;
    const sleep = pack.sounds.sleep;
    if (bark) {
      barkSound.src = bark.src;
      barkSound.loop = bark.loop;
    }
    if (sleep) {
      snoreSound.src = sleep.src;
      snoreSound.loop = sleep.loop;
    }
  } catch (error) {
    console.error(`Could not load the ${kind} pet pack`, error);
  }
}

function rect(x, y, width, height, color) {
  ctx.fillStyle = color;
  ctx.fillRect(x, y, width, height);
}

function pixelText(text, x, y, color = "#241611") {
  ctx.save();
  ctx.font = "bold 4px monospace";
  ctx.textAlign = "center";
  ctx.fillStyle = "#fffdf6";
  const width = Math.max(22, ctx.measureText(text).width + 8);
  rect(x - width / 2 - 1, y - 5, width + 2, 8, "#241611");
  rect(x - width / 2, y - 6, width, 8, "#fffdf6");
  ctx.fillStyle = color;
  ctx.fillText(text, x, y);
  ctx.restore();
}

function drawHouse(kind) {
  if (kind === "cat") {
    rect(26, 35, 22, 8, "#241611");
    rect(25, 37, 24, 8, "#e05a87");
    rect(28, 35, 18, 2, "#ffd49a");
    rect(31, 39, 12, 3, "#9d315d");
    return;
  }

  rect(24, 27, 25, 18, "#241611");
  rect(26, 29, 21, 16, "#c76a2d");
  rect(22, 27, 29, 4, "#241611");
  rect(25, 24, 23, 4, "#e98b3c");
  rect(34, 34, 9, 11, "#241611");
}

function drawCat(snapshot, frame) {
  const bob = ["walk", "run", "play"].includes(snapshot.behavior) ? frame : 0;
  const y = 21 - bob;

  rect(10, y + 13, 5, 2, "#241611");
  rect(7, y + 10, 4, 4, "#241611");
  rect(8, y + 9, 3, 3, "#e87524");
  rect(13, y + 9, 20, 12, "#241611");
  rect(15, y + 10, 18, 10, "#e87524");
  rect(29, y + 4, 14, 15, "#241611");
  rect(31, y + 6, 11, 12, "#e87524");
  rect(30, y + 2, 4, 6, "#241611");
  rect(38, y + 2, 4, 6, "#241611");
  rect(31, y + 3, 2, 4, "#e87524");
  rect(39, y + 3, 2, 4, "#e87524");
  rect(33, y + 9, 2, 2, "#241611");
  rect(39, y + 9, 2, 2, "#241611");
  rect(36, y + 12, 2, 1, "#7d321d");

  const legShift = frame ? 2 : 0;
  rect(17 + legShift, y + 19, 4, 6, "#241611");
  rect(18 + legShift, y + 19, 3, 5, "#e87524");
  rect(27 - legShift, y + 19, 4, 6, "#241611");
  rect(28 - legShift, y + 19, 3, 5, "#e87524");

  if (snapshot.behavior === "groom") {
    rect(31, y + 15, 8, 3, "#e87524");
  }
}

function drawPuppy(snapshot, frame) {
  const bob = ["walk", "run", "play"].includes(snapshot.behavior) ? frame : 0;
  const y = 21 - bob;
  rect(8, y + 8, 5, 3, "#241611");
  rect(7, y + 5, 3, 5, "#ffffff");
  rect(11, y + 9, 22, 13, "#241611");
  rect(13, y + 10, 19, 11, "#fff8e7");
  rect(28, y + 4, 15, 16, "#241611");
  rect(30, y + 6, 12, 13, "#fff8e7");
  rect(27, y + 3, 5, 9, "#241611");
  rect(28, y + 4, 3, 7, "#9c653f");
  rect(40, y + 3, 5, 9, "#241611");
  rect(41, y + 4, 3, 7, "#9c653f");
  rect(33, y + 9, 2, 2, "#241611");
  rect(39, y + 9, 2, 2, "#241611");
  rect(42, y + 13, 3, 2, "#241611");

  const legShift = frame ? 2 : 0;
  rect(15 + legShift, y + 20, 5, 6, "#241611");
  rect(16 + legShift, y + 20, 3, 5, "#fff8e7");
  rect(27 - legShift, y + 20, 5, 6, "#241611");
  rect(28 - legShift, y + 20, 3, 5, "#fff8e7");
}

function drawSleep(snapshot, frame) {
  drawHouse(snapshot.kind);
  ctx.fillStyle = "#2f66f0";
  ctx.font = "bold 5px monospace";
  ctx.fillText("z", 16, 26 - frame * 2);
  ctx.fillText("Z", 20, 20 - frame * 2);
}

function drawPlayToy(kind, frame) {
  if (kind === "cat") {
    rect(43 - frame * 4, 37 + frame * 2, 3, 3, "#ff4d8d");
    return;
  }

  rect(40 - frame * 5, 37 + frame * 2, 6, 6, "#241611");
  rect(41 - frame * 5, 38 + frame * 2, 4, 4, "#ef3f35");
}

function packFrame(snapshot, animation) {
  if (snapshot.reducedMotion && !["walk", "run"].includes(snapshot.behavior)) {
    return snapshot.behavior === "sleep"
      ? animation.frames[animation.frames.length - 1]
      : animation.frames[0];
  }
  const duration = animation.frameDurationMs * (snapshot.reducedMotion ? 2 : 1);
  const index = Math.floor(snapshot.elapsedMs / duration);
  const boundedIndex = animation.loop
    ? index % animation.frames.length
    : Math.min(index, animation.frames.length - 1);
  return animation.frames[boundedIndex];
}

function drawDogAtlas(snapshot) {
  const animation =
    activePack.animations[snapshot.behavior] ?? activePack.animations.idle;
  const frame = packFrame(snapshot, animation);
  const row = animation.row;
  const sourceWidth = activePack.atlas.frameWidth;
  const sourceHeight = activePack.atlas.frameHeight;
  const inset = (canvas.width - PET_SIZE) / 2;

  if (snapshot.behavior === "sleep" && snapshot.kind === "puppy") {
    drawDogHouse(snapshot.name);
    ctx.save();
    dogHouseDoorPath();
    ctx.clip();
    ctx.drawImage(
      dogAtlas,
      frame * sourceWidth,
      row * sourceHeight,
      sourceWidth,
      sourceHeight,
      61,
      111,
      136,
      136,
    );
    ctx.restore();

    ctx.fillStyle = "#d7e9ff";
    ctx.font = "bold 18px monospace";
    ctx.fillText("z", 191, 70 - (frame % 2) * 5);
    ctx.font = "bold 25px monospace";
    ctx.fillText("Z", 208, 48 - (frame % 2) * 5);
    return;
  }

  ctx.save();
  if (snapshot.facing === "left") {
    ctx.translate(canvas.width, 0);
    ctx.scale(-1, 1);
  }
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(
    dogAtlas,
    frame * sourceWidth,
    row * sourceHeight,
    sourceWidth,
    sourceHeight,
    inset,
    canvas.height - PET_SIZE,
    PET_SIZE,
    PET_SIZE,
  );
  ctx.restore();
}

function dogHouseDoorPath() {
  ctx.beginPath();
  ctx.arc(129, 154, 49, Math.PI, 0);
  ctx.lineTo(178, 232);
  ctx.lineTo(80, 232);
  ctx.closePath();
}

function drawDogHouse(name) {
  ctx.fillStyle = "rgba(36, 22, 17, 0.2)";
  ctx.fillRect(27, 232, 204, 8);

  ctx.fillStyle = "#4a2518";
  ctx.fillRect(34, 91, 190, 143);
  ctx.fillStyle = "#bd602d";
  ctx.fillRect(42, 99, 174, 135);

  ctx.fillStyle = "#4a2518";
  dogHouseDoorPath();
  ctx.fill();

  ctx.beginPath();
  ctx.moveTo(18, 99);
  ctx.lineTo(128, 17);
  ctx.lineTo(239, 99);
  ctx.closePath();
  ctx.fillStyle = "#4a2518";
  ctx.fill();

  ctx.beginPath();
  ctx.moveTo(32, 94);
  ctx.lineTo(128, 28);
  ctx.lineTo(225, 94);
  ctx.closePath();
  ctx.fillStyle = "#e4873d";
  ctx.fill();

  ctx.fillStyle = "#f7c96f";
  ctx.fillRect(91, 86, 74, 19);
  ctx.fillStyle = "#4a2518";
  ctx.font = "bold 12px monospace";
  ctx.textAlign = "center";
  ctx.fillText(name.slice(0, 8).toUpperCase(), 128, 100);
}

function drawCareNeed(care) {
  if (!care) return;
  const text =
    care.hunger >= 75
      ? "FOOD!"
      : care.energy <= 25
        ? "REST!"
        : care.cleanliness <= 25
          ? "BATH!"
          : care.happiness <= 30
            ? "PLAY!"
            : "";
  if (!text) return;

  ctx.save();
  ctx.font = "bold 13px monospace";
  ctx.textAlign = "center";
  ctx.fillStyle = "#241611";
  ctx.fillRect(169, 9, 76, 29);
  ctx.fillStyle = "#fffdf6";
  ctx.fillRect(173, 5, 68, 29);
  ctx.fillStyle = "#dc4c00";
  ctx.fillText(text, 207, 25);
  ctx.restore();
}

function syncSounds(snapshot) {
  const masterVolume = (snapshot.soundVolume ?? 65) / 100;
  barkSound.volume = (activePack?.sounds.bark?.volume ?? 0.65) * masterVolume;
  snoreSound.volume = (activePack?.sounds.sleep?.volume ?? 0.28) * masterVolume;
  const shouldSnore = Boolean(activePack?.sounds.sleep) && snapshot.behavior === "sleep";
  const wasSnoring =
    Boolean(lastSnapshot) && !snoreSound.paused && lastSnapshot.behavior === "sleep";

  if (shouldSnore && !wasSnoring) {
    snoreSound.currentTime = 0;
    snoreSound.play().catch((error) => console.warn("Snore audio unavailable", error));
  } else if (!shouldSnore && wasSnoring) {
    snoreSound.pause();
    snoreSound.currentTime = 0;
  }
}

function render(snapshot) {
  if (snapshot.kind !== activeKind) activatePack(snapshot.kind);
  if (
    activePack?.sounds.bark &&
    snapshot.mode === "bark" &&
    snapshot.behavior === "bark" &&
    lastSnapshot?.mode !== "bark"
  ) {
    barkSound.currentTime = 0;
    barkSound.play().catch((error) => console.warn("Bark audio unavailable", error));
  }
  syncSounds(snapshot);
  lastSnapshot = snapshot;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.save();
  const scale = PET_SCALES[snapshot.petSize] ?? PET_SCALES.medium;
  ctx.translate(canvas.width / 2, canvas.height / 2);
  ctx.scale(scale, scale);
  ctx.translate(-canvas.width / 2, -canvas.height / 2);

  if (activePack && dogAtlas.complete && dogAtlas.naturalWidth) {
    drawDogAtlas(snapshot);
  } else {
    ctx.imageSmoothingEnabled = false;
    ctx.save();
    ctx.scale(canvas.width / 50, canvas.height / 50);
    rect(8, 44, 36, 2, "rgba(36, 22, 17, 0.22)");

    if (snapshot.behavior === "sleep") {
      drawSleep(snapshot, snapshot.frame);
    } else {
      if (snapshot.facing === "left") {
        ctx.translate(50, 0);
        ctx.scale(-1, 1);
      }
      if (snapshot.kind === "cat") drawCat(snapshot, snapshot.frame);
      else drawPuppy(snapshot, snapshot.frame);
      if (snapshot.behavior === "play") drawPlayToy(snapshot.kind, snapshot.frame);
    }

    ctx.restore();
    ctx.save();
    ctx.scale(canvas.width / 50, canvas.height / 50);
    if (snapshot.behavior !== "sleep") pixelText(snapshot.name, 25, 8);
    ctx.restore();
  }
  drawCareNeed(snapshot.care);
  ctx.restore();
}

dogAtlas.addEventListener("load", () => {
  if (lastSnapshot) render(lastSnapshot);
});

async function tick() {
  let delay = 250;
  try {
    const snapshot = await invoke("tick");
    render(snapshot);
    delay = snapshot.nextTickMs;
  } catch (error) {
    console.error("Virtual Pet tick failed", error);
    delay = 1000;
  }
  window.setTimeout(tick, delay);
}

render({
  name: "Mochi",
  kind: "puppy",
  mode: "auto",
  behavior: "idle",
  facing: "right",
  frame: 0,
  elapsedMs: 0,
  petSize: "medium",
  soundVolume: 65,
  reducedMotion: false,
  care: null,
});
tick();

const canvas = document.querySelector("#pet");
const ctx = canvas.getContext("2d", { alpha: true });
const invoke = window.__TAURI__.core.invoke;
const dogAtlas = new Image();
const ATLAS_COLUMNS = 4;
const ATLAS_ROWS = 8;
const PET_SIZE = 224;
const DOG_ROWS = {
  idle: 0,
  walk: 1,
  run: 2,
  bark: 3,
  play: 4,
  sleep: 5,
  attention: 6,
  sniff: 7,
  groom: 0,
};
let lastSnapshot;

ctx.imageSmoothingEnabled = false;
dogAtlas.src = "assets/dog-atlas-v2.png";
const barkSound = new Audio("assets/bark.wav");
barkSound.preload = "auto";
barkSound.volume = 0.65;

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

function drawDogAtlas(snapshot) {
  const frame = snapshot.frame % ATLAS_COLUMNS;
  const row = DOG_ROWS[snapshot.behavior] ?? DOG_ROWS.idle;
  const sourceWidth = dogAtlas.naturalWidth / ATLAS_COLUMNS;
  const sourceHeight = dogAtlas.naturalHeight / ATLAS_ROWS;
  const inset = (canvas.width - PET_SIZE) / 2;

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

function render(snapshot) {
  if (
    snapshot.kind === "puppy" &&
    snapshot.mode === "bark" &&
    snapshot.behavior === "bark" &&
    lastSnapshot?.mode !== "bark"
  ) {
    barkSound.currentTime = 0;
    barkSound.play().catch((error) => console.warn("Bark audio unavailable", error));
  }
  lastSnapshot = snapshot;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  if (snapshot.kind === "puppy" && dogAtlas.complete && dogAtlas.naturalWidth) {
    drawDogAtlas(snapshot);
    return;
  }

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
});
tick();

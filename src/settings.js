const invoke = window.__TAURI__.core.invoke;
const form = document.querySelector("#settings-form");
const nameInput = document.querySelector("#name");
const kindInput = document.querySelector("#kind");
const petSizeInput = document.querySelector("#pet-size");
const movementSpeedInput = document.querySelector("#movement-speed");
const movementSpeedValue = document.querySelector("#movement-speed-value");
const soundVolumeInput = document.querySelector("#sound-volume");
const soundVolumeValue = document.querySelector("#sound-volume-value");
const reducedMotionInput = document.querySelector("#reduced-motion");
const careEnabledInput = document.querySelector("#care-enabled");
const careDifficultyInput = document.querySelector("#care-difficulty");
const message = document.querySelector("#message");
const submitButton = form.querySelector('button[type="submit"]');

function setMessage(text, state = "") {
  message.textContent = text;
  message.dataset.state = state;
}

function updateRangeValues() {
  movementSpeedValue.value = `${Number(movementSpeedInput.value).toFixed(1)}×`;
  soundVolumeValue.value = `${soundVolumeInput.value}%`;
}

function updateCareState() {
  careDifficultyInput.disabled = !careEnabledInput.checked;
}

movementSpeedInput.addEventListener("input", updateRangeValues);
soundVolumeInput.addEventListener("input", updateRangeValues);
careEnabledInput.addEventListener("change", updateCareState);

async function load() {
  try {
    const catalogResponse = await fetch("pet-packs/catalog.json");
    if (catalogResponse.ok) {
      const catalog = window.VirtualPetPack.normalizeCatalog(await catalogResponse.json());
      kindInput.replaceChildren(
        ...catalog.map((pet) => {
          const option = document.createElement("option");
          option.value = pet.id;
          option.textContent = pet.name;
          return option;
        }),
      );
    }
    const settings = await invoke("get_settings");
    nameInput.value = settings.name;
    kindInput.value = settings.kind;
    petSizeInput.value = settings.petSize;
    movementSpeedInput.value = settings.movementSpeed;
    soundVolumeInput.value = settings.soundVolume;
    reducedMotionInput.checked = settings.reducedMotion;
    careEnabledInput.checked = settings.careEnabled;
    careDifficultyInput.value = settings.careDifficulty;
    updateRangeValues();
    updateCareState();
    nameInput.focus();
    nameInput.select();
  } catch (error) {
    setMessage(`Could not load settings: ${String(error)}`, "error");
  }
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  setMessage("Saving…");
  submitButton.disabled = true;
  try {
    await invoke("save_settings", {
      settings: {
        name: nameInput.value.trim(),
        kind: kindInput.value,
        petSize: petSizeInput.value,
        movementSpeed: Number(movementSpeedInput.value),
        soundVolume: Number(soundVolumeInput.value),
        reducedMotion: reducedMotionInput.checked,
        careEnabled: careEnabledInput.checked,
        careDifficulty: careDifficultyInput.value,
      },
    });
    setMessage("Settings saved.", "success");
  } catch (error) {
    setMessage(`Could not save settings: ${String(error)}`, "error");
  } finally {
    submitButton.disabled = false;
  }
});

load();

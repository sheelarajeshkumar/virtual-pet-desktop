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
const dayNightEnabledInput = document.querySelector("#day-night-enabled");
const careEnabledInput = document.querySelector("#care-enabled");
const careNotificationsInput = document.querySelector("#care-notifications");
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
  careNotificationsInput.disabled = !careEnabledInput.checked;
}

movementSpeedInput.addEventListener("input", updateRangeValues);
soundVolumeInput.addEventListener("input", updateRangeValues);
careEnabledInput.addEventListener("change", updateCareState);

async function load() {
  try {
    const pets = [];
    const catalogResponse = await fetch("pet-packs/catalog.json");
    if (catalogResponse.ok) {
      const catalog = window.VirtualPetPack.normalizeCatalog(await catalogResponse.json());
      pets.push(...catalog);
    }
    const installed = await invoke("list_installed_pets");
    pets.push(...installed.map((pet) => ({ id: pet.id, name: `${pet.name} (Community)` })));
    kindInput.replaceChildren(
      ...pets.map((pet) => {
        const option = document.createElement("option");
        option.value = pet.id;
        option.textContent = pet.name;
        return option;
      }),
    );
    const settings = await invoke("get_settings");
    nameInput.value = settings.name;
    kindInput.value = settings.kind;
    petSizeInput.value = settings.petSize;
    movementSpeedInput.value = settings.movementSpeed;
    soundVolumeInput.value = settings.soundVolume;
    reducedMotionInput.checked = settings.reducedMotion;
    dayNightEnabledInput.checked = settings.dayNightEnabled;
    careEnabledInput.checked = settings.careEnabled;
    careNotificationsInput.checked = settings.careNotifications;
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
    if (careNotificationsInput.checked) {
      const notifications = window.__TAURI__.notification;
      let permitted = await notifications.isPermissionGranted();
      if (!permitted) permitted = (await notifications.requestPermission()) === "granted";
      if (!permitted) throw new Error("Notification permission was not granted.");
    }
    await invoke("save_settings", {
      settings: {
        name: nameInput.value.trim(),
        kind: kindInput.value,
        petSize: petSizeInput.value,
        movementSpeed: Number(movementSpeedInput.value),
        soundVolume: Number(soundVolumeInput.value),
        reducedMotion: reducedMotionInput.checked,
        dayNightEnabled: dayNightEnabledInput.checked,
        careEnabled: careEnabledInput.checked,
        careNotifications: careNotificationsInput.checked,
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

const invoke = window.__TAURI__.core.invoke;
const form = document.querySelector("#settings-form");
const nameInput = document.querySelector("#name");
const kindInput = document.querySelector("#kind");
const message = document.querySelector("#message");

async function load() {
  try {
    const settings = await invoke("get_settings");
    nameInput.value = settings.name;
    kindInput.value = settings.kind;
    nameInput.focus();
    nameInput.select();
  } catch (error) {
    message.textContent = String(error);
  }
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  message.textContent = "";
  try {
    await invoke("save_settings", {
      name: nameInput.value,
      kind: kindInput.value,
    });
  } catch (error) {
    message.textContent = String(error);
  }
});

load();


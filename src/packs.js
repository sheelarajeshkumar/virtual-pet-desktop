const invoke = window.__TAURI__.core.invoke;
const dialog = window.__TAURI__.dialog;
const importButton = document.querySelector("#import");
const packsElement = document.querySelector("#packs");
const empty = document.querySelector("#empty");
const message = document.querySelector("#message");

function setMessage(text, state = "") {
  message.textContent = text;
  message.dataset.state = state;
}

async function removePack(pack) {
  const confirmed = await dialog.confirm(
    `Remove ${pack.name}? The bundled Puppy and Cat packs are not affected.`,
    { title: "Remove pet pack", kind: "warning" },
  );
  if (!confirmed) return;
  setMessage(`Removing ${pack.name}…`);
  try {
    await invoke("remove_pet_pack", { id: pack.id });
    setMessage(`${pack.name} removed.`, "success");
    await load();
  } catch (error) {
    setMessage(`Could not remove pack: ${String(error)}`, "error");
  }
}

function render(packs) {
  empty.hidden = packs.length > 0;
  packsElement.replaceChildren(
    ...packs.map((pack) => {
      const item = document.createElement("article");
      item.className = "pack";
      const details = document.createElement("div");
      const name = document.createElement("strong");
      const meta = document.createElement("span");
      const remove = document.createElement("button");
      name.textContent = pack.name;
      meta.textContent = `${pack.species} · v${pack.version} · ${pack.id}`;
      remove.type = "button";
      remove.className = "remove";
      remove.textContent = "REMOVE";
      remove.addEventListener("click", () => removePack(pack));
      details.append(name, meta);
      item.append(details, remove);
      return item;
    }),
  );
}

async function load() {
  try {
    render(await invoke("list_installed_pets"));
  } catch (error) {
    setMessage(`Could not list packs: ${String(error)}`, "error");
  }
}

importButton.addEventListener("click", async () => {
  const sourceDirectory = await dialog.open({ directory: true, multiple: false });
  if (!sourceDirectory) return;
  importButton.disabled = true;
  setMessage("Validating and installing pack…");
  try {
    const pack = await invoke("install_pet_pack", { sourceDirectory });
    setMessage(`${pack.name} installed. Select it in Settings.`, "success");
    await load();
  } catch (error) {
    setMessage(`Could not install pack: ${String(error)}`, "error");
  } finally {
    importButton.disabled = false;
  }
});

load();

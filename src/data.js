const invoke = window.__TAURI__.core.invoke;
const dialog = window.__TAURI__.dialog;
const exportBackup = document.querySelector("#export-backup");
const importBackup = document.querySelector("#import-backup");
const exportPack = document.querySelector("#export-pack");
const packSelect = document.querySelector("#pack");
const message = document.querySelector("#message");

function setMessage(text, state = "") {
  message.textContent = text;
  message.dataset.state = state;
}

exportBackup.addEventListener("click", async () => {
  const path = await dialog.save({
    defaultPath: "virtual-pet-backup.json",
    filters: [{ name: "Virtual Pet backup", extensions: ["json"] }],
  });
  if (!path) return;
  try {
    await invoke("export_backup", { path });
    setMessage("Backup exported.", "success");
  } catch (error) {
    setMessage(`Export failed: ${String(error)}`, "error");
  }
});

importBackup.addEventListener("click", async () => {
  const path = await dialog.open({
    multiple: false,
    directory: false,
    filters: [{ name: "Virtual Pet backup", extensions: ["json"] }],
  });
  if (!path) return;
  const confirmed = await dialog.confirm(
    "Replace current settings, care state, and inventory with this backup?",
    { title: "Import backup", kind: "warning" },
  );
  if (!confirmed) return;
  try {
    await invoke("import_backup", { path });
    setMessage("Backup restored. The pet updated immediately.", "success");
  } catch (error) {
    setMessage(`Import failed: ${String(error)}`, "error");
  }
});

exportPack.addEventListener("click", async () => {
  const destinationDirectory = await dialog.open({ directory: true, multiple: false });
  if (!destinationDirectory) return;
  try {
    const exportedPath = await invoke("export_pet_pack", {
      id: packSelect.value,
      destinationDirectory,
    });
    setMessage(`Pack exported to ${exportedPath}`, "success");
  } catch (error) {
    setMessage(`Pack export failed: ${String(error)}`, "error");
  }
});

async function load() {
  try {
    const packs = await invoke("list_installed_pets");
    packSelect.replaceChildren(
      ...packs.map((pack) => {
        const option = document.createElement("option");
        option.value = pack.id;
        option.textContent = `${pack.name} (${pack.version})`;
        return option;
      }),
    );
    exportPack.disabled = packs.length === 0;
    if (!packs.length) setMessage("Install a community pet pack before exporting one.");
  } catch (error) {
    exportPack.disabled = true;
    setMessage(`Could not load installed packs: ${String(error)}`, "error");
  }
}

load();

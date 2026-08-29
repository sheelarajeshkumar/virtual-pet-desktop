const invoke = window.__TAURI__.core.invoke;
const dialog = window.__TAURI__.dialog;
const importButton = document.querySelector("#import");
const extensionsElement = document.querySelector("#extensions");
const empty = document.querySelector("#empty");
const message = document.querySelector("#message");
let runToken = 0;

function setMessage(text, state = "") {
  message.textContent = text;
  message.dataset.state = state;
}

const wait = (durationMs) => new Promise((resolve) => window.setTimeout(resolve, durationMs));

async function run(extension) {
  const token = ++runToken;
  setMessage(`Running ${extension.name}…`);
  try {
    for (const step of extension.steps) {
      if (token !== runToken) break;
      await invoke("set_extension_action", { action: step.action });
      await wait(step.durationMs);
    }
    setMessage(`${extension.name} complete.`, "success");
  } catch (error) {
    setMessage(`Behavior failed: ${String(error)}`, "error");
  } finally {
    if (token === runToken) await invoke("set_extension_action", { action: "auto" });
  }
}

async function remove(extension) {
  if (!(await dialog.confirm(`Remove ${extension.name}?`, { title: "Remove behavior" }))) return;
  try {
    await invoke("remove_behavior_extension", { id: extension.id });
    setMessage(`${extension.name} removed.`, "success");
    await load();
  } catch (error) {
    setMessage(`Could not remove behavior: ${String(error)}`, "error");
  }
}

function render(extensions) {
  empty.hidden = extensions.length > 0;
  extensionsElement.replaceChildren(
    ...extensions.map((extension) => {
      const article = document.createElement("article");
      article.className = "extension";
      const title = document.createElement("h2");
      const description = document.createElement("p");
      const controls = document.createElement("div");
      const runButton = document.createElement("button");
      const removeButton = document.createElement("button");
      title.textContent = extension.name;
      description.textContent = extension.description;
      controls.className = "controls";
      runButton.textContent = "RUN";
      runButton.addEventListener("click", () => run(extension));
      removeButton.textContent = "REMOVE";
      removeButton.className = "remove";
      removeButton.addEventListener("click", () => remove(extension));
      controls.append(runButton, removeButton);
      article.append(title, description, controls);
      return article;
    }),
  );
}

async function load() {
  try {
    render(await invoke("list_behavior_extensions"));
  } catch (error) {
    setMessage(`Could not list behaviors: ${String(error)}`, "error");
  }
}

importButton.addEventListener("click", async () => {
  const path = await dialog.open({
    multiple: false,
    directory: false,
    filters: [{ name: "Virtual Pet behavior", extensions: ["json"] }],
  });
  if (!path) return;
  try {
    const extension = await invoke("install_behavior_extension", { path });
    setMessage(`${extension.name} installed.`, "success");
    await load();
  } catch (error) {
    setMessage(`Could not install behavior: ${String(error)}`, "error");
  }
});

load();

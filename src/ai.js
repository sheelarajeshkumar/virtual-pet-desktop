const invoke = window.__TAURI__.core.invoke;
const form = document.querySelector("#settings-form");
const enabled = document.querySelector("#enabled");
const endpoint = document.querySelector("#endpoint");
const model = document.querySelector("#model");
const memoryEnabled = document.querySelector("#memory-enabled");
const proactiveEnabled = document.querySelector("#proactive-enabled");
const voiceOutput = document.querySelector("#voice-output");
const cooldown = document.querySelector("#cooldown");
const clearMemory = document.querySelector("#clear-memory");
const chatForm = document.querySelector("#chat-form");
const message = document.querySelector("#message");
const mood = document.querySelector("#mood");
const context = document.querySelector("#context");
const send = document.querySelector("#send");
const messages = document.querySelector("#messages");
const status = document.querySelector("#status");
let settings;
let busy = false;

function setStatus(text, state = "") {
  status.textContent = text;
  status.dataset.state = state;
}

function renderEnabledState() {
  const canChat = Boolean(settings?.enabled) && !busy;
  for (const control of chatForm.elements) control.disabled = !canChat;
  send.textContent = busy ? "THINKING…" : "SEND";
}

function applySettings(next) {
  settings = next;
  enabled.checked = next.enabled;
  endpoint.value = next.endpoint;
  model.value = next.model;
  memoryEnabled.checked = next.memoryEnabled;
  proactiveEnabled.checked = next.proactiveSuggestions;
  voiceOutput.checked = next.voiceOutput;
  cooldown.value = String(next.proactiveCooldownMinutes);
  cooldown.disabled = !next.proactiveSuggestions;
  clearMemory.disabled = !next.memoryEnabled;
  renderEnabledState();
}

function settingsInput() {
  return {
    enabled: enabled.checked,
    endpoint: endpoint.value.trim(),
    model: model.value.trim(),
    memoryEnabled: memoryEnabled.checked,
    proactiveSuggestions: proactiveEnabled.checked,
    proactiveCooldownMinutes: Number(cooldown.value),
    voiceOutput: voiceOutput.checked,
  };
}

function speak(text) {
  if (!settings?.voiceOutput) return;
  if (!("speechSynthesis" in window) || typeof SpeechSynthesisUtterance === "undefined") {
    setStatus("Voice output is unavailable on this system.", "error");
    return;
  }
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(new SpeechSynthesisUtterance(text));
}

async function confirmAction(action, button) {
  button.disabled = true;
  try {
    await invoke("confirm_ai_action", { action });
    button.textContent = "CONFIRMED";
    setStatus(`${action} confirmed.`, "success");
  } catch (error) {
    button.disabled = false;
    setStatus(`Action failed: ${String(error)}`, "error");
  }
}

function appendMessage(role, text, suggestedAction = null) {
  messages.querySelector(".empty")?.remove();
  const bubble = document.createElement("article");
  bubble.className = `message ${role}`;
  const content = document.createElement("span");
  content.textContent = text;
  bubble.append(content);
  if (role === "assistant" && suggestedAction) {
    const confirm = document.createElement("button");
    confirm.type = "button";
    confirm.textContent = `CONFIRM ${suggestedAction.toUpperCase()}`;
    confirm.addEventListener("click", () => confirmAction(suggestedAction, confirm));
    bubble.append(confirm);
  }
  messages.append(bubble);
  messages.scrollTop = messages.scrollHeight;
}

async function sendChat(input, showUser = true) {
  if (busy || !settings?.enabled) return null;
  busy = true;
  renderEnabledState();
  setStatus("Waiting for local Ollama…");
  if (showUser) appendMessage("user", input.message);
  try {
    const response = await invoke("ai_chat", { input });
    appendMessage("assistant", response.reply, response.suggestedAction);
    speak(response.reply);
    setStatus("Reply received from local Ollama.", "success");
    return response;
  } catch (error) {
    setStatus(`AI unavailable: ${String(error)}`, "error");
    return null;
  } finally {
    busy = false;
    renderEnabledState();
  }
}

async function saveSettings(successMessage) {
  setStatus("Saving…");
  try {
    applySettings(await invoke("save_ai_settings", { settings: settingsInput() }));
    setStatus(successMessage, "success");
    return true;
  } catch (error) {
    enabled.checked = Boolean(settings?.enabled);
    renderEnabledState();
    setStatus(`Could not save AI settings: ${String(error)}`, "error");
    return false;
  }
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  await saveSettings("AI settings saved.");
});

enabled.addEventListener("change", () => {
  saveSettings(enabled.checked ? "AI Companion enabled." : "AI Companion disabled.");
});
proactiveEnabled.addEventListener("change", () => {
  cooldown.disabled = !proactiveEnabled.checked;
});
memoryEnabled.addEventListener("change", () => {
  clearMemory.disabled = !memoryEnabled.checked;
});

clearMemory.addEventListener("click", async () => {
  if (!window.confirm("Clear the locally stored AI conversation history?")) return;
  try {
    await invoke("clear_ai_memory");
    messages.replaceChildren();
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.textContent = "Local AI memory cleared.";
    messages.append(empty);
    setStatus("Local AI memory cleared.", "success");
  } catch (error) {
    setStatus(`Could not clear memory: ${String(error)}`, "error");
  }
});

chatForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const text = message.value.trim();
  if (!text) return;
  message.value = "";
  await sendChat({
    message: text,
    mood: mood.value || null,
    context: context.value.trim() || null,
    proactive: false,
  });
  message.focus();
});

async function checkProactiveSuggestion() {
  if (busy || !settings?.enabled || !settings.proactiveSuggestions) return;
  try {
    const response = await invoke("ai_proactive", {
      mood: mood.value || null,
      context: context.value.trim() || null,
    });
    if (!response) return;
    appendMessage("assistant", response.reply, response.suggestedAction);
    speak(response.reply);
    setStatus("Your companion has a suggestion.", "success");
  } catch (error) {
    setStatus(`Proactive AI unavailable: ${String(error)}`, "error");
  }
}

async function load() {
  try {
    applySettings(await invoke("get_ai_settings"));
    setStatus(settings.enabled ? "AI Companion is ready." : "AI Companion is disabled.");
  } catch (error) {
    setStatus(`Could not load AI settings: ${String(error)}`, "error");
  }
}

load();
setInterval(checkProactiveSuggestion, 60_000);

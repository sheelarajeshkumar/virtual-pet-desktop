const { invoke, Channel } = window.__TAURI__.core;
const dialog = window.__TAURI__.dialog;
const byId = (id) => document.querySelector(`#${id}`);
const form = byId("settings-form");
const chatForm = byId("chat-form");
const searchForm = byId("search-form");
const enabled = byId("enabled");
const provider = byId("provider");
const endpoint = byId("endpoint");
const model = byId("model");
const embeddingModel = byId("embedding-model");
const personality = byId("personality");
const memoryEnabled = byId("memory-enabled");
const semanticMemory = byId("semantic-memory");
const proactiveEnabled = byId("proactive-enabled");
const voiceOutput = byId("voice-output");
const speechInput = byId("speech-input");
const cooldown = byId("cooldown");
const voice = byId("voice");
const speechRate = byId("speech-rate");
const speechRateValue = byId("speech-rate-value");
const providerHelp = byId("provider-help");
const availableModels = byId("available-models");
const clearMemory = byId("clear-memory");
const testProvider = byId("test-provider");
const exportChat = byId("export-chat");
const importChat = byId("import-chat");
const message = byId("message");
const microphone = byId("microphone");
const mood = byId("mood");
const context = byId("context");
const send = byId("send");
const messages = byId("messages");
const searchQuery = byId("search-query");
const searchResults = byId("search-results");
const status = byId("status");
const providerDefaults = {
  ollama: { endpoint: "http://127.0.0.1:11434", model: "llama3.2", embedding: "nomic-embed-text" },
  lmStudio: { endpoint: "http://127.0.0.1:1234", model: "local-model", embedding: "local-model" },
  llamaCpp: { endpoint: "http://127.0.0.1:8080", model: "local-model", embedding: "local-model" },
};
let settings;
let busy = false;
let recognition;

function setStatus(text, state = "") {
  status.textContent = text;
  status.dataset.state = state;
}

function renderEnabledState() {
  const canChat = Boolean(settings?.enabled) && !busy;
  for (const control of chatForm.elements) control.disabled = !canChat;
  send.textContent = busy ? "THINKING…" : "SEND";
  microphone.disabled = !canChat || !settings?.speechInput || !speechRecognitionClass();
  semanticMemory.disabled = !memoryEnabled.checked;
  embeddingModel.disabled = !semanticMemory.checked;
  cooldown.disabled = !proactiveEnabled.checked;
  voice.disabled = !voiceOutput.checked;
  speechRate.disabled = !voiceOutput.checked;
  for (const button of [clearMemory, exportChat, importChat]) {
    button.disabled = !memoryEnabled.checked;
  }
}

function applySettings(next) {
  settings = next;
  enabled.checked = next.enabled;
  provider.value = next.provider;
  endpoint.value = next.endpoint;
  model.value = next.model;
  embeddingModel.value = next.embeddingModel;
  personality.value = next.personality;
  memoryEnabled.checked = next.memoryEnabled;
  semanticMemory.checked = next.semanticMemory;
  proactiveEnabled.checked = next.proactiveSuggestions;
  voiceOutput.checked = next.voiceOutput;
  speechInput.checked = next.speechInput;
  cooldown.value = String(next.proactiveCooldownMinutes);
  voice.value = next.voiceName;
  speechRate.value = String(next.speechRate);
  speechRateValue.textContent = `${Number(next.speechRate).toFixed(1)}×`;
  renderEnabledState();
}

function settingsInput() {
  return {
    enabled: enabled.checked,
    provider: provider.value,
    endpoint: endpoint.value.trim(),
    model: model.value.trim(),
    embeddingModel: embeddingModel.value.trim(),
    memoryEnabled: memoryEnabled.checked,
    semanticMemory: semanticMemory.checked,
    proactiveSuggestions: proactiveEnabled.checked,
    proactiveCooldownMinutes: Number(cooldown.value),
    voiceOutput: voiceOutput.checked,
    voiceName: voice.value,
    speechRate: Number(speechRate.value),
    speechInput: speechInput.checked,
    personality: personality.value,
  };
}

function loadVoices() {
  if (!("speechSynthesis" in window)) return;
  const selected = voice.value || settings?.voiceName || "";
  const options = [new Option("System default", "")];
  for (const item of window.speechSynthesis.getVoices()) {
    options.push(new Option(`${item.name} (${item.lang})`, item.name));
  }
  voice.replaceChildren(...options);
  voice.value = selected;
}

function speak(text) {
  if (!settings?.voiceOutput) return;
  if (!("speechSynthesis" in window) || typeof SpeechSynthesisUtterance === "undefined") {
    setStatus("Voice output is unavailable on this system.", "error");
    return;
  }
  const utterance = new SpeechSynthesisUtterance(text);
  utterance.rate = settings.speechRate;
  utterance.voice = window.speechSynthesis
    .getVoices()
    .find((candidate) => candidate.name === settings.voiceName) ?? null;
  window.speechSynthesis.cancel();
  window.speechSynthesis.speak(utterance);
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

async function confirmRoutine(routine, button) {
  button.disabled = true;
  try {
    const installed = await invoke("confirm_ai_routine", { routine });
    button.textContent = "INSTALLED";
    setStatus(`${installed.name} installed in Behavior extensions.`, "success");
  } catch (error) {
    button.disabled = false;
    setStatus(`Routine failed: ${String(error)}`, "error");
  }
}

function appendMessage(role, text = "") {
  messages.querySelector(".empty")?.remove();
  const bubble = document.createElement("article");
  bubble.className = `message ${role}`;
  const content = document.createElement("span");
  content.textContent = text;
  bubble.append(content);
  messages.append(bubble);
  messages.scrollTop = messages.scrollHeight;
  return { bubble, content };
}

function addConfirmations(bubble, response) {
  if (response.suggestedAction) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = `CONFIRM ${response.suggestedAction.toUpperCase()}`;
    button.addEventListener("click", () => confirmAction(response.suggestedAction, button));
    bubble.append(button);
  }
  if (response.routine) {
    const summary = document.createElement("small");
    summary.textContent = `Routine: ${response.routine.name} · ${response.routine.steps.length} safe steps`;
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = "INSTALL ROUTINE";
    button.addEventListener("click", () => confirmRoutine(response.routine, button));
    bubble.append(summary, button);
  }
}

function showResponse(response) {
  const item = appendMessage("assistant", response.reply);
  addConfirmations(item.bubble, response);
  speak(response.reply);
}

async function sendChat(input) {
  if (busy || !settings?.enabled) return;
  busy = true;
  renderEnabledState();
  setStatus(`Waiting for local ${provider.options[provider.selectedIndex].text}…`);
  appendMessage("user", input.message);
  const streaming = appendMessage("assistant", "…");
  let finalResponse;
  try {
    const onEvent = new Channel();
    onEvent.onmessage = (event) => {
      if (event.event === "delta") streaming.content.textContent = event.data.reply || "…";
      if (event.event === "done") finalResponse = event.data;
      messages.scrollTop = messages.scrollHeight;
    };
    await invoke("ai_chat_stream", { input, onEvent });
    if (!finalResponse) throw new Error("Local provider ended without a final response.");
    streaming.content.textContent = finalResponse.reply;
    addConfirmations(streaming.bubble, finalResponse);
    speak(finalResponse.reply);
    setStatus("Streamed reply received from the local provider.", "success");
  } catch (error) {
    streaming.bubble.remove();
    setStatus(`AI unavailable: ${String(error)}`, "error");
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
    if (settings) applySettings(settings);
    setStatus(`Could not save AI settings: ${String(error)}`, "error");
    return false;
  }
}

async function runProviderTest() {
  testProvider.disabled = true;
  providerHelp.textContent = "Checking the local server and model list…";
  try {
    const result = await invoke("test_ai_provider", { settings: settingsInput() });
    availableModels.replaceChildren(...result.models.map((name) => new Option(name, name)));
    providerHelp.textContent = `${result.providerName} reachable. ${result.models.length} model(s). Chat model ${result.modelAvailable ? "found" : "not found"}. Embedding model ${result.embeddingModelAvailable ? "ready" : "not found"}. ${result.setupHint}`;
    providerHelp.dataset.state = result.modelAvailable ? "success" : "warning";
  } catch (error) {
    providerHelp.textContent = `${String(error)} ${providerSetupHint()}`;
    providerHelp.dataset.state = "error";
  } finally {
    testProvider.disabled = false;
  }
}

function providerSetupHint() {
  return {
    ollama: "Run `ollama serve`, pull the chat model, then test again.",
    lmStudio: "Load a model in LM Studio Developer and start the server on port 1234.",
    llamaCpp: "Start llama-server with a GGUF model on port 8080.",
  }[provider.value];
}

function speechRecognitionClass() {
  return window.SpeechRecognition || window.webkitSpeechRecognition;
}

function startRecognition() {
  const Recognition = speechRecognitionClass();
  if (!settings?.speechInput || !Recognition) {
    setStatus("Microphone recognition is disabled or unavailable.", "error");
    return;
  }
  recognition?.abort();
  recognition = new Recognition();
  recognition.continuous = false;
  recognition.interimResults = false;
  if ("processLocally" in recognition) recognition.processLocally = true;
  recognition.onstart = () => {
    microphone.textContent = "LISTENING…";
    setStatus("Listening for one message…");
  };
  recognition.onresult = (event) => {
    message.value = event.results[0][0].transcript;
    message.focus();
  };
  recognition.onerror = (event) => setStatus(`Speech recognition failed: ${event.error}`, "error");
  recognition.onend = () => {
    microphone.textContent = "MIC";
  };
  recognition.start();
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  await saveSettings("AI settings saved.");
});
enabled.addEventListener("change", () => saveSettings(enabled.checked ? "AI Companion enabled." : "AI Companion disabled."));
provider.addEventListener("change", () => {
  const defaults = providerDefaults[provider.value];
  endpoint.value = defaults.endpoint;
  model.value = defaults.model;
  embeddingModel.value = defaults.embedding;
  providerHelp.textContent = providerSetupHint();
});
for (const control of [memoryEnabled, semanticMemory, proactiveEnabled, voiceOutput, speechInput]) {
  control.addEventListener("change", renderEnabledState);
}
speechRate.addEventListener("input", () => {
  speechRateValue.textContent = `${Number(speechRate.value).toFixed(1)}×`;
});
testProvider.addEventListener("click", runProviderTest);
microphone.addEventListener("click", startRecognition);

clearMemory.addEventListener("click", async () => {
  if (!(await dialog.confirm("Clear all encrypted AI chat and semantic memory for every pet?", { title: "Clear AI memory", kind: "warning" }))) return;
  try {
    await invoke("clear_ai_memory");
    messages.replaceChildren(Object.assign(document.createElement("p"), { className: "empty", textContent: "Encrypted AI memory cleared." }));
    setStatus("Encrypted AI memory cleared.", "success");
  } catch (error) {
    setStatus(`Could not clear memory: ${String(error)}`, "error");
  }
});

exportChat.addEventListener("click", async () => {
  const path = await dialog.save({ defaultPath: "virtual-pet-ai-chat.json", filters: [{ name: "Virtual Pet AI chat", extensions: ["json"] }] });
  if (!path) return;
  try {
    await invoke("export_ai_chat", { path });
    setStatus("Chat archive exported. The chosen export file is readable JSON.", "success");
  } catch (error) {
    setStatus(`Chat export failed: ${String(error)}`, "error");
  }
});

importChat.addEventListener("click", async () => {
  const path = await dialog.open({ multiple: false, directory: false, filters: [{ name: "Virtual Pet AI chat", extensions: ["json"] }] });
  if (!path || !(await dialog.confirm("Replace current AI chat history with this archive? Semantic vectors will be rebuilt from new conversations.", { title: "Import AI chat", kind: "warning" }))) return;
  try {
    const count = await invoke("import_ai_chat", { path });
    setStatus(`Imported ${count} chat messages into encrypted storage.`, "success");
  } catch (error) {
    setStatus(`Chat import failed: ${String(error)}`, "error");
  }
});

chatForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const text = message.value.trim();
  if (!text) return;
  message.value = "";
  await sendChat({ message: text, mood: mood.value || null, context: context.value.trim() || null, proactive: false });
  message.focus();
});

searchForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  searchResults.textContent = "Searching local vectors…";
  try {
    const results = await invoke("search_ai_memory", { query: searchQuery.value.trim() });
    searchResults.replaceChildren(...results.map((result) => {
      const item = document.createElement("article");
      item.textContent = `${Math.round(result.score * 100)}% · ${result.text}`;
      return item;
    }));
    if (!results.length) searchResults.textContent = "No related memories found for this pet.";
  } catch (error) {
    searchResults.textContent = `Search failed: ${String(error)}`;
  }
});

async function load() {
  try {
    if (location.href.includes("release-media=1")) {
      const releaseSettings = {
        enabled: false, provider: "ollama", endpoint: providerDefaults.ollama.endpoint,
        model: providerDefaults.ollama.model, embeddingModel: providerDefaults.ollama.embedding,
        memoryEnabled: false, semanticMemory: false, proactiveSuggestions: false,
        proactiveCooldownMinutes: 30, voiceOutput: false, voiceName: "", speechRate: 1,
        speechInput: false, personality: "loyal",
      };
      applySettings(releaseSettings);
      window.setTimeout(() => applySettings(releaseSettings), 100);
      providerHelp.textContent = "AI is disabled by default. Choose a local provider, test it, then enable only the features you want.";
      setStatus("Release preview: default local-first configuration.");
      window.scrollTo(0, 0);
      return;
    }
    const runtime = await invoke("get_ai_runtime");
    applySettings(runtime.settings);
    loadVoices();
    if (runtime.memoryError) setStatus(`Memory warning: ${runtime.memoryError}`, "error");
    else setStatus(settings.enabled ? "AI Companion is ready." : "AI Companion is disabled.");
    for (const response of await invoke("take_ai_suggestions")) showResponse(response);
  } catch (error) {
    setStatus(`Could not load AI settings: ${String(error)}`, "error");
  }
}

window.speechSynthesis?.addEventListener?.("voiceschanged", loadVoices);
window.__TAURI__.event.listen("ai-suggestion", async ({ payload }) => {
  await invoke("take_ai_suggestions").catch(() => {});
  showResponse(payload);
});
load();

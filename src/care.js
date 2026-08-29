const invoke = window.__TAURI__.core.invoke;
const title = document.querySelector("#title");
const disabledNotice = document.querySelector("#disabled");
const message = document.querySelector("#message");
const buttons = [...document.querySelectorAll("[data-action]")];
const restButton = document.querySelector("#rest-action");
const needs = ["hunger", "energy", "happiness", "cleanliness"];

function render(result) {
  const { name, enabled, state } = result;
  title.textContent = `${name}'s Care`;
  disabledNotice.hidden = enabled;
  buttons.forEach((button) => {
    button.disabled = !enabled;
  });
  restButton.dataset.action = state.sleeping ? "wake" : "sleep";
  restButton.textContent = state.sleeping ? "Wake" : "Rest";
  needs.forEach((need) => {
    const value = Math.round(need === "hunger" ? 100 - state.hunger : state[need]);
    document.querySelector(`#${need}`).value = value;
    document.querySelector(`#${need}-value`).value = `${value}%`;
  });
}

async function refresh() {
  try {
    render(await invoke("get_care_state"));
  } catch (error) {
    message.textContent = `Could not load care state: ${String(error)}`;
  }
}

buttons.forEach((button) => {
  button.addEventListener("click", async () => {
    const label = button.textContent;
    buttons.forEach((candidate) => {
      candidate.disabled = true;
    });
    message.textContent = "";
    try {
      render(await invoke("perform_care_action", { action: button.dataset.action }));
      message.textContent = `${label} complete.`;
    } catch (error) {
      message.textContent = `Care action failed: ${String(error)}`;
    } finally {
      await refresh();
    }
  });
});

refresh();

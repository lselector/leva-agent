/* Settings module — model selection. */

/**
 * Initialize the settings module.
 */
function initSettings() {
  var select = $("model-select");
  loadModels(select);

  select.addEventListener(
    "change", function () {
      switchModel(this.value);
    }
  );
}

/**
 * Load available models from server.
 */
async function loadModels(select) {
  try {
    var resp = await fetch("/api/models");
    if (!resp.ok) return;
    var data = await resp.json();

    select.innerHTML = "";
    data.available.forEach(function (m) {
      var opt = document.createElement("option");
      opt.value = m;
      opt.textContent = m;
      if (m === data.current) {
        opt.selected = true;
      }
      select.appendChild(opt);
    });
  } catch (e) {
    /* offline — keep defaults */
  }
}

/**
 * Switch the active model.
 */
async function switchModel(model) {
  try {
    var resp = await fetch(
      "/api/models/current",
      {
        method: "PUT",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ model: model }),
      }
    );
    var data = await resp.json();
    if (data.error) {
      alert("Error: " + data.error);
    }
  } catch (e) {
    alert("Cannot reach server");
  }
}

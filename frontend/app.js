/* Main app — tab navigation, sessions, init. */

/**
 * Initialize the application.
 */
function initApp() {
  initTabNavigation();
  initChat();
  initFiles();
  initJobs();
  initSettings();
  startHealthCheck();
  loadSessionList();
}

/**
 * Set up tab navigation.
 */
function initTabNavigation() {
  var tabs = $$(".nav-tab");
  tabs.forEach(function (tab) {
    tab.addEventListener("click", function () {
      var viewId = this.getAttribute("data-view");
      switchView(viewId);
    });
  });

  /* New chat button */
  $("new-chat-btn").addEventListener(
    "click",
    function () {
      switchView("chat-view");
      newSession();
      loadSessionList();
    }
  );
}

/**
 * Switch to a different view tab.
 */
function switchView(viewId) {
  $$(".nav-tab").forEach(function (tab) {
    if (tab.getAttribute("data-view") === viewId) {
      tab.classList.add("active");
    } else {
      tab.classList.remove("active");
    }
  });
  $$(".view").forEach(function (view) {
    if (view.id === viewId) {
      view.classList.add("active");
    } else {
      view.classList.remove("active");
    }
  });
}

/**
 * Load session list from server.
 */
async function loadSessionList() {
  var container = $("sessions");
  try {
    var resp = await fetch("/api/chat/history");
    if (!resp.ok) return;
    var sessions = await resp.json();
    container.innerHTML = "";
    sessions.reverse().forEach(function (sid) {
      var div = document.createElement("div");
      div.classList.add("session-item");
      if (sid === currentSessionId) {
        div.classList.add("active");
      }
      div.textContent = sid;
      div.addEventListener(
        "click",
        function () {
          loadSession(sid);
        }
      );
      container.appendChild(div);
    });
  } catch (e) {
    /* offline — skip */
  }
}

/**
 * Load a previous session's messages.
 */
async function loadSession(sid) {
  try {
    var resp = await fetch(
      "/api/chat/" + sid
    );
    if (!resp.ok) return;
    var msgs = await resp.json();
    currentSessionId = sid;
    $("messages").innerHTML = "";
    $("chat-title").textContent = sid;
    msgs.forEach(function (m) {
      addMessage(m.role, m.content);
    });
    switchView("chat-view");
    loadSessionList();
  } catch (e) {
    /* skip */
  }
}

/**
 * Periodically check server health.
 */
function startHealthCheck() {
  async function check() {
    var online = await checkHealth();
    setOnline(online);
  }
  check();
  setInterval(check, 10000);
}

/* Start the app when DOM is ready */
document.addEventListener(
  "DOMContentLoaded", initApp
);

/* Shared utility helpers for Jarvis frontend. */

/**
 * Shorthand for document.getElementById.
 */
function $(id) {
  return document.getElementById(id);
}

/**
 * Shorthand for document.querySelectorAll.
 */
function $$(selector) {
  return document.querySelectorAll(selector);
}

/**
 * Generate a short random session ID.
 */
function generateId() {
  var ts = Date.now().toString(36);
  var rand = Math.random().toString(36).slice(2, 7);
  return ts + "-" + rand;
}

/**
 * Escape HTML special characters.
 */
function escapeHtml(text) {
  var div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Check if the server is reachable.
 */
async function checkHealth() {
  try {
    var resp = await fetch("/api/health");
    var data = await resp.json();
    return data.status === "ok";
  } catch (e) {
    return false;
  }
}

/**
 * Update the connection status indicator.
 */
function setOnline(isOnline) {
  var dot = $("status-dot");
  var text = $("status-text");
  if (isOnline) {
    dot.classList.add("online");
    text.textContent = "Online";
  } else {
    dot.classList.remove("online");
    text.textContent = "Offline";
  }
}

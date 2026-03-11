/* Jobs module — display running/completed jobs. */

var AUTO_JOBS_URL = "http://localhost:8001";

/**
 * Initialize the jobs module.
 */
function initJobs() {
  loadJobs();
  /* Refresh every 5 seconds */
  setInterval(loadJobs, 5000);
}

/**
 * Load and display jobs from server.
 */
async function loadJobs() {
  var content = $("jobs-content");
  try {
    var resp = await fetch(
      AUTO_JOBS_URL + "/jobs/list"
    );
    if (!resp.ok) {
      content.innerHTML =
        '<p class="error-text">' +
        "Cannot reach server</p>";
      return;
    }
    var jobs = await resp.json();
    renderJobs(jobs);
  } catch (e) {
    content.innerHTML =
      '<p class="empty-state">' +
      "Cannot reach automation server" +
      "</p>";
  }
}

/**
 * Render the jobs list.
 */
function renderJobs(jobs) {
  var content = $("jobs-content");

  if (!jobs || jobs.length === 0) {
    content.innerHTML =
      '<p class="empty-state">' +
      "No jobs running</p>";
    return;
  }

  content.innerHTML = "";

  /* Sort: running first, then by time */
  jobs.sort(function (a, b) {
    if (a.status === "running") return -1;
    if (b.status === "running") return 1;
    return (
      (b.created_at || 0) -
      (a.created_at || 0)
    );
  });

  jobs.forEach(function (job) {
    var div = document.createElement("div");
    div.classList.add("job-item");

    var statusClass = job.status || "pending";
    var timeStr = job.created_at
      ? formatTime(job.created_at)
      : "";

    div.innerHTML =
      '<span class="job-status ' +
      escapeHtml(statusClass) +
      '"></span>' +
      '<span class="job-name">' +
      escapeHtml(job.type || "unknown") +
      " — " +
      escapeHtml(job.status || "?") +
      "</span>" +
      '<span class="job-time">' +
      escapeHtml(timeStr) +
      "</span>";

    content.appendChild(div);
  });
}

/**
 * Format a Unix timestamp to time string.
 */
function formatTime(ts) {
  var d = new Date(ts * 1000);
  return d.toLocaleTimeString();
}

/* Files module — browse & upload files. */

var currentPath = ".";
var AUTO_URL = "http://localhost:8001";

/**
 * Initialize the files module.
 */
function initFiles() {
  var dropZone = $("file-drop-zone");
  var fileInput = $("file-input");

  dropZone.addEventListener(
    "click", function () {
      fileInput.click();
    }
  );

  dropZone.addEventListener(
    "dragover", function (e) {
      e.preventDefault();
      dropZone.classList.add("drag-over");
    }
  );

  dropZone.addEventListener(
    "dragleave", function () {
      dropZone.classList.remove("drag-over");
    }
  );

  dropZone.addEventListener(
    "drop", function (e) {
      e.preventDefault();
      dropZone.classList.remove("drag-over");
      handleFileUpload(e.dataTransfer.files);
    }
  );

  fileInput.addEventListener(
    "change", function () {
      handleFileUpload(this.files);
      this.value = "";
    }
  );

  loadFileList(".");
}

/**
 * Load and display file listing.
 */
async function loadFileList(path) {
  currentPath = path;
  var list = $("file-list");
  list.innerHTML = '<p class="loading-text">' +
    "Loading...</p>";

  try {
    var url = AUTO_URL + "/files/list" +
      "?path=" + encodeURIComponent(path);
    var resp = await fetch(url);
    var data = await resp.json();

    if (data.error) {
      list.innerHTML =
        '<p class="error-text">' +
        escapeHtml(data.error) + "</p>";
      return;
    }

    renderFileList(data.files, path);
  } catch (e) {
    list.innerHTML =
      '<p class="error-text">' +
      "Cannot reach automation server" +
      "</p>";
  }
}

/**
 * Render the file list UI.
 */
function renderFileList(files, path) {
  var list = $("file-list");
  list.innerHTML = "";

  /* Breadcrumb / path bar */
  var pathBar = document.createElement("div");
  pathBar.classList.add("file-path-bar");
  pathBar.innerHTML = buildBreadcrumb(path);
  list.appendChild(pathBar);

  if (path !== ".") {
    var parent = parentPath(path);
    var upItem = createFileItem(
      "📁", ".. (up)", true
    );
    upItem.addEventListener(
      "click", function () {
        loadFileList(parent);
      }
    );
    list.appendChild(upItem);
  }

  files.forEach(function (f) {
    var isDir = f.endsWith("/");
    var name = isDir ? f.slice(0, -1) : f;
    var display = name.split("/").pop();
    var icon = isDir ? "📁" : fileIcon(display);

    var item = createFileItem(
      icon, display, isDir
    );

    if (isDir) {
      item.addEventListener(
        "click", function () {
          loadFileList(name);
        }
      );
    } else {
      item.addEventListener(
        "click", function () {
          viewFile(name);
        }
      );
    }

    list.appendChild(item);
  });
}

/**
 * Create a file list item element.
 */
function createFileItem(icon, name, isDir) {
  var div = document.createElement("div");
  div.classList.add("file-item");
  if (isDir) div.classList.add("file-dir");
  div.innerHTML =
    '<span class="file-icon">' +
    icon + "</span>" +
    '<span class="file-name">' +
    escapeHtml(name) + "</span>";
  return div;
}

/**
 * View a file's content.
 */
async function viewFile(path) {
  try {
    var resp = await fetch(
      AUTO_URL + "/files/read",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({ path: path }),
      }
    );
    var data = await resp.json();
    if (data.error) {
      alert("Error: " + data.error);
      return;
    }
    showFileViewer(path, data.content);
  } catch (e) {
    alert("Cannot read file");
  }
}

/**
 * Show file content in a viewer overlay.
 */
function showFileViewer(path, content) {
  var name = path.split("/").pop();
  var list = $("file-list");
  list.innerHTML = "";

  var viewer = document.createElement("div");
  viewer.classList.add("file-viewer");

  var header = document.createElement("div");
  header.classList.add("file-viewer-header");
  header.innerHTML =
    '<span class="file-viewer-name">' +
    escapeHtml(name) + "</span>" +
    '<button class="file-viewer-close">' +
    "✕ Close</button>";

  var pre = document.createElement("pre");
  pre.classList.add("file-viewer-content");
  pre.textContent = content;

  viewer.appendChild(header);
  viewer.appendChild(pre);
  list.appendChild(viewer);

  header.querySelector(
    ".file-viewer-close"
  ).addEventListener("click", function () {
    loadFileList(parentPath(path));
  });
}

/**
 * Handle file upload via drop or input.
 */
async function handleFileUpload(fileList) {
  for (var i = 0; i < fileList.length; i++) {
    var file = fileList[i];
    try {
      var text = await readFileAsText(file);
      var path = currentPath === "."
        ? file.name
        : currentPath + "/" + file.name;
      await fetch(
        AUTO_URL + "/files/write",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            path: path,
            content: text,
          }),
        }
      );
    } catch (e) {
      alert("Upload failed: " + file.name);
    }
  }
  loadFileList(currentPath);
}

/**
 * Read a File object as text.
 */
function readFileAsText(file) {
  return new Promise(function (resolve, reject) {
    var reader = new FileReader();
    reader.onload = function () {
      resolve(reader.result);
    };
    reader.onerror = reject;
    reader.readAsText(file);
  });
}

/**
 * Build breadcrumb HTML from path.
 */
function buildBreadcrumb(path) {
  if (path === ".") return "📂 /";
  var parts = path.split("/");
  var html = "📂 / ";
  for (var i = 0; i < parts.length; i++) {
    html += escapeHtml(parts[i]);
    if (i < parts.length - 1) html += " / ";
  }
  return html;
}

/**
 * Get parent path from a path string.
 */
function parentPath(path) {
  var parts = path.split("/");
  parts.pop();
  return parts.length ? parts.join("/") : ".";
}

/**
 * Return an icon for a file extension.
 */
function fileIcon(name) {
  var ext = name.split(".").pop().toLowerCase();
  var icons = {
    md: "📝", txt: "📄", py: "🐍",
    js: "📜", json: "📋", html: "🌐",
    css: "🎨", sh: "⚙️", toml: "⚙️",
    yaml: "⚙️", yml: "⚙️",
  };
  return icons[ext] || "📄";
}

/* Chat module — handles sending messages
   and displaying responses via SSE streaming.
   Supports file attachments (images, text). */

var currentSessionId = null;
var isSending = false;
var pendingFiles = []; /* {file, dataUrl, type} */

/**
 * Initialize the chat module.
 */
function initChat() {
  var input = $("chat-input");
  var sendBtn = $("send-btn");
  var attachBtn = $("attach-btn");
  var fileInput = $("chat-file-input");

  sendBtn.addEventListener("click", handleSend);
  attachBtn.addEventListener(
    "click", function () {
      fileInput.click();
    }
  );
  fileInput.addEventListener(
    "change", handleFileSelect
  );

  input.addEventListener("keydown", function (e) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  });

  /* Auto-resize textarea */
  input.addEventListener("input", function () {
    this.rows = 1;
    var lines = Math.ceil(
      this.scrollHeight / 21
    );
    this.rows = Math.min(lines, 6);
  });

  /* Drag and drop on chat view */
  setupChatDragDrop();

  /* Start a new session */
  newSession();
  showWelcome();
}

/**
 * Set up drag-and-drop on the chat view.
 */
function setupChatDragDrop() {
  var chatView = $("chat-view");
  var counter = 0;

  chatView.addEventListener(
    "dragenter", function (e) {
      e.preventDefault();
      counter++;
      chatView.classList.add("drag-over");
    }
  );
  chatView.addEventListener(
    "dragleave", function (e) {
      e.preventDefault();
      counter--;
      if (counter <= 0) {
        counter = 0;
        chatView.classList.remove("drag-over");
      }
    }
  );
  chatView.addEventListener(
    "dragover", function (e) {
      e.preventDefault();
    }
  );
  chatView.addEventListener(
    "drop", function (e) {
      e.preventDefault();
      counter = 0;
      chatView.classList.remove("drag-over");
      if (e.dataTransfer.files.length > 0) {
        addFiles(e.dataTransfer.files);
      }
    }
  );
}

/**
 * Handle file input selection.
 */
function handleFileSelect(e) {
  if (e.target.files.length > 0) {
    addFiles(e.target.files);
  }
  e.target.value = "";
}

/**
 * Add files to pending attachments.
 */
function addFiles(fileList) {
  for (var i = 0; i < fileList.length; i++) {
    var file = fileList[i];
    if (pendingFiles.length >= 5) break;
    readFileAsDataUrl(file);
  }
}

/**
 * Read a file and add to pending list.
 */
function readFileAsDataUrl(file) {
  var reader = new FileReader();
  reader.onload = function (e) {
    var isImage = file.type.startsWith("image/");
    pendingFiles.push({
      file: file,
      name: file.name,
      dataUrl: e.target.result,
      isImage: isImage,
      mimeType: file.type || "application/octet-stream",
    });
    renderAttachmentPreview();
  };
  reader.readAsDataURL(file);
}

/**
 * Render attachment chips in the preview area.
 */
function renderAttachmentPreview() {
  var preview = $("attachment-preview");
  preview.innerHTML = "";

  pendingFiles.forEach(function (pf, idx) {
    var chip = document.createElement("div");
    chip.classList.add("attachment-chip");

    var inner = "";
    if (pf.isImage) {
      inner +=
        '<img src="' +
        pf.dataUrl +
        '" alt="preview">';
    } else {
      inner += '<span>📄</span>';
    }
    inner +=
      '<span class="attachment-chip-name">' +
      escapeHtml(pf.name) +
      "</span>" +
      '<button class="attachment-chip-remove"' +
      ' data-idx="' + idx + '">✕</button>';

    chip.innerHTML = inner;
    preview.appendChild(chip);
  });

  /* Bind remove buttons */
  var btns = preview.querySelectorAll(
    ".attachment-chip-remove"
  );
  btns.forEach(function (btn) {
    btn.addEventListener("click", function () {
      var idx = parseInt(
        this.getAttribute("data-idx")
      );
      pendingFiles.splice(idx, 1);
      renderAttachmentPreview();
    });
  });
}

/**
 * Create a new chat session.
 */
function newSession() {
  currentSessionId = generateId();
  $("messages").innerHTML = "";
  $("chat-title").textContent = "New Chat";
  pendingFiles = [];
  renderAttachmentPreview();
  showWelcome();
}

/**
 * Show the welcome message.
 */
function showWelcome() {
  var msgs = $("messages");
  msgs.innerHTML =
    '<div class="welcome-message">' +
    '<span class="welcome-icon">⚡</span>' +
    '<span class="welcome-title">' +
    "Hello! I'm Jarvis" +
    "</span>" +
    '<span class="welcome-subtitle">' +
    "Your local AI assistant. " +
    "Type a message or drop files " +
    "to get started." +
    "</span>" +
    "</div>";
}

/**
 * Handle the send button click.
 */
function handleSend() {
  if (isSending) return;
  var input = $("chat-input");
  var text = input.value.trim();
  var files = pendingFiles.slice();

  if (!text && files.length === 0) return;

  input.value = "";
  input.rows = 1;
  pendingFiles = [];
  renderAttachmentPreview();

  /* Clear welcome if present */
  var welcome = document.querySelector(
    ".welcome-message"
  );
  if (welcome) welcome.remove();

  /* Show user message with attachments */
  addUserMessage(text, files);
  sendStreaming(text, files);
  setTimeout(loadSessionList, 2000);
}

/**
 * Add a user message with optional images.
 */
function addUserMessage(text, files) {
  var msgs = $("messages");
  var div = document.createElement("div");
  div.classList.add("message", "message-user");

  var bodyHtml = "";
  /* Show attached images */
  files.forEach(function (pf) {
    if (pf.isImage) {
      bodyHtml +=
        '<img class="msg-image" src="' +
        pf.dataUrl + '" alt="' +
        escapeHtml(pf.name) + '">';
    } else {
      bodyHtml +=
        '<div>📎 ' +
        escapeHtml(pf.name) + '</div>';
    }
  });
  if (text) {
    bodyHtml += escapeHtml(text);
  }

  div.innerHTML =
    '<div class="message-role">You</div>' +
    '<div class="message-body">' +
    bodyHtml + "</div>";

  msgs.appendChild(div);
  scrollToBottom();
}

/**
 * Add an assistant message bubble.
 */
function addMessage(role, content) {
  var msgs = $("messages");
  var div = document.createElement("div");
  div.classList.add("message");
  div.classList.add("message-" + role);

  var roleLabel = role === "user"
    ? "You" : "Jarvis";
  var bodyHtml = role === "assistant"
    ? renderMarkdown(content)
    : escapeHtml(content);

  div.innerHTML =
    '<div class="message-role">' +
    roleLabel + "</div>" +
    '<div class="message-body">' +
    bodyHtml + "</div>";

  msgs.appendChild(div);
  scrollToBottom();
  return div;
}

/**
 * Create empty assistant message for streaming.
 */
function addStreamingMessage() {
  var msgs = $("messages");
  var div = document.createElement("div");
  div.classList.add("message");
  div.classList.add("message-assistant");
  div.id = "streaming-msg";
  div.innerHTML =
    '<div class="message-role">Jarvis</div>' +
    '<div class="message-body"></div>';
  msgs.appendChild(div);
  scrollToBottom();
  return div;
}

/**
 * Show loading dots while waiting.
 */
function showLoading() {
  var msgs = $("messages");
  var div = document.createElement("div");
  div.classList.add("message");
  div.classList.add("message-assistant");
  div.id = "loading-msg";
  div.innerHTML =
    '<div class="message-role">Jarvis</div>' +
    '<div class="loading-dots">' +
    '<span class="loading-dot"></span>' +
    '<span class="loading-dot"></span>' +
    '<span class="loading-dot"></span>' +
    "</div>";
  msgs.appendChild(div);
  scrollToBottom();
}

/**
 * Remove the loading indicator.
 */
function hideLoading() {
  var el = $("loading-msg");
  if (el) el.remove();
}

/**
 * Scroll the messages area to the bottom.
 */
function scrollToBottom() {
  var msgs = $("messages");
  msgs.scrollTop = msgs.scrollHeight;
}

/**
 * Build content array for OpenAI vision API.
 */
function buildContent(text, files) {
  /* No files — simple string */
  if (!files || files.length === 0) {
    return text || "";
  }

  /* With files — multipart content array */
  var parts = [];

  files.forEach(function (pf) {
    if (pf.isImage) {
      parts.push({
        type: "image_url",
        image_url: {
          url: pf.dataUrl,
          detail: "auto",
        },
      });
    } else {
      /* Text file: extract base64 content */
      var b64 = pf.dataUrl.split(",")[1] || "";
      var decoded = atob(b64);
      parts.push({
        type: "text",
        text: "File: " + pf.name +
          "\n```\n" + decoded + "\n```",
      });
    }
  });

  if (text) {
    parts.push({ type: "text", text: text });
  }

  return parts;
}

/**
 * Send message via SSE streaming.
 */
async function sendStreaming(text, files) {
  isSending = true;
  $("send-btn").classList.add("disabled");
  showLoading();

  var content = buildContent(text, files);

  try {
    var resp = await fetch("/api/chat", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        message: text || "Analyze this file",
        content: content,
        session_id: currentSessionId,
        stream: true
      })
    });

    hideLoading();

    if (!resp.ok) {
      addMessage(
        "assistant",
        "Error: server returned " + resp.status
      );
      return;
    }

    var msgDiv = addStreamingMessage();
    var bodyEl = msgDiv.querySelector(
      ".message-body"
    );
    var fullText = "";

    var reader = resp.body.getReader();
    var decoder = new TextDecoder();
    var buffer = "";

    while (true) {
      var result = await reader.read();
      if (result.done) break;

      buffer += decoder.decode(
        result.value, { stream: true }
      );

      var lines = buffer.split("\n");
      buffer = lines.pop();

      for (var i = 0; i < lines.length; i++) {
        var line = lines[i].trim();
        if (!line.startsWith("data: ")) continue;

        var data = line.slice(6);
        if (data === "[DONE]") continue;

        try {
          var obj = JSON.parse(data);
          if (obj.token) {
            fullText += obj.token;
            bodyEl.innerHTML = renderMarkdown(
              fullText
            );
            scrollToBottom();
          }
        } catch (e) {
          /* skip parse errors */
        }
      }
    }

    /* Remove streaming ID */
    msgDiv.removeAttribute("id");

  } catch (e) {
    hideLoading();
    addMessage(
      "assistant",
      "Error: could not reach server. " +
      "Is Jarvis running?"
    );
  } finally {
    isSending = false;
    $("send-btn").classList.remove("disabled");
  }
}

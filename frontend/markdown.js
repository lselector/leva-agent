/* Simple markdown to HTML renderer. */

/**
 * Convert markdown text to HTML.
 * Supports: bold, italic, code, code blocks,
 * links, lists, blockquotes, headings.
 */
function renderMarkdown(text) {
  if (!text) return "";

  var html = escapeHtml(text);

  /* Code blocks: ```...``` */
  html = html.replace(
    /```(\w*)\n([\s\S]*?)```/g,
    function (m, lang, code) {
      return "<pre><code>" + code.trim() +
        "</code></pre>";
    }
  );

  /* Inline code: `...` */
  html = html.replace(
    /`([^`]+)`/g,
    "<code>$1</code>"
  );

  /* Bold: **...** */
  html = html.replace(
    /\*\*(.+?)\*\*/g,
    "<strong>$1</strong>"
  );

  /* Italic: *...* */
  html = html.replace(
    /\*(.+?)\*/g,
    "<em>$1</em>"
  );

  /* Links: [text](url) */
  html = html.replace(
    /\[([^\]]+)\]\(([^)]+)\)/g,
    '<a href="$2" target="_blank">$1</a>'
  );

  /* Blockquotes: > text */
  html = html.replace(
    /^&gt; (.+)$/gm,
    "<blockquote>$1</blockquote>"
  );

  /* Unordered lists: - item */
  html = html.replace(
    /^- (.+)$/gm,
    "<li>$1</li>"
  );
  html = html.replace(
    /(<li>.*<\/li>\n?)+/g,
    function (m) { return "<ul>" + m + "</ul>"; }
  );

  /* Headings: ### text */
  html = html.replace(
    /^### (.+)$/gm,
    "<h3>$1</h3>"
  );
  html = html.replace(
    /^## (.+)$/gm,
    "<h2>$1</h2>"
  );
  html = html.replace(
    /^# (.+)$/gm,
    "<h1>$1</h1>"
  );

  /* Paragraphs: double newline */
  html = html.replace(/\n\n/g, "</p><p>");
  html = "<p>" + html + "</p>";

  /* Clean up empty paragraphs */
  html = html.replace(/<p>\s*<\/p>/g, "");

  /* Fix nested block elements in <p> */
  html = html.replace(
    /<p>(<(?:pre|ul|ol|blockquote|h[1-6]))/g,
    "$1"
  );
  html = html.replace(
    /(<\/(?:pre|ul|ol|blockquote|h[1-6])>)<\/p>/g,
    "$1"
  );

  return html;
}

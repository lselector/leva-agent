"""High-level browser actions via CDP."""
import time

from .client import (
    close_tab,
    execute_js,
    get_page_text,
    navigate,
    new_tab,
    wait_for_load,
)

# ---------------------------------------------------------------
# Web search


def web_search(query: str) -> list[dict]:
    """Search Google and return top results.

    Returns list of dicts: title, url, snippet.
    """
    import urllib.parse
    search_url = (
        "https://www.google.com/search?q="
        + urllib.parse.quote_plus(query)
    )
    tab = new_tab(search_url)
    try:
        time.sleep(2)  # let JS render results

        script = """
        (function() {
            var results = [];
            // Google 2025+ DOM: h3 inside .zReHs containers
            var headings = document.querySelectorAll('h3');
            for (var i = 0; i < headings.length && results.length < 10; i++) {
                var h = headings[i];
                // Walk up to find the nearest anchor
                var a = h.closest('a') ||
                        h.querySelector('a') ||
                        h.parentElement && h.parentElement.closest('a');
                if (!a || !a.href || a.href.startsWith('https://www.google')) continue;
                // Snippet: next sibling text block
                var container = h.closest('[data-sokoban-container]') ||
                                h.closest('.MjjYud') ||
                                h.parentElement;
                var snippetEl = container
                    ? container.querySelector('[data-sncf="1"]') ||
                      container.querySelector('.VwiC3b') ||
                      container.querySelector('span[style]')
                    : null;
                results.push({
                    title: h.textContent.trim(),
                    url: a.href,
                    snippet: snippetEl ? snippetEl.textContent.trim() : ''
                });
            }
            return JSON.stringify(results);
        })()
        """
        raw = execute_js(tab, script)
        import json
        results = json.loads(raw) if raw else []
        return [r for r in results if r.get("title")]
    finally:
        close_tab(tab)


# ---------------------------------------------------------------
# Web fetch


def web_fetch(url: str, max_chars: int = 10000) -> str:
    """Fetch a URL and return its main text content.

    Extracts article/main content, strips noise,
    truncates to max_chars.
    """
    tab = new_tab(url)
    try:
        time.sleep(2)  # let JS-heavy pages render

        script = """
        (function() {
            // Try semantic content containers first
            var el = document.querySelector('article') ||
                     document.querySelector('main') ||
                     document.querySelector('[role="main"]') ||
                     document.querySelector('.content') ||
                     document.querySelector('#content') ||
                     document.body;
            if (!el) return document.body.innerText;

            // Remove noisy elements
            var noisy = el.querySelectorAll(
                'nav, footer, header, aside, ' +
                '.nav, .footer, .header, .sidebar, ' +
                '.advertisement, .ad, script, style'
            );
            noisy.forEach(function(n) { n.remove(); });

            return el.innerText || el.textContent || '';
        })()
        """
        text = execute_js(tab, script) or ""
        text = text.strip()
        if len(text) > max_chars:
            text = text[:max_chars] + "\n...[truncated]"
        return text
    finally:
        close_tab(tab)


# ---------------------------------------------------------------
# Web screenshot


def web_screenshot(url: str) -> bytes:
    """Navigate to url and take a PNG screenshot."""
    from .client import take_screenshot
    tab = new_tab(url)
    try:
        time.sleep(2)
        return take_screenshot(tab)
    finally:
        close_tab(tab)


# ---------------------------------------------------------------
# LinkedIn


def linkedin_feed() -> list[dict]:
    """Read LinkedIn feed and return recent posts.

    Returns list of dicts: author, text, likes.
    """
    tab = new_tab("https://www.linkedin.com/feed/")
    try:
        time.sleep(4)  # feed takes a few seconds

        # Scroll down to load more posts
        execute_js(tab, "window.scrollBy(0, 800)")
        time.sleep(1)

        script = """
        (function() {
            var posts = [];
            var items = document.querySelectorAll(
                '.feed-shared-update-v2'
            );
            for (var i = 0; i < items.length; i++) {
                var el = items[i];
                var authorEl = el.querySelector(
                    '.update-components-actor__name'
                );
                var textEl = el.querySelector(
                    '.feed-shared-text'
                ) || el.querySelector(
                    '.update-components-text'
                );
                var likesEl = el.querySelector(
                    '.social-details-social-counts'
                );
                if (authorEl) {
                    posts.push({
                        author: authorEl.textContent.trim(),
                        text: textEl
                            ? textEl.textContent.trim()
                            : '',
                        likes: likesEl
                            ? likesEl.textContent.trim()
                            : ''
                    });
                }
            }
            return JSON.stringify(posts);
        })()
        """
        raw = execute_js(tab, script)
        import json
        posts = json.loads(raw) if raw else []
        return [p for p in posts if p.get("author")]
    finally:
        close_tab(tab)


def linkedin_like(
    post_index: int = 0, dry_run: bool = True
) -> dict:
    """Like a LinkedIn feed post by index.

    If dry_run=True, returns post info without clicking.
    """
    tab = new_tab("https://www.linkedin.com/feed/")
    try:
        time.sleep(4)

        # Get posts info
        script = """
        (function() {
            var items = document.querySelectorAll(
                '.feed-shared-update-v2'
            );
            var el = items[%d];
            if (!el) return JSON.stringify({error: 'post not found'});
            var authorEl = el.querySelector(
                '.update-components-actor__name'
            );
            var textEl = el.querySelector('.feed-shared-text') ||
                         el.querySelector('.update-components-text');
            var likeBtn = el.querySelector(
                'button[aria-label*="Like"]'
            ) || el.querySelector(
                '.reactions-react-button'
            );
            return JSON.stringify({
                author: authorEl
                    ? authorEl.textContent.trim() : '',
                text: textEl
                    ? textEl.textContent.trim().slice(0, 200) : '',
                like_button_found: likeBtn !== null
            });
        })()
        """ % post_index

        import json
        raw = execute_js(tab, script)
        info = json.loads(raw) if raw else {}

        if dry_run:
            info["dry_run"] = True
            info["action"] = "not liked (dry run)"
            return info

        # Click the like button
        click_script = """
        (function() {
            var items = document.querySelectorAll(
                '.feed-shared-update-v2'
            );
            var el = items[%d];
            if (!el) return 'post not found';
            var btn = el.querySelector(
                'button[aria-label*="Like"]'
            ) || el.querySelector(
                '.reactions-react-button'
            );
            if (!btn) return 'like button not found';
            btn.click();
            return 'clicked';
        })()
        """ % post_index

        result = execute_js(tab, click_script)
        time.sleep(1)
        info["action"] = result
        return info
    finally:
        close_tab(tab)

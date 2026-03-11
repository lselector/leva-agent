"""CDP Browser client — connects to Chrome via Chrome DevTools Protocol."""
import json
import threading
import time

import pychrome

# Suppress pychrome's JSONDecodeError noise when tabs are closed.
# The _recv_loop thread gets an empty WebSocket frame on close and
# raises JSONDecodeError in an unhandled thread — we silence it here.
_original_excepthook = threading.excepthook


def _quiet_excepthook(args):
    if args.exc_type is json.JSONDecodeError:
        return
    _original_excepthook(args)


threading.excepthook = _quiet_excepthook

CDP_URL = "http://127.0.0.1:9222"

_browser = None


def get_browser() -> pychrome.Browser:
    """Connect to Chrome running with --remote-debugging-port=9222."""
    global _browser
    if _browser is None:
        _browser = pychrome.Browser(url=CDP_URL)
    return _browser


def list_tabs() -> list[dict]:
    """Return list of open tabs as dicts with id, url, title."""
    browser = get_browser()
    tabs = browser.list_tab()
    return [{"id": t.id, "url": t.url, "title": getattr(t, "title", "")} for t in tabs]


def new_tab(url: str = "about:blank") -> pychrome.Tab:
    """Open a new Chrome tab and navigate to url."""
    browser = get_browser()
    tab = browser.new_tab()
    tab.start()
    tab.Page.enable()
    navigate(tab, url)
    return tab


def close_tab(tab: pychrome.Tab) -> None:
    """Stop and close a tab."""
    import logging
    # Suppress pychrome background thread noise on close
    logging.getLogger("pychrome").setLevel(logging.CRITICAL)
    try:
        tab.stop()
    except Exception:
        pass
    browser = get_browser()
    try:
        browser.close_tab(tab)
    except Exception:
        pass


def navigate(tab: pychrome.Tab, url: str) -> None:
    """Navigate tab to url and wait for load."""
    tab.Page.navigate(url=url)
    wait_for_load(tab)


def wait_for_load(tab: pychrome.Tab, timeout: float = 10.0) -> None:
    """Wait for page to finish loading (polls document.readyState)."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            result = tab.Runtime.evaluate(
                expression="document.readyState"
            )
            if result.get("result", {}).get("value") == "complete":
                return
        except Exception:
            pass
        time.sleep(0.3)


def execute_js(tab: pychrome.Tab, script: str):
    """Execute JavaScript in tab and return the result value."""
    result = tab.Runtime.evaluate(expression=script, returnByValue=True)
    return result.get("result", {}).get("value")


def get_page_text(tab: pychrome.Tab) -> str:
    """Extract visible text content from the page."""
    script = """
    (function() {
        var el = document.querySelector('article') ||
                 document.querySelector('main') ||
                 document.querySelector('[role="main"]') ||
                 document.body;
        return el ? el.innerText : document.body.innerText;
    })()
    """
    return execute_js(tab, script) or ""


def take_screenshot(tab: pychrome.Tab) -> bytes:
    """Take a screenshot and return PNG bytes."""
    result = tab.Page.captureScreenshot(format="png")
    import base64
    return base64.b64decode(result["data"])

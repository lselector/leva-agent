"""Browser manager using Playwright."""

import asyncio
import os
from playwright.async_api import (
    async_playwright,
    Browser,
    BrowserContext,
    Page,
)

# Chrome profile path — set via env or default
CHROME_PROFILE = os.environ.get(
    "CHROME_PROFILE",
    "/Users/levselector/Library/"
    "Application Support/Google/"
    "Chrome/Profile 1",
)

# Set to True for headless (no GUI)
HEADLESS = os.environ.get(
    "BROWSER_HEADLESS", "false"
).lower() == "true"


# --------------------------------------------------------------
class BrowserManager:
    """Manages a shared Playwright browser."""

    def __init__(self):
        """Initialize browser manager."""
        self._pw = None
        self._browser = None
        self._context = None
        self._page = None

    async def _ensure_browser(self):
        """Launch browser with Chrome profile."""
        if self._context is None:
            self._pw = await async_playwright(
            ).start()
            self._context = (
                await self._pw.chromium
                .launch_persistent_context(
                    user_data_dir=CHROME_PROFILE,
                    headless=HEADLESS,
                    viewport={
                        "width": 1280,
                        "height": 720,
                    },
                    channel="chrome",
                    args=[
                        "--disable-blink-features"
                        "=AutomationControlled",
                    ],
                    ignore_default_args=[
                        "--enable-automation",
                    ],
                )
            )

    async def get_page(self) -> Page:
        """Get or create a page."""
        await self._ensure_browser()
        if self._page is None or (
            self._page.is_closed()
        ):
            self._page = (
                await self._context.new_page()
            )
        return self._page

    async def navigate(self, url: str):
        """Navigate to a URL, return title."""
        page = await self.get_page()
        await page.goto(
            url,
            wait_until="domcontentloaded",
            timeout=15000,
        )
        title = await page.title()
        return {"status": "ok", "title": title}

    async def extract_text(self, url: str):
        """Extract text content from a URL."""
        page = await self.get_page()
        await page.goto(
            url,
            wait_until="domcontentloaded",
            timeout=15000,
        )
        text = await page.inner_text("body")
        # Truncate to 10K chars
        if len(text) > 10000:
            text = text[:10000] + "\n...[truncated]"
        return {"content": text}

    async def screenshot(self, url: str):
        """Take a screenshot, return bytes."""
        page = await self.get_page()
        await page.goto(
            url,
            wait_until="domcontentloaded",
            timeout=15000,
        )
        data = await page.screenshot(
            type="png",
            full_page=False,
        )
        return data

    async def click_element(
        self, selector: str
    ):
        """Click an element on current page."""
        page = await self.get_page()
        await page.click(
            selector, timeout=5000
        )
        return {"status": "ok"}

    async def type_text(
        self, selector: str, text: str
    ):
        """Type text into an element."""
        page = await self.get_page()
        await page.fill(
            selector, text, timeout=5000
        )
        return {"status": "ok"}

    async def get_page_content(self):
        """Get current page HTML."""
        page = await self.get_page()
        html = await page.content()
        if len(html) > 20000:
            html = html[:20000] + "...[truncated]"
        return {"html": html}

    async def close(self):
        """Close browser and cleanup."""
        if self._page and not (
            self._page.is_closed()
        ):
            await self._page.close()
        if self._context:
            await self._context.close()
        if self._browser:
            await self._browser.close()
        if self._pw:
            await self._pw.stop()
        self._page = None
        self._context = None
        self._browser = None
        self._pw = None


# Singleton instance
browser_mgr = BrowserManager()

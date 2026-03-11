"""LinkedIn browser automation actions."""

from .browser_manager import browser_mgr

LINKEDIN_FEED = "https://www.linkedin.com/feed/"


# --------------------------------------------------------------
async def get_feed():
    """Get recent posts from LinkedIn feed."""
    try:
        page = await browser_mgr.get_page()
        await page.goto(
            LINKEDIN_FEED,
            wait_until="domcontentloaded",
            timeout=20000,
        )
        # Wait for feed posts
        await page.wait_for_selector(
            "div.feed-shared-update-v2",
            timeout=10000,
        )
        posts_el = (
            await page.query_selector_all(
                "div.feed-shared-update-v2"
            )
        )
        posts = []
        limit = min(len(posts_el), 5)
        for i in range(limit):
            post = posts_el[i]
            author = await _safe_text(
                post,
                "span.update-components-actor"
                "__name",
            )
            text = await _safe_text(
                post,
                "div.feed-shared-update-v2"
                "__description",
            )
            if not text:
                text = await _safe_text(
                    post,
                    "span.break-words",
                )
            posts.append({
                "author": author,
                "text": _truncate(text, 300),
            })
        return {"posts": posts}
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
async def like_post(
    post_index: int = 0,
    dry_run: bool = True,
):
    """Like a post in the feed."""
    try:
        page = await browser_mgr.get_page()
        # Make sure we're on the feed
        if "linkedin.com/feed" not in (
            page.url
        ):
            await page.goto(
                LINKEDIN_FEED,
                wait_until="domcontentloaded",
                timeout=20000,
            )
        await page.wait_for_selector(
            "div.feed-shared-update-v2",
            timeout=10000,
        )
        posts = (
            await page.query_selector_all(
                "div.feed-shared-update-v2"
            )
        )
        if post_index >= len(posts):
            return {
                "error": (
                    f"Post index {post_index}"
                    f" out of range "
                    f"({len(posts)} posts)"
                ),
            }
        post = posts[post_index]
        # Find the like button
        like_btn = await post.query_selector(
            'button[aria-label*="Like"]'
        )
        if not like_btn:
            return {
                "error": "Like button not found"
            }
        if dry_run:
            text = await _safe_text(
                post, "span.break-words"
            )
            return {
                "status": "dry_run",
                "post_index": post_index,
                "post_preview": _truncate(
                    text, 200
                ),
            }
        await like_btn.click()
        await page.wait_for_timeout(1000)
        return {
            "status": "liked",
            "post_index": post_index,
        }
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
async def _safe_text(element, selector):
    """Safely extract text from selector."""
    try:
        el = await element.query_selector(
            selector
        )
        if el:
            return await el.inner_text()
        return ""
    except Exception:
        return ""


# --------------------------------------------------------------
def _truncate(text, max_len):
    """Truncate text to max length."""
    if not text:
        return ""
    text = text.strip()
    if len(text) > max_len:
        return text[:max_len] + "..."
    return text

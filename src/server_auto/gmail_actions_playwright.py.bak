"""Gmail browser automation actions."""

from .browser_manager import browser_mgr

GMAIL_URL = "https://mail.google.com/mail/u/0/"


# --------------------------------------------------------------
async def get_inbox():
    """Get recent emails from Gmail inbox."""
    try:
        page = await browser_mgr.get_page()
        await page.goto(
            GMAIL_URL + "#inbox",
            wait_until="commit",
            timeout=120000,
        )

        # Gmail loading screen — wait and
        # try to push past it
        for attempt in range(12):
            await page.wait_for_timeout(5000)

            # Click any "here" or reload links
            for sel in [
                'a[href*="zx="]',
                'a:has-text("here")',
                'a:has-text("click here")',
                'a:has-text("load")',
            ]:
                link = (
                    await page.query_selector(
                        sel
                    )
                )
                if link:
                    await link.click()
                    await page.wait_for_timeout(
                        3000
                    )
                    break

            # Check if inbox loaded
            rows = (
                await page.query_selector_all(
                    "tr.zA"
                )
            )
            if rows:
                break

            # Try reloading after 30s
            if attempt == 5:
                await page.reload(
                    timeout=60000
                )

        # Grab email rows
        rows = await page.query_selector_all(
            "tr.zA"
        )
        emails = []
        limit = min(len(rows), 15)
        for i in range(limit):
            row = rows[i]
            sender = await _safe_text(
                row, "span.yP, span.zF"
            )
            subject = await _safe_text(
                row, "span.bog"
            )
            snippet = await _safe_text(
                row, "span.y2"
            )
            emails.append({
                "from": sender,
                "subject": subject,
                "snippet": snippet,
            })
        return {"emails": emails}
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
async def _get_inbox_standard(page):
    """Fallback: standard Gmail selectors."""
    try:
        await page.goto(
            GMAIL_URL + "#inbox",
            wait_until="commit",
            timeout=90000,
        )
        await page.wait_for_timeout(10000)
        rows = await page.query_selector_all(
            "tr.zA"
        )
        emails = []
        limit = min(len(rows), 10)
        for i in range(limit):
            row = rows[i]
            sender = await _safe_text(
                row, "span.yP, span.zF"
            )
            subject = await _safe_text(
                row, "span.bog"
            )
            snippet = await _safe_text(
                row, "span.y2"
            )
            emails.append({
                "from": sender,
                "subject": subject,
                "snippet": snippet,
            })
        return {"emails": emails}
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
async def send_email(
    to: str, subject: str, body: str
):
    """Compose and send an email."""
    try:
        page = await browser_mgr.get_page()
        await page.goto(
            GMAIL_URL + "#inbox",
            wait_until="domcontentloaded",
            timeout=20000,
        )
        # Click compose button
        await page.click(
            "div.T-I.T-I-KE.L3",
            timeout=10000,
        )
        await page.wait_for_timeout(1000)

        # Fill To field
        await page.fill(
            'input[name="to"]',
            to,
            timeout=5000,
        )
        # Fill Subject
        await page.fill(
            'input[name="subjectbox"]',
            subject,
            timeout=5000,
        )
        # Fill Body
        await page.fill(
            'div[aria-label="Message Body"]',
            body,
            timeout=5000,
        )
        return {
            "status": "draft_ready",
            "to": to,
            "subject": subject,
            "note": "Draft composed. "
            "Call /browser/gmail/send-confirm "
            "to actually send.",
        }
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
async def confirm_send():
    """Click the Send button."""
    try:
        page = await browser_mgr.get_page()
        await page.click(
            'div[aria-label*="Send"]',
            timeout=5000,
        )
        await page.wait_for_timeout(2000)
        return {"status": "sent"}
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
async def _safe_inner(element):
    """Safely get inner text from element."""
    try:
        if element:
            return await element.inner_text()
        return ""
    except Exception:
        return ""

# Gmail Automation — Setup Instructions

## Overview

The Gmail automation in Leva uses Playwright
to control a Chromium browser and interact with
Gmail's web interface. It can read your inbox,
compose emails, and send them.

## ⚠️ Important: Requires Logged-In Browser

The browser must have access to your existing
Chrome profile where you're already logged into
Gmail. Without this, Gmail will just show a
Google login page.

## Files Involved

- `src/server_auto/browser_manager.py`
  — Manages the Playwright browser instance
- `src/server_auto/gmail_actions.py`
  — Gmail-specific actions (inbox, compose, send)
- `src/server_auto/routes_browser.py`
  — API endpoints for Gmail

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/browser/gmail/inbox` | Get recent emails |
| POST | `/browser/gmail/compose` | Compose draft |
| POST | `/browser/gmail/send-confirm` | Send it |

## Step-by-Step Setup

### Step 1: Find Your Chrome Profile Path

On macOS, the default Chrome profile is at:
```
/Users/levselector/Library/Application Support/Google/Chrome/Default
```

To find it, open Chrome and go to:
```
chrome://version
```
Look for "Profile Path" — that's the directory.

Found path: "/Users/levselector/Library/Application Support/Google/Chrome/Profile 1"


### Step 2: Update browser_manager.py

We need to change the browser manager to:
1. Use `launch_persistent_context()` instead
   of `launch()` + `new_context()` — this
   reuses your Chrome cookies and login sessions
2. Switch to `headless=False` so you can see
   the browser and handle any 2FA prompts
3. Point to your Chrome profile path

The key change is:
```python
# Instead of:
self._browser = await self._pw.chromium.launch(
    headless=True,
)
self._context = await self._browser.new_context(...)

# Use:
self._context = await self._pw.chromium.launch_persistent_context(
    user_data_dir="/path/to/chrome/profile",
    headless=False,
    viewport={"width": 1280, "height": 720},
)
```

### Step 3: Close Chrome First!

**Important**: Playwright cannot use a Chrome
profile that is currently open in another Chrome
instance. Before testing, you must:
1. Quit Google Chrome completely
2. Then start Leva

### Step 4: Start Leva

```bash
./start
```

### Step 5: Test Reading Inbox

In another terminal:
```bash
curl -s -X POST \
  http://localhost:8001/browser/gmail/inbox \
  | python -m json.tool
```

Expected response:
```json
{
  "emails": [
    {
      "from": "John Doe",
      "subject": "Meeting tomorrow",
      "snippet": "Hi, just confirming..."
    },
    ...
  ]
}
```

### Step 6: Test Composing an Email

```bash
curl -s -X POST \
  http://localhost:8001/browser/gmail/compose \
  -H "Content-Type: application/json" \
  -d '{
    "to": "test@example.com",
    "subject": "Test from Leva",
    "body": "Hello from Leva AI agent!"
  }' | python -m json.tool
```

Expected response:
```json
{
  "status": "draft_ready",
  "to": "test@example.com",
  "subject": "Test from Leva",
  "note": "Draft composed. Call
    /browser/gmail/send-confirm
    to actually send."
}
```

### Step 7: Send the Email (Optional)

**Warning**: This actually sends the email!
```bash
curl -s -X POST \
  http://localhost:8001/browser/gmail/send-confirm \
  | python -m json.tool
```

## Troubleshooting

### "Error: timeout" on inbox
- Gmail may take time to load. Try increasing
  the timeout in `gmail_actions.py`
- Make sure you're logged into Gmail in the
  Chrome profile being used

### "Error: browser closed"
- Make sure Chrome is fully quit before
  starting Leva (Playwright needs exclusive
  access to the profile)

### Gmail shows login page
- The Chrome profile path may be wrong
- Try `chrome://version` in Chrome to find
  the correct "Profile Path"

### 2FA / Security prompts
- Run with `headless=False` so you can see
  and interact with any security prompts
- After first successful login, subsequent
  runs should work without prompts

## Next Steps

To implement the Chrome profile support:
1. Add `CHROME_PROFILE` to `src/config.py`
2. Update `browser_manager.py` to use
   `launch_persistent_context()`
3. Test with `./start` and the curl commands
   above

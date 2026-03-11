# Gmail API — Setup & Implementation Plan

## Overview

Replace browser-based Gmail automation with
the official Gmail API using OAuth2. This is
faster, more reliable, and Google-approved.

## Why Gmail API > Browser Automation

| Feature | Browser | Gmail API |
|---------|---------|-----------|
| Speed | 30-60s load | < 1 second |
| Reliability | Fragile | Rock solid |
| Detection | Google blocks | Approved |
| Auth | Chrome profile | OAuth2 token |
| Headless | Problematic | Works fine |

## Prerequisites

- Google account (personal or Workspace)
- Access to Google Cloud Console
- Python packages: `google-api-python-client`,
  `google-auth-httplib2`, `google-auth-oauthlib`

## Step-by-Step Setup

### Step 1: Create Google Cloud Project

1. Go to https://console.cloud.google.com
2. Click "Select a project" → "New Project"
3. Name it "Jarvis Gmail" (or similar)
4. Click "Create"

### Step 2: Enable Gmail API

1. In the project, go to:
   APIs & Services → Library
2. Search for "Gmail API"
3. Click "Gmail API" → "Enable"

### Step 3: Configure OAuth Consent Screen

1. Go to: APIs & Services → OAuth consent screen
2. Choose "External" (or "Internal" if
   using Google Workspace)
3. Fill in:
   - App name: "Jarvis"
   - User support email: your email
   - Developer contact: your email
4. Click "Save and Continue"
5. Add scopes:
   - `https://www.googleapis.com/auth/gmail.readonly`
   - `https://www.googleapis.com/auth/gmail.send`
   - `https://www.googleapis.com/auth/gmail.compose`
   - `https://www.googleapis.com/auth/gmail.modify`
6. Click "Save and Continue"
7. Add test users: your Gmail address
8. Click "Save and Continue"

### Step 4: Create OAuth2 Credentials

1. Go to: APIs & Services → Credentials
2. Click "Create Credentials" → "OAuth client ID"
3. Application type: "Desktop app"
4. Name: "Jarvis Desktop"
5. Click "Create"
6. Click "Download JSON"
7. Save the file as:
   `credentials/gmail_credentials.json`
   in the project root

### Step 5: Install Python Packages

```bash
uv add google-api-python-client \
       google-auth-httplib2 \
       google-auth-oauthlib
```

### Step 6: First-Time Authentication

Run the auth script (we will create this):
```bash
python -m src.gmail_api.auth
```

This will:
1. Open your browser
2. Ask you to sign in to Google
3. Ask you to grant permissions
4. Save a token file at:
   `credentials/gmail_token.json`

After this one-time setup, the token is
reused automatically (and refreshed as needed).

### Step 7: Test the API

```bash
# Read inbox
curl -s -X POST \
  http://localhost:8001/api/gmail/inbox \
  | python -m json.tool

# Send email
curl -s -X POST \
  http://localhost:8001/api/gmail/send \
  -H "Content-Type: application/json" \
  -d '{
    "to": "test@example.com",
    "subject": "Test from Jarvis",
    "body": "Hello from Jarvis AI!"
  }' | python -m json.tool
```

## Implementation Plan

### Files to Create

```
credentials/
  gmail_credentials.json   ← from Google
  gmail_token.json         ← auto-generated
  .gitignore               ← ignore secrets

src/gmail_api/
  __init__.py
  auth.py          ← OAuth2 authentication
  client.py        ← Gmail API client
  routes.py        ← FastAPI endpoints
```

### File: src/gmail_api/auth.py

Handles OAuth2 flow:
- Load credentials from JSON file
- Check for existing token
- Refresh expired token
- Run browser auth flow if no token
- Save token for reuse

### File: src/gmail_api/client.py

Gmail API operations:
- `get_inbox(max_results=15)` — list emails
- `get_email(msg_id)` — read single email
- `send_email(to, subject, body)` — send
- `search_emails(query)` — search inbox
- `get_labels()` — list labels/folders

### File: src/gmail_api/routes.py

FastAPI endpoints:
- `POST /api/gmail/inbox` — get recent emails
- `POST /api/gmail/send` — send an email
- `POST /api/gmail/search` — search emails
- `GET  /api/gmail/email/{id}` — read email

### Wire into Server

Add routes to `src/server_auto/app.py`:
```python
from src.gmail_api.routes import router
app.include_router(router)
```

## API Endpoints (Final)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/gmail/inbox` | Recent emails |
| POST | `/api/gmail/send` | Send email |
| POST | `/api/gmail/search` | Search emails |
| GET | `/api/gmail/email/{id}` | Read email |

## Security Notes

- `credentials/` directory is gitignored
- Token file contains refresh token — keep safe
- OAuth scopes are minimal (read + send)
- Token auto-refreshes, no re-auth needed
- Can revoke access at:
  https://myaccount.google.com/permissions

## Troubleshooting

### "Access blocked: app not verified"
- This is normal for new OAuth apps
- Click "Advanced" → "Go to Jarvis (unsafe)"
- Or publish the app for production use

### "Token expired"
- The code auto-refreshes tokens
- If refresh fails, delete `gmail_token.json`
  and re-run auth

### "Insufficient permissions"
- Make sure all required scopes are added
  in the OAuth consent screen
- Delete token and re-authenticate

## Next Steps

After setup is complete:
1. Register Gmail tools with the LLM so
   Jarvis can read/send emails via chat
2. Add email summarization capabilities
3. Add draft management (save, edit, delete)

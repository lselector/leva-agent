# Progress: Email, Web Research & LinkedIn
## 2026-03-11

Reference:
- Plan: `20260311_plan01.md`
- Tasks: `20260311_plan01_tasks.md`

---

## Track A — Gmail API

### Task 1 — Install Gmail API Dependencies
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Added google-api-python-client>=2.100, google-auth-httplib2>=0.2, google-auth-oauthlib>=1.2 to pyproject.toml. uv sync installed 24 packages. All imports verified.

---

### Task 2 — Google Cloud Project Setup
- **Status**: ⏳ PARTIAL (manual steps needed)
- **Started**: 2026-03-11
- **Completed**:
- **Notes**: Created credentials/ dir with .gitignore (*.json). Added credentials/ to root .gitignore. MANUAL STEPS REQUIRED: create Google Cloud project, enable Gmail API, configure OAuth consent screen, create Desktop OAuth2 credentials, download as credentials/gmail_credentials.json.

---

### Task 3 — Gmail Auth Module
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created src/gmail_api/__init__.py and src/gmail_api/auth.py. get_gmail_service() handles token load/refresh/new flow. Imports verified. Requires credentials/gmail_credentials.json to run auth flow.

---

### Task 4 — Gmail Client Module
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created src/gmail_api/client.py. Functions: get_inbox, get_email, send_email, search_emails, get_labels. Handles multipart body extraction and attachment metadata. Imports verified.

---

### Task 5 — Gmail API Routes
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created src/gmail_api/routes.py. Endpoints: POST /api/gmail/inbox, GET /api/gmail/email/{id}, POST /api/gmail/send, POST /api/gmail/search, GET /api/gmail/labels. Wired into server_auto/app.py.

---

### Task 6 — Register Gmail as LLM Tools
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Added gmail_inbox, gmail_send, gmail_search to TOOLS dict and get_tools_schema() in src/tools/registry.py. Updated prompts/TOOLS.md.

---

## Track B — CDP + Web Research

### Task 7 — Install CDP Python Package
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Added pychrome>=0.2 to pyproject.toml alongside Gmail deps. uv sync installed pychrome==0.2.4. Import verified.

---

### Task 8 — CDP Client Module
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created src/cdp_browser/__init__.py and src/cdp_browser/client.py. Functions: get_browser, list_tabs, new_tab, close_tab, navigate, wait_for_load, execute_js, get_page_text, take_screenshot. Imports verified.

---

### Task 9 — Web Search Action
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Implemented web_search(query) in src/cdp_browser/actions.py. Opens Google search URL, extracts .g result elements (title, url, snippet) via JS. Returns up to 10 results.

---

### Task 10 — Web Fetch Action
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Implemented web_fetch(url, max_chars=10000) in src/cdp_browser/actions.py. Extracts article/main content, strips nav/footer/ads, truncates to 10k chars. Also added web_screenshot, linkedin_feed, linkedin_like to same file (Tasks 14/15 logic done early).

---

### Task 11 — CDP Web Routes
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created src/cdp_browser/routes.py. Endpoints: GET /api/web/tabs, POST /api/web/search, POST /api/web/fetch, POST /api/web/screenshot, POST /api/linkedin/feed, POST /api/linkedin/like. Wired into server_auto/app.py.

---

### Task 12 — Register Web as LLM Tools
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Added web_search, web_fetch to TOOLS dict and schemas. Updated prompts/TOOLS.md.

---

### Task 13 — Chrome Launch Helper Script
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created chrome_debug bash script. Checks if CDP already running on :9222 before launching. chmod +x applied.

---

## Track C — LinkedIn CDP

### Task 14 — LinkedIn Feed via CDP
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: linkedin_feed() added to src/cdp_browser/actions.py during Batch 3. Opens linkedin.com/feed, scrolls, extracts .feed-shared-update-v2 posts (author, text, likes) via JS.

---

### Task 15 — LinkedIn Like via CDP
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: linkedin_like(post_index, dry_run) added to src/cdp_browser/actions.py during Batch 3. Dry run returns post info without clicking. Real run clicks like button.

---

### Task 16 — LinkedIn Routes + LLM Tools
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Routes done in Task 11. LLM tools linkedin_feed, linkedin_like added to registry and schemas. Updated prompts/TOOLS.md.

---

## Track D — Polish + Test

### Task 17 — Perplexity API Fallback
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Created src/cdp_browser/perplexity.py. web_research(query) calls Perplexity sonar model. Added to registry as 20th tool. Requires PERPLEXITY_API_KEY in .env.

---

### Task 18 — Retire Playwright Code
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Removed Gmail+LinkedIn section from routes_browser.py. Archived gmail_actions.py → gmail_actions_playwright.py.bak and linkedin_actions.py → linkedin_actions_playwright.py.bak. Server loads cleanly. Old /browser/gmail and /browser/linkedin routes gone.

---

### Task 19 — Update Documentation
- **Status**: ✅ DONE
- **Started**: 2026-03-11
- **Completed**: 2026-03-11
- **Notes**: Updated prompts/SYSTEM.md (added email/web/LinkedIn capabilities section). Updated prompts/TOOLS.md (added web_research). Updated README.md (new features list + API endpoints table + setup instructions for Gmail OAuth, CDP, Perplexity).

---

### Task 20 — End-to-End Integration Test
- **Status**: NOT STARTED
- **Started**:
- **Completed**:
- **Notes**:

---

## Overall Progress

| Track | Tasks | Done | Status |
|-------|-------|------|--------|
| A — Gmail API | 1-6 | 6/6 (1 partial) | ✅ DONE |
| B — CDP + Web | 7-13 | 7/7 | ✅ DONE |
| C — LinkedIn | 14-16 | 3/3 | ✅ DONE |
| D — Polish | 17-20 | 3/4 | IN PROGRESS |
| **Total** | **20** | **19/20** | **IN PROGRESS** |

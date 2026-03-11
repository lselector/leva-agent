"""Gmail API operations."""
import base64
import email as email_lib
from email.mime.text import MIMEText
from typing import Any

from .auth import get_gmail_service


def get_inbox(max_results: int = 15) -> list[dict]:
    """List recent emails from inbox.

    Returns list of dicts: id, from, subject, snippet, date.
    """
    service = get_gmail_service()
    resp = service.users().messages().list(
        userId="me",
        labelIds=["INBOX"],
        maxResults=max_results,
    ).execute()

    messages = resp.get("messages", [])
    results = []
    for msg in messages:
        meta = service.users().messages().get(
            userId="me",
            id=msg["id"],
            format="metadata",
            metadataHeaders=["From", "Subject", "Date"],
        ).execute()
        headers = {
            h["name"]: h["value"]
            for h in meta.get("payload", {}).get("headers", [])
        }
        results.append({
            "id": msg["id"],
            "from": headers.get("From", ""),
            "subject": headers.get("Subject", ""),
            "date": headers.get("Date", ""),
            "snippet": meta.get("snippet", ""),
        })
    return results


def get_email(msg_id: str) -> dict:
    """Get full email by message ID.

    Returns dict: from, to, subject, body, date, attachments.
    """
    service = get_gmail_service()
    msg = service.users().messages().get(
        userId="me",
        id=msg_id,
        format="full",
    ).execute()

    payload = msg.get("payload", {})
    headers = {
        h["name"]: h["value"]
        for h in payload.get("headers", [])
    }

    body = _extract_body(payload)
    attachments = _extract_attachments(payload)

    return {
        "id": msg_id,
        "from": headers.get("From", ""),
        "to": headers.get("To", ""),
        "subject": headers.get("Subject", ""),
        "date": headers.get("Date", ""),
        "body": body,
        "attachments": attachments,
    }


def send_email(to: str, subject: str, body: str) -> dict:
    """Send an email.

    Returns dict with id and status.
    """
    service = get_gmail_service()
    message = MIMEText(body)
    message["to"] = to
    message["subject"] = subject
    raw = base64.urlsafe_b64encode(
        message.as_bytes()
    ).decode("utf-8")

    sent = service.users().messages().send(
        userId="me",
        body={"raw": raw},
    ).execute()

    return {"id": sent["id"], "status": "sent"}


def search_emails(
    query: str, max_results: int = 10
) -> list[dict]:
    """Search emails using Gmail query syntax.

    Returns list of email summaries.
    """
    service = get_gmail_service()
    resp = service.users().messages().list(
        userId="me",
        q=query,
        maxResults=max_results,
    ).execute()

    messages = resp.get("messages", [])
    results = []
    for msg in messages:
        meta = service.users().messages().get(
            userId="me",
            id=msg["id"],
            format="metadata",
            metadataHeaders=["From", "Subject", "Date"],
        ).execute()
        headers = {
            h["name"]: h["value"]
            for h in meta.get("payload", {}).get("headers", [])
        }
        results.append({
            "id": msg["id"],
            "from": headers.get("From", ""),
            "subject": headers.get("Subject", ""),
            "date": headers.get("Date", ""),
            "snippet": meta.get("snippet", ""),
        })
    return results


def get_labels() -> list[str]:
    """List all Gmail labels."""
    service = get_gmail_service()
    resp = service.users().labels().list(userId="me").execute()
    return [lbl["name"] for lbl in resp.get("labels", [])]


# ---------------------------------------------------------------
# Helpers

def _extract_body(payload: dict) -> str:
    """Recursively extract plain text body from message payload."""
    mime_type = payload.get("mimeType", "")
    if mime_type == "text/plain":
        data = payload.get("body", {}).get("data", "")
        if data:
            return base64.urlsafe_b64decode(
                data + "=="
            ).decode("utf-8", errors="replace")

    for part in payload.get("parts", []):
        result = _extract_body(part)
        if result:
            return result

    # Fallback: try HTML if no plain text
    if mime_type == "text/html":
        data = payload.get("body", {}).get("data", "")
        if data:
            html = base64.urlsafe_b64decode(
                data + "=="
            ).decode("utf-8", errors="replace")
            # Strip basic HTML tags
            import re
            return re.sub(r"<[^>]+>", " ", html)

    return ""


def _extract_attachments(payload: dict) -> list[dict]:
    """Extract attachment metadata from message payload."""
    attachments = []
    for part in payload.get("parts", []):
        if part.get("filename"):
            attachments.append({
                "filename": part["filename"],
                "mimeType": part.get("mimeType", ""),
                "size": part.get("body", {}).get("size", 0),
            })
        attachments.extend(_extract_attachments(part))
    return attachments

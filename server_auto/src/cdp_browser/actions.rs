/// High-level browser actions via CDP.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use super::client::{close_tab, open_session};

// ---------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Search Google and return top results.
pub async fn web_search(query: &str) -> Result<Vec<SearchResult>> {
    let encoded = urlencoding::encode(query);
    let search_url = format!("https://www.google.com/search?q={encoded}");
    let (tab_id, mut session) = open_session(&search_url).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let script = r#"
    (function() {
        var results = [];
        var headings = document.querySelectorAll('h3');
        for (var i = 0; i < headings.length && results.length < 10; i++) {
            var h = headings[i];
            var a = h.closest('a') || h.querySelector('a') ||
                    (h.parentElement && h.parentElement.closest('a'));
            if (!a || !a.href || a.href.startsWith('https://www.google')) continue;
            var container = h.closest('[data-sokoban-container]') ||
                            h.closest('.MjjYud') || h.parentElement;
            var snippetEl = container ? (
                container.querySelector('[data-sncf="1"]') ||
                container.querySelector('.VwiC3b') ||
                container.querySelector('span[style]')
            ) : null;
            results.push({
                title: h.textContent.trim(),
                url: a.href,
                snippet: snippetEl ? snippetEl.textContent.trim() : ''
            });
        }
        return JSON.stringify(results);
    })()"#;

    let raw = session.evaluate(script).await?;
    let _ = close_tab(&tab_id).await;
    let results: Vec<SearchResult> = serde_json::from_str(raw.as_str().unwrap_or("[]"))
        .unwrap_or_default();
    Ok(results.into_iter().filter(|r| !r.title.is_empty()).collect())
}

/// Fetch a URL and return its main text content.
pub async fn web_fetch(url: &str, max_chars: usize) -> Result<String> {
    let (tab_id, mut session) = open_session(url).await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let script = r#"
    (function() {
        var el = document.querySelector('article') ||
                 document.querySelector('main') ||
                 document.querySelector('[role="main"]') ||
                 document.querySelector('.content') ||
                 document.querySelector('#content') ||
                 document.body;
        if (!el) return document.body.innerText;
        var noisy = el.querySelectorAll(
            'nav, footer, header, aside, .nav, .footer, .header, .sidebar, .advertisement, .ad, script, style'
        );
        noisy.forEach(function(n) { n.remove(); });
        return el.innerText || el.textContent || '';
    })()"#;

    let raw = session.evaluate(script).await?;
    let _ = close_tab(&tab_id).await;
    let mut text = raw.as_str().unwrap_or("").trim().to_string();
    if text.len() > max_chars {
        text.truncate(max_chars);
        text.push_str("\n...[truncated]");
    }
    Ok(text)
}

/// Take a screenshot of a URL and return PNG bytes.
pub async fn web_screenshot(url: &str) -> Result<Vec<u8>> {
    let (tab_id, mut session) = open_session(url).await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let png = session.screenshot().await?;
    let _ = close_tab(&tab_id).await;
    Ok(png)
}

// ---------------------------------------------------------------
// LinkedIn

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedInPost {
    pub author: String,
    pub text: String,
    pub likes: String,
}

pub async fn linkedin_feed() -> Result<Vec<LinkedInPost>> {
    let (tab_id, mut session) = open_session("https://www.linkedin.com/feed/").await?;
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    session.evaluate("window.scrollBy(0, 800)").await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let script = r#"
    (function() {
        var posts = [];
        var items = document.querySelectorAll('.feed-shared-update-v2');
        for (var i = 0; i < items.length; i++) {
            var el = items[i];
            var authorEl = el.querySelector('.update-components-actor__name') ||
                           el.querySelector('.update-components-actor__title') ||
                           el.querySelector('[class*="actor__name"]');
            var textEl = el.querySelector('.feed-shared-text') ||
                         el.querySelector('.update-components-text') ||
                         el.querySelector('.attributed-text-segment-list__content');
            var likesEl = el.querySelector('.social-details-social-counts');
            if (authorEl) {
                posts.push({
                    author: (function(t) {
                        // Remove duplicate names (e.g. "Jim TarantinoJim Tarantino")
                        t = t.trim().split('\n')[0].trim();
                        var half = Math.floor(t.length / 2);
                        if (half > 2 && t.slice(0, half) === t.slice(half)) t = t.slice(0, half);
                        return t;
                    })(authorEl.textContent),
                    text: textEl ? textEl.textContent.trim() : '',
                    likes: likesEl ? likesEl.textContent.trim() : ''
                });
            }
        }
        return JSON.stringify(posts);
    })()"#;

    let raw = session.evaluate(script).await?;
    let _ = close_tab(&tab_id).await;
    let posts: Vec<LinkedInPost> = serde_json::from_str(raw.as_str().unwrap_or("[]"))
        .unwrap_or_default();
    Ok(posts.into_iter().filter(|p| !p.author.is_empty()).collect())
}

pub async fn linkedin_like(post_index: usize, dry_run: bool) -> Result<Value> {
    let (tab_id, mut session) = open_session("https://www.linkedin.com/feed/").await?;
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    let info_script = format!(r#"
    (function() {{
        var items = document.querySelectorAll('.feed-shared-update-v2');
        var el = items[{post_index}];
        if (!el) return JSON.stringify({{error: 'post not found'}});
        var authorEl = el.querySelector('.update-components-actor__name') ||
                       el.querySelector('.update-components-actor__title') ||
                       el.querySelector('[class*="actor__name"]');
        var textEl = el.querySelector('.feed-shared-text') ||
                     el.querySelector('.update-components-text') ||
                     el.querySelector('.attributed-text-segment-list__content');
        var likeBtn = el.querySelector('button[aria-label*="Like"]') || el.querySelector('.reactions-react-button');
        var authorRaw = authorEl ? authorEl.textContent.trim().split('\n')[0].trim() : '';
        var half = Math.floor(authorRaw.length / 2);
        if (half > 2 && authorRaw.slice(0, half) === authorRaw.slice(half)) authorRaw = authorRaw.slice(0, half);
        return JSON.stringify({{
            author: authorRaw,
            text: textEl ? textEl.textContent.trim().slice(0, 200) : '',
            like_button_found: likeBtn !== null
        }});
    }})()"#);

    let raw = session.evaluate(&info_script).await?;
    let mut info: Value = serde_json::from_str(raw.as_str().unwrap_or("{}")).unwrap_or_default();

    if dry_run {
        info["dry_run"] = json!(true);
        info["action"] = json!("not liked (dry run)");
        let _ = close_tab(&tab_id).await;
        return Ok(info);
    }

    let click_script = format!(r#"
    (function() {{
        var items = document.querySelectorAll('.feed-shared-update-v2');
        var el = items[{post_index}];
        if (!el) return 'post not found';
        var btn = el.querySelector('button[aria-label*="Like"]') || el.querySelector('.reactions-react-button');
        if (!btn) return 'like button not found';
        btn.click();
        return 'clicked';
    }})()"#);

    let result = session.evaluate(&click_script).await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    info["action"] = result;
    let _ = close_tab(&tab_id).await;
    Ok(info)
}

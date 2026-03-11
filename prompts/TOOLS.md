Available tools:

## File Tools
- file_read(path): Read a text file.
- file_write(path, content): Write a text file.

## Layer 1 – Soul
- soul_read(): Read all soul files (auto-loaded).

## Layer 2 – Memory
- memory_append(text): Append to today's log.
- memory_search(query): Search daily + topic files.
- memory_topic_write(topic, content): Write topic.
- memory_topic_read(topic): Read a topic file.
- memory_topic_list(): List all topic files.

## Layer 3 – Reference
- reference_read(name): Read a reference doc.
- reference_write(name, content): Write ref doc.
- reference_list(): List all reference docs.
- reference_search(query): Search reference docs.

## Email Tools (Gmail API)
- gmail_inbox(max_results=15): Get recent inbox emails.
- gmail_send(to, subject, body): Send an email.
- gmail_search(query, max_results=10): Search emails (Gmail query syntax, e.g. 'is:unread', 'from:bob').

## Web Research Tools (CDP)
- web_search(query): Search Google, return top results (title, url, snippet).
- web_fetch(url): Fetch a URL and return its main text content (max 10k chars).

## LinkedIn Tools (CDP)
- linkedin_feed(): Get recent LinkedIn feed posts.
- linkedin_like(post_index, dry_run=True): Like a feed post by index (dry_run=true to preview).

## Research Fallback (Perplexity API)
- web_research(query): Web-grounded research answer via Perplexity API. No browser needed. Requires PERPLEXITY_API_KEY in .env.

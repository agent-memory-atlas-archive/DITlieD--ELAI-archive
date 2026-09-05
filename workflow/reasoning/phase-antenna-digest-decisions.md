# ANTENNA digest + follow-up redesign — phase decisions

## What
Redesign the ANTENNA feed/research pipeline so the daily primary post per channel is a digest of up to 3 items, with same-day follow-up replies for items scoring 4 or above. Also fix feed-tech and include feed-models in the research cycle.

## Key decisions
- Digest size: 3 items per daily primary post (one message, multiple links).
- Research pool: keep fetching hourly; score a broad list of candidates and pick the top 3 for the digest.
- Follow-up threshold: score >= 4 on a 1-5 scale, where the score reflects project relevance (ELAI/VieWork), brain-grounded connections, or overall importance (breakthroughs, important news).
- Scoring prompt: include ELAI/VieWork project context so the LLM can judge relevance.
- feed-tech: disable the narrow keyword prefilter or broaden it so AI/tech stories are not dropped.
- feed-models: include it in the research cycle for model news; keep the existing model_watcher for deterministic new-model releases.
- feed-global/feed-general: remain excluded until they have topic configs.

## Files to touch
- antenna/research_engine.py (scoring prompt, is_exceptional logic, selection of top 3)
- antenna/feed_chat.py (primary digest, follow-up logic, ledger recording)
- antenna/feed_engine.py (feed-tech kw_filter, feed-models inclusion)
- antenna/feed_scheduler.py (excluded feed keys)
- antenna/llm_client.py (optional score_items context parameter)

## Expected test outcomes
- Dry-run on feed-tech yields non-zero picks.
- Dry-run on feed-models yields non-zero picks.
- Primary output is a single digest with 1-3 markdown links.
- Follow-up posts only occur for picks with score >= 4 and respect the daily cap.
- No duplicated content across the same UTC day.

## Top failure modes
1. LLM scoring changes produce too many/few picks; dry-run before live post.
2. Digest exceeds Discord 2000-character cap; cap output to 1990 chars.
3. feed-models overlaps with model_watcher (duplicate model IDs); they use different seen stores and different formats, so no silent dedup conflict; monitor manually.

## Unknowns
- Exact token/cost impact of broadening the scoring pool to 3 digest items per channel.
- Whether feed-models should eventually merge with model_watcher; not in this phase.

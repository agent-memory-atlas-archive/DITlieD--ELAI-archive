Folder ownership (HARD RULE)

Loaded by: any task that writes inside .projects/<app-folder>/.

If the iOS gig is a multi-dev codebase with per-developer folders (e.g. Sources/Alice/, Sources/Bob/), NEVER write inside another dev's folder. Default to a shared folder (Sources/Shared/, Sources/_Main/, or whatever the team uses) and ask before writing into someone else's space.

Reading is fine. Writing is not.

Names per app live in workflow/memory-bank/apps/{current_app}/projectBrief.md under the team section, or in the studio/teamRoster.md if cross-app.

This rule applies to .swift files, .xib / .storyboard files, .xcassets, Info.plist fragments, and any other authoring artifact in those subtrees.

Edge case: if maintainer has his own personal folder under Sources/maintainer/ on the current app, that folder is HIS — main LLM may write there if maintainer explicitly asks. Default still goes to the shared folder.

Solo-contractor case: if maintainer is the only dev on the gig (no per-dev folders), this rule is informational only — write anywhere within .projects/<app>/. Per-feature module organization defined in coding-standards.md / file-discipline.md.

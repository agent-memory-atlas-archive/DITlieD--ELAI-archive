# Cascade Workflow Integration Plan

**Objective:** Make Cascade read and use the existing workflow files from F:\.ELAI_workflow\

**Cascade's File Structure (Discovered):**
- **Rules:** `C:\Users\user\.codeium\windsurf\memories\global_rules.md` ← Already has your CLAUDE.md content!
- **Workflows:** `C:\Users\user\.codeium\windsurf\windsurf\workflows\` ← Has review.md
- **Cascade data:** `C:\Users\user\.codeium\windsurf\cascade\` (.pb files)
- **MCP config:** `c:\Users\user\.codeium\windsurf\mcp_config.json`

**Your Existing Files (Reuse These):**
- F:\.ELAI_workflow\CLAUDE.md
- F:\.ELAI_workflow\.claude\agents\
- F:\.ELAI_workflow\.claude\rules\
- F:\.ELAI_workflow\.claude\skills\
- F:\.ELAI_workflow\workflow\memory-bank\
- C:\Users\user\.claude\rag\ (RAG server)

**The Core Question:** Where do we put Cascade-specific workflow files so Cascade can read them?

---

## Solution: File Mapping

**Where Cascade reads files:**

| File Type | Cascade Location | Your Existing Files | Action |
|-----------|------------------|---------------------|--------|
| **Rules** | `C:\Users\user\.codeium\windsurf\memories\global_rules.md` | `F:\.ELAI_workflow\CLAUDE.md` | Update global_rules.md for Cascade |
| **Workflows** | `C:\Users\user\.codeium\windsurf\windsurf\workflows\*.md` | `F:\.ELAI_workflow\.claude\skills\*/*.md` | Copy/adapt skills to workflows folder |
| **Memory-bank** | Keep in `F:\.ELAI_workflow\workflow\memory-bank\` | Already there | Cascade reads via read_file |
| **RAG Server** | Add to MCP config | `C:\Users\user\.claude\rag\` | Add to mcp_config.json |

---

## Step 1: Update global_rules.md for Cascade

**File:** `C:\Users\user\.codeium\windsurf\memories\global_rules.md`

**Current Status:** Already contains your CLAUDE.md content (copied over)

**Changes Needed:**
1. Replace `mcp_think` with `mcp4_sequentialthinking` (Cascade's reasoning tool)
2. Remove agent spawning references (Cascade can't spawn sub-agents)
3. Adapt tool names (read_file instead of Read, etc.)
4. Keep all core philosophy (grounded-by-default, GRC, FSM)

**Key Edits:**
- Line 5-6: Replace mcp_think with mcp4_sequentialthinking
- Line 15-16: Remove context delegation and agent spawning rules
- Line 17-21: Simplify hook enforcement (manual checks instead of hooks)
- Line 59-65: Adapt skills section to reference workflows folder

---

## Step 2: Copy Skills to Workflows Folder

**Source:** `F:\.ELAI_workflow\.claude\skills\`
**Destination:** `C:\Users\user\.codeium\windsurf\windsurf\workflows\`

**Files to Copy:**
1. `planning-workflow/SKILL.md` → `workflows/planning-workflow.md`
2. `prompt-transform/SKILL.md` → `workflows/prompt-transform.md`
3. `rag-tools/SKILL.md` → `workflows/rag-tools.md`

**Adaptations Needed:**
1. Remove agent spawning references
2. Replace tool names (Read → read_file, etc.)
3. Keep MCP tool calls (they work the same)
4. Follow the review.md format (YAML frontmatter with description)

---

## Step 3: Add RAG Server to MCP Config

**File:** `c:\Users\user\.codeium\windsurf\mcp_config.json`

**Add this entry:**

```json
"local-rag": {
  "command": "python",
  "args": ["-m", "server"],
  "cwd": "C:\\Users\\user\\.claude\\rag",
  "env": {
    "PYTHONPATH": "C:\\Users\\user\\.claude\\rag"
  },
  "disabled": false
}
```

**What This Provides:**
- mcp_search_context, mcp_memorize, mcp_ingest_file
- mcp_update_context, mcp_log_change, mcp_report_build
- mcp_fsm_get_state, mcp_fsm_transition (FSM state machine)

---

## Step 4: Memory-Bank Integration (No Changes Needed)

**Location:** Keep in `F:\.ELAI_workflow\workflow\memory-bank\`

**How Cascade Uses Them:**
- Cascade reads files via `read_file` tool
- RAG server tools update files automatically:
  - mcp_update_context → activeContext.md
  - mcp_log_change → changelog.md
  - mcp_report_build → progress.md

**No file movement needed** - Cascade can read from F:\.ELAI_workflow\

---

## Implementation Steps (In Order)

### Step 1: Add RAG MCP Server (5 min)
```bash
# Edit c:\Users\user\.codeium\windsurf\mcp_config.json
# Add local-rag entry
# Restart Windsurf
```

### Step 2: Update global_rules.md (15 min)
```bash
# Edit C:\Users\user\.codeium\windsurf\memories\global_rules.md
# Replace mcp_think with mcp4_sequentialthinking
# Remove agent spawning references
# Adapt tool names
```

### Step 3: Copy Skills to Workflows (20 min)
```bash
# Copy F:\.ELAI_workflow\.claude\skills\planning-workflow/SKILL.md
#   to C:\Users\user\.codeium\windsurf\windsurf\workflows\planning-workflow.md
# Adapt content (remove agents, change tool names)
# Repeat for prompt-transform and rag-tools
```

### Step 4: Test Integration (15 min)
```bash
# Restart Windsurf
# Check if local-rag tools appear
# Call mcp_fsm_get_state to verify FSM works
# Read a memory-bank file to verify access
```

**Total Time:** ~55 minutes

---

## How Cascade Reads These Files

**Rules Loading:**
- Cascade automatically loads `C:\Users\user\.codeium\windsurf\memories\global_rules.md` at session start
- This is equivalent to CLAUDE.md in Claude Code

**Workflow Loading:**
- Cascade loads workflows from `C:\Users\user\.codeium\windsurf\windsurf\workflows\*.md`
- When a trigger matches (e.g., "implement", "plan"), Cascade loads the relevant workflow
- This is equivalent to .claude/skills/ in Claude Code

**Memory-Bank Access:**
- Cascade uses `read_file` tool to read from `F:\.ELAI_workflow\workflow\memory-bank\`
- No automatic loading - explicit reads when needed
- RAG server tools handle writes automatically

**MCP Tools:**
- Once added to mcp_config.json, tools are available in every Cascade session
- No additional configuration needed

---

## Summary

**Where files go:**
- **Rules:** `C:\Users\user\.codeium\windsurf\memories\global_rules.md` (update existing)
- **Workflows:** `C:\Users\user\.codeium\windsurf\windsurf\workflows\` (copy from .claude/skills/)
- **Memory-bank:** `F:\.ELAI_workflow\workflow\memory-bank\` (keep as-is)
- **RAG Server:** Add to `c:\Users\user\.codeium\windsurf\mcp_config.json`

**How Cascade reads them:**
- Rules: Auto-loaded at session start
- Workflows: Auto-loaded when triggers match
- Memory-bank: Explicit read_file calls
- MCP tools: Available after adding to config

**Next Action:** Start with Step 1 - Add RAG MCP server to mcp_config.json

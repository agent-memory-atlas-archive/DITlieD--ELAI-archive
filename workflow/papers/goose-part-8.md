# Goose CLI Map (Block's open-source Rust agent)
**Partition**: crates/goose-cli (50 files, 22,582 LOC)
**Explorer**: partition-8 repo mapper

---

## 1. COMMAND SURFACE

### Top-level commands (cli.rs:781, clap enum Command)

| Command | Purpose | Signature |
|---------|---------|-----------|
| `configure` | Setup provider/API keys/extensions | `goose configure` (interactive wizard) |
| `info` | Show goose config & status | `goose info [--verbose] [--check]` |
| `doctor` | Health check for goose setup | `goose doctor` |
| `session` (alias: `s`) | Create/manage/list chat sessions | `goose s [--resume] [--fork] [--name <NAME>]` |
| `run` | Execute instruction file or stdin | `goose run [--input <FILE>] [--format text\|json\|stream-json]` |
| `project` (alias: `p`) | Open last project directory | `goose p` |
| `projects` (alias: `ps`) | List recent project directories | `goose ps` |
| `recipe` | Recipe utilities (validate, deeplink) | `goose recipe [list\|open\|validate\|explain]` |
| `skills` | Skill utilities | `goose skills list` |
| `plugin` | Manage plugins | `goose plugin [install\|update]` |
| `schedule` (alias: `sched`) | Manage scheduled jobs (cron) | `goose schedule [add\|list\|remove\|run-now]` |
| `gateway` (alias: `gw`) | External platform integrations (Telegram) | `goose gw [...]` |
| `mcp` | Run bundled MCP servers | `goose mcp [server-name]` |
| `acp` | Run as ACP agent (stdio) | `goose acp [--with-builtin NAME]` |
| `serve` | HTTP+WebSocket ACP server | `goose serve [--host 127.0.0.1] [--port 3284]` |
| `term` | Terminal-integrated session | `goose term [init\|run\|log]` (per-terminal persistent) |
| `tui` | Terminal UI (if feature enabled) | `goose tui` |
| `update` | Update CLI version | `goose update [--canary] [--reconfigure]` |

### Session subcommands (SessionCommand enum, cli.rs:487)

| Subcommand | Purpose |
|------------|---------|
| `list` | List sessions (filters: `--working_dir`, `--limit`, `--ascending` sort) |
| `remove` | Delete sessions (interactive, by ID, name, or regex) |
| `export` | Export to markdown/json/yaml; Nostr publish via `--nostr --relay` |
| `import` | Import from Claude Code .jsonl, goose JSON export, or encrypted Nostr link |
| `diagnostics` | Generate session zip with debug info |

### Schedule subcommands (SchedulerCommand enum, cli.rs:587)

| Subcommand | Purpose |
|------------|---------|
| `add` | Add cron job: `--schedule-id ID --cron "0 * * * *" --recipe-source <path>` |
| `list` | List all scheduled jobs |
| `remove` | Delete job by ID |
| `run-now` | Trigger a scheduled job immediately |
| `services status` | Check scheduler daemon status |
| `services stop` | Stop scheduler daemon |
| `sessions` | List sessions created by scheduler |

---

## 2. INTERACTIVE SESSION UX

### Prompt Rendering (input.rs:153)

**Prompt string**: `"> "` (simple greater-than with space)

```rust
// input.rs:153
let input = match editor.readline("> ") {
    Ok(text) => text,
    Err(e) => match e {
        rustyline::error::ReadlineError::Interrupted => return Ok(InputResult::Exit),
        rustyline::error::ReadlineError::Eof => return Ok(InputResult::Exit),
        ...
    }
}
```

- **Editor**: `rustyline` (Rust readline clone)
- **Newline key**: Customizable via `GOOSE_CLI_NEWLINE_KEY` env var; default `Ctrl+J` (input.rs:86-93)
- **Completion**: GooseCompleter struct provides hint-based completion (completion.rs)
- **Ctrl+C**: Clear current line if text entered; double-tap to exit (input.rs:47-84 CtrlCHandler)
- **History**: Per-session, saved on exit (input.rs:163-165)

### Streaming Output Buffer (streaming_buffer.rs, 828 LOC)

Handles real-time markdown-formatted streaming responses via `MarkdownBuffer`. Renders syntax highlighting with bat (syntax highlighter).

### Token/Cost Display (output.rs:1367-1445)

**Context usage display** (`display_context_usage`):
- **Format**: `[green━━━━╌╌╌╌] 45% 1.2k/3.0k` (bar + percentage + token count)
- **Bar colors**: Green (<50%), Yellow (50-85%), Red (≥85%)
- **Token formatting**: Suffixes M (>1M), k (>1k), plain (output.rs:1394-1402)
- **Where**: Called after each API response (session/mod.rs output flow)

**Cost display** (`display_cost_usage`):
```rust
// output.rs:1434-1445
pub fn display_cost_usage(provider: &str, model: &str, input_tokens: usize, output_tokens: usize) {
    if let Some(cost) = estimate_cost_usd(provider, model, input_tokens, output_tokens) {
        eprintln!(
            "Cost: {} USD ({} tokens: in {}, out {})",
            style(format!("${:.4}", cost)).cyan(),
            input_tokens + output_tokens,
            input_tokens,
            output_tokens
        );
    }
}
```
- **Format**: `Cost: $0.0032 USD (1245 tokens: in 500, out 745)`
- **Pricing**: Pulled from canonical model metadata (`goose::providers::canonical::maybe_get_canonical_model`)
- **Output**: stderr via `eprintln!` (non-blocking, appears after response)

### Session Metadata (mod.rs:59-96)

Session output includes JSON envelope with token tracking:

```rust
#[derive(Serialize, Deserialize, Debug)]
struct JsonMetadata {
    total_tokens: Option<i32>,
    input_tokens: Option<i32>,
    output_tokens: Option<i32>,
    status: String,
}

#[derive(Serialize, Debug)]
enum StreamEvent {
    Message { message: Message },
    Notification { extension_id: String, data: NotificationData },
    Error { error: String },
    Complete {
        total_tokens: Option<i32>,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
    },
}
```
- Tokens tracked per turn, aggregated in final `Complete` event.

### Slash Commands (input.rs:190-316)

REPL supports these meta-commands:

| Command | Effect |
|---------|--------|
| `/exit`, `/quit` | Exit session |
| `/?`, `/help` | Print help + editor help |
| `/t` | Toggle theme (light/dark/ansi) |
| `/t <theme>` | Switch to theme |
| `/prompts [--extension NAME]` | List extension prompts |
| `/prompt <name> [--info] [key=val...]` | Invoke prompt with args |
| `/extension <name>` | Add extension |
| `/builtin <name>` | Add builtin extension |
| `/mode <mode>` | Switch goose mode |
| `/model` or `/model <name>` | Change model |
| `/plan <text>` | Start planning mode |
| `/endplan` | Exit planning mode |
| `/clear` | Clear conversation history |
| `/recipe [path.yaml]` | Load recipe |
| `/compact` | Compact session (reduce context) |
| `/summarize` | Deprecated (renamed to `/compact`) |
| `/r` | Toggle full tool output display |
| `/edit [text]` | Open external editor to compose message |
| `/skills [list\|load <names>]` | Manage skills |

---

## 3. PROVIDER/MODEL CONFIGURATION FLOW

### Entry point: configure_provider_dialog (configure.rs:675)

**Step 1: Provider selection** (configure.rs:696-700)
```rust
let provider_name = cliclack::select("Which model provider should we use?")
    .initial_value(&default_provider)
    .items(&provider_items)
    .filter_mode()  // Searchable dropdown
    .interact()?;
```
- Fetches all providers via `providers().await` (configure.rs:680)
- Sorted alphabetically by display name
- Shows: name, display_name, description

**Step 2: Configure primary keys** (configure.rs:708-716)
For each provider's primary or OAuth config key:
```rust
for key in provider_meta.config_keys.iter().filter(|k| k.primary || k.oauth_flow) {
    configure_single_key(config, provider_name, &provider_meta.display_name, key).await?
}
```
- Prompts for API key entry (possibly OAuth flow)
- Keys stored in config (varies by provider: env var, config file, secure storage)

**Step 3: Optional advanced settings** (configure.rs:723-735)
```rust
if !non_primary_keys.is_empty()
    && cliclack::confirm("Would you like to configure advanced settings?")
        .initial_value(false)
        .interact()?
{
    // Configure optional keys
}
```

**Step 4: Fetch supported models** (configure.rs:737-746)
```rust
let spin = spinner();
spin.start("Attempting to fetch supported models...");
let temp_provider = create(provider_name, temp_model_config, Vec::new()).await?;
let models_res = retry_operation(&RetryConfig::default(), || async {
    temp_provider.fetch_recommended_models().await
}).await;
```
- Creates temp provider instance
- Calls `fetch_recommended_models()` with retry (configure.rs:742-744)
- Spinner shows progress

**Step 5: Select model** (configure.rs:748-763)
- **If models available**: Show dropdown via `select_model_from_list` (configure.rs:755)
- **If empty list**: Free-text input with default from `GOOSE_MODEL` env var (configure.rs:759-761)
- **If fetch failed**: Show error and abort (configure.rs:752-753)

**Step 6: Optional reasoning/thinking effort** (configure.rs:766-784)
```rust
if supports_thinking {
    let effort: &str = cliclack::select("Select thinking effort:")
        .item("off", "Off - No extended thinking", "")
        .item("low", "Low - Better latency, lighter reasoning", "")
        .item("medium", "Medium - Moderate thinking", "")
        .item("high", "High - Deep reasoning", "")
        .item("max", "Max - No constraints on thinking depth", "")
        .initial_value("off")
        .interact()?;
    config.set_goose_thinking_effort(effort)?;
}
```
- Queries `fetch_model_info(&model)` to check `.reasoning` flag (configure.rs:766)
- Fallback to `ModelConfig::new(&model).is_reasoning_model()` if unavailable (configure.rs:768-770)

**Step 7: Test configuration** (configure.rs:787-811)
```rust
let spin = spinner();
spin.start("Checking your configuration...");
let toolshim_enabled = std::env::var("GOOSE_TOOLSHIM").map(...).unwrap_or(false);
let toolshim_model = std::env::var("GOOSE_TOOLSHIM_OLLAMA_MODEL").ok();
match test_provider_configuration(provider_name, &model, toolshim_enabled, toolshim_model).await {
    Ok(()) => {
        goose::config::set_active_provider(config, provider_name, &model)?;
        print_config_file_saved()?;
        Ok(true)
    }
    Err(e) => {
        spin.stop(style(e.to_string()).red());
        cliclack::outro(style(format!("Failed to configure provider: {e}")).on_red().white())?;
        Ok(false)
    }
}
```
- Calls `test_provider_configuration()` (from `goose::providers::provider_test`)
- On success: commits config via `set_active_provider()`, prints success
- On failure: shows error in red, aborts

### Custom providers (configure_custom_provider_dialog, configure.rs:2108)
Alternative: Create custom provider with `CreateCustomProviderParams` (declarative_providers.rs).

---

## 4. DIAGNOSTIC & STATUS COMMANDS

### `goose info` (commands/info.rs)

Displays:
- Config file location & status
- Active provider & model
- Session storage path
- Extensions enabled
- **Provider health check** (if `--check`): Attempts test API call, shows latency & success/failure

### `goose doctor` (commands/doctor.rs)

Full system health check (runs automatically on first setup if config missing).

### Session diagnostics (cli.rs:574-583, session.rs)

`goose session diagnostics [--session-id ID] [-o <output.zip>]` generates zip with:
- Session metadata
- Message log
- Extension configs
- Extension call history

### Term prompt integration (cli.rs:1236)

`goose term info` prints compact JSON for shell prompt:
```
{"model": "claude-3-5-sonnet", "tokens": 1234, "status": "ready"}
```
Used to integrate goose stats into shell PROMPT_COMMAND.

---

## 5. PORTABLE MECHANISMS & REIMPLEMENTATION IDEAS

### A. Rustyline-based REPL with custom keybindings (input.rs:86-160)

**Reuse**: Generic prompt loop with:
- **Custom newline key**: Configurable via env var (default Ctrl+J)
- **Ctrl+C handler**: Double-tap to exit, single-tap to clear line
- **Completion hints**: Pluggable via `rustyline::Hint` trait
- **History management**: Automatic, per-session

**Files**: `input.rs`, `completion.rs`

### B. Streaming markdown buffer with syntax highlighting (streaming_buffer.rs:828 LOC, output.rs)

**Reuse**: Real-time chunk aggregation with bat-powered code coloring:
- Accumulates partial markdown
- Detects code block boundaries
- Syntax-highlights on flush
- Handles terminal width wrapping

**Portable pattern**: Abstract `StreamBuffer` trait with:
- `add_chunk(String)` → accumulate
- `flush() → String` → render & clear

**Files**: `streaming_buffer.rs`, `output.rs`

### C. Cost estimation from canonical provider metadata (output.rs:1417-1430)

**Reuse**: Centralized pricing lookup:
```rust
fn estimate_cost_usd(provider: &str, model: &str, input_tokens: usize, output_tokens: usize) -> Option<f64> {
    let canonical_model = maybe_get_canonical_model(provider, model)?;
    let input_cost_per_token = canonical_model.cost.input? / 1_000_000.0;
    let output_cost_per_token = canonical_model.cost.output? / 1_000_000.0;
    let input_cost = input_cost_per_token * input_tokens as f64;
    let output_cost = output_cost_per_token * output_tokens as f64;
    Some(input_cost + output_cost)
}
```
- **Input**: provider name, model name, token counts
- **Output**: USD cost (None if pricing unavailable)
- **Cacheable**: Pulls from canonical model registry (updated per release)

**Files**: `output.rs:1417`

### D. Interactive provider configuration wizard (configure.rs:675-812)

**Pattern**: Multi-step async dialog:
1. **Provider selection** with filtering (cliclack::select)
2. **Key entry** (OAuth or manual, per provider)
3. **Model fetch & selection** with retry logic
4. **Feature negotiation** (reasoning effort for supported models)
5. **Validation** via test API call
6. **Atomic commit** to config on success

**Reusable component**: `ProviderConfigWizard` trait:
- `select_provider()` → select from registry
- `configure_keys()` → per-provider key entry
- `fetch_models()` → list or free-text fallback
- `test_and_save()` → validate & persist

**Files**: `configure.rs:675-812`

### E. Shell integration setup (term.rs:40-150+)

**Reuse**: Per-shell init script generation with:
- **Bash/Zsh**: `preexec` hook to log commands
- **Fish**: Custom `fish_postexec` function
- **Nu**: Pure Nu script
- **PowerShell**: PS object pipeline

**Template system** (term.rs:23-38):
```rust
struct ShellConfig {
    script_template: &'static str,
    command_not_found: Option<&'static str>,
}
impl Shell {
    fn config(&self) -> &'static ShellConfig { ... }
}
```
- Substitutes `{session_id}`, `{goose_bin}` at runtime
- Per-shell command_not_found handler optional

**Portable**: Extract as config-driven template engine.

**Files**: `term.rs:40-150`

### F. Session import/export with multi-format support (session.rs, commands/session.rs)

**Formats supported**:
- Goose JSON export
- Claude Code .jsonl (line-delimited messages)
- Encrypted Nostr events (with relay publish)

**Reusable**: Abstract exporter:
```rust
trait SessionExporter {
    fn export(&self, session: &Session) -> Result<String>;
}
impl SessionExporter for MarkdownExporter { ... }
impl SessionExporter for JsonExporter { ... }
impl SessionExporter for NostrExporter { ... }
```

**Files**: `commands/session.rs` (export/import handlers), `session/export.rs` (serialization)

### G. Token formatting with human-readable suffixes (output.rs:1394-1402)

**Reuse**: Lossy compression for large numbers:
```rust
fn format_tokens(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
```
- **Pattern**: Thresholds (1M, 1k), precision (1 decimal for M, 0 for k)
- **Portable**: Generic over any large-integer metric (tokens, bytes, etc.)

**Files**: `output.rs:1394`

### H. Context usage visualization (output.rs:1367-1415)

**Reuse**: Terminal bar chart with color-coding:
```rust
pub fn display_context_usage(total_tokens: usize, context_limit: usize) {
    let percentage = (((total_tokens as f64 / context_limit as f64) * 100.0).round() as usize).min(100);
    let bar_width = 20;
    let filled = ((percentage as f64 / 100.0) * bar_width as f64).round() as usize;
    let bar = format!("{}{}", "━".repeat(filled), "╌".repeat(empty));
    let colored_bar = if percentage < 50 { style(bar).green().dim() }
                      else if percentage < 85 { style(bar).yellow() }
                      else { style(bar).red() };
    println!("  {} {} {}", colored_bar, style(format!("{}%", percentage)).dim(),
             style(format!("{}/{}", format_tokens(total_tokens), format_tokens(context_limit))).dim());
}
```
- **Colors**: Green/Yellow/Red thresholds (50%/85%)
- **Bar**: Unicode box-drawing (━ filled, ╌ empty)
- **Suffix**: Percentage + formatted token counts

**Portable**: Parameterize thresholds, colors, bar character.

**Files**: `output.rs:1367`

### I. Scheduled job cron expression parsing (commands/schedule.rs, cli.rs:587-643)

**Reuse**: Cron-to-schedule-id mapping with cron expression validation.

**Pattern**: Subcommand hierarchy for cron job management (add/list/remove/run-now).

**Files**: `commands/schedule.rs`, `cli.rs:587`

---

## EVIDENCE: TOKEN DISPLAY IN PRACTICE

**Full trace**:
1. Session sends message to provider (session/mod.rs:224: `provider.complete()`)
2. Provider returns usage metadata (input_tokens, output_tokens)
3. Session captures in `StreamEvent::Complete { total_tokens, input_tokens, output_tokens }` (mod.rs:89-95)
4. CLI outputs cost via `display_cost_usage(provider, model, input_tokens, output_tokens)` (output.rs:1434)
5. Also displays context bar via `display_context_usage(total_tokens, context_limit)` (output.rs:1367)

**JSON streaming mode** (cli.rs:297: `value_parser = PossibleValuesParser::new(["text", "json", "stream-json"])`) emits these as `StreamEvent` JSON objects per turn.

---

## REFERENCE: KEY FILE SIZES & STRUCTURE

| File | Lines | Purpose |
|------|-------|---------|
| session/mod.rs | 2449 | Main session loop, message handling, token tracking |
| cli.rs | 2292 | Clap CLI definitions, all subcommands |
| commands/configure.rs | 2140 | Provider setup wizard, key entry, model selection |
| session/output.rs | 1605 | Rendering: cost, context, markdown, syntax highlighting |
| commands/review/orchestrator.rs | 1182 | Code review multi-agent dispatch |
| session/export.rs | 1163 | Session serialization (md/json/yaml/nostr) |
| session/streaming_buffer.rs | 828 | Real-time markdown+syntax streaming |
| session/input.rs | 820 | REPL prompt, slash commands, keybinding |
| session/completion.rs | 708 | Hint-based readline completion |
| session/builder.rs | 700 | Session initialization & config building |


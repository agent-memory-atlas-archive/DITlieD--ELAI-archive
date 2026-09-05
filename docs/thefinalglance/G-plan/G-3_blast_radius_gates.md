# G-3: Deterministic runtime blast-radius gates — post-expansion FS gate + egress IP-pin

STATUS: PLANNED (WAVE 2; in-process, fail-closed; consumes G-1 live taint; composes with the shipped E-30 argv gate)

Objective: two DETERMINISTIC runtime gates that fire in the production PreToolUse dispatch BEFORE a
destructive verb executes: (a) a post-shell-EXPANSION filesystem write/delete gate that canonicalizes
~/glob/symlink to an absolute real path and FAILS CLOSED when the target escapes the per-agent worktree
root; (b) a host-side egress check that rejects hostnames carrying null bytes / control chars and confirms
the resolved IP is in the pinned allow-set. Both emit ONE `blast_radius_blocked` SSB row carrying the FULL
pre- AND post-expansion command (the rm-rf #10077 forensics gap: the EXPANDED command was never logged).
The SHIPPED E-30 gate (crates/safety/src/destructive_verb_allowlist.rs) gates the VERB on the raw,
whitespace-split, PRE-expansion source; G-3's net-new is the POST-expansion path/host + null-byte reject
that E-30's regex categorically cannot see. The gate is pure set/prefix membership — NEVER an LLM permission
classifier, which collapses to an 81% false-negative rate under adversarial scope-escalation
(arXiv:2604.04978). Estimated effort: ~1-2 sessions (~8-12 h). Pure in-process / fail-closed; no subprocess,
no RunPod at gate time (W-10 Tier-1 whole).

Pre-requisites: E-30 destructive_verb_allowlist.rs SHIPPED (grep-confirmed 2026-06-02: check_argv:157,
AllowlistVerdict:18, DENY_PATTERNS RegexSet:39, argv_blake3:138 — NOTE the module is
destructive_verb_allowlist.rs, NOT pretool_argv.rs which does not exist). The production PreToolUse path
InProcessHookDispatcher::dispatch_pre_tool_use SHIPPED (hook_dispatcher.rs:947; E-30 argv check at :959-967).
sandbox/researcher_net.rs SHIPPED (allowed_hosts:12, apply_iptables_output_allow:109, configure/try_configure
:50/:61 — Linux-cfg). G-1 taint SHIPPED-after-G-1 (provenance.rs Provenance:131, Provenanced<T>:196;
capability_gate.rs allow_tool_with_origin:84, DESTRUCTIVE_TOOLS:57).
Parallel-safe with: G-2, G-4 (disjoint files). NOT parallel-safe with any plan editing hook_dispatcher.rs
dispatch_pre_tool_use, crates/orchestrator/src/sandbox/{mod,researcher_net}.rs, or ssb.rs's write path in the
same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R3. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R3: ELAI's runtime confinement is layered but has two DETERMINISTIC blast-radius
holes. (1) The FS side: E-30's destructive_verb_allowlist.rs gates the verb by RegexSet over the raw Bash
source split on whitespace (hook_dispatcher.rs:962) — that is PRE-shell-expansion. A command like
`rm -rf "$HOME/../*"` or one routed through a symlink escapes the regex because the dangerous EXPANDED path
is never computed. The rm-rf #10077 / #12637 incidents (CONFIRMED) are exactly this: validation ran on the
unexpanded string and the expanded command was never even logged for forensics. (2) The egress side:
sandbox/researcher_net.rs pins resolved IPs via veth+iptables OUTPUT allow (researcher_net.rs:109) on Linux,
but compile_seccomp_network_extra (sandbox/mod.rs:1269) opens AF_INET WHOLESALE (socket(41) for
AF_INET/AF_INET6 x STREAM/DGRAM at :1296-1320, default EPERM at :1324) and the deployment note (mod.rs:1255-1265)
explicitly states egress filtering is NOT done at the seccomp layer. A hostname carrying a null byte /
control char can spoof the SOCKS5 path (CONFIRMED).

The fix is deterministic, in two parts, both fired in the production PreToolUse dispatch BEFORE the verb runs:
a post-expansion FS-write/delete gate that canonicalizes the target (std::fs::canonicalize resolves ~, glob,
and symlinks) and fails closed when the real path is not prefixed by the per-agent worktree root
(agent_pool.rs scratch_dir:497); and a host-side egress check that rejects any host string containing a null
byte or control char BEFORE the resolve, then confirms the resolved IP is in the pinned allow-set. Both are
pure set/prefix membership — never "ask the model if this is OK" (the 81% FNR classifier failure mode,
arXiv:2604.04978). This extends the deterministic-first posture E-30 ALREADY chose: destructive_verb_allowlist.rs:1-6
states "in-prompt instructions are NOT a safety boundary" (Replit ignored an ALL-CAPS DO-NOT-DROP 11 times,
theregister.com 2025-07-21), and the RegexSet runs at T1 before any LLM tier (hook_dispatcher.rs:954-959).

ELAI ground truth (grep+source-confirmed 2026-06-02): egress IP-pin lives in
crates/orchestrator/src/sandbox/researcher_net.rs (ResearcherNetConfig.allowed_hosts:12, default
arxiv.org/huggingface.co at :20; apply_iptables_output_allow:109 runs `iptables -A OUTPUT -d <ip> -j ACCEPT`,
default-deny REJECT at :77, veth at :65; configure/try_configure:50/:61 DNS-resolve + pin), non-Linux a no-op
(:82/:116). The syscall layer compile_seccomp_network_extra (sandbox/mod.rs:1269) opens AF_INET wholesale and
does not egress-filter (deployment note mod.rs:1255-1265). The E-30 deterministic argv allowlist SHIPPED at
crates/safety/src/destructive_verb_allowlist.rs (check_argv:157 -> AllowlistVerdict:18, DENY_PATTERNS:39,
argv_blake3:138) has NO typed SSB action_type — it surfaces only as a HookFinding rule_id "e30-argv-deny" +
a tracing trace (hook_dispatcher.rs:974), so blast_radius_blocked is the FIRST typed SSB row for a
deterministic FS/egress block. The production gate path is
crates/orchestrator/src/hook_dispatcher.rs::InProcessHookDispatcher::dispatch_pre_tool_use (:947), which runs
check_argv at :967 on BASH_TOOL_USE_ACTION, splitting raw source on whitespace at :962 (PRE-expansion only) —
G-3 hooks here. G-1 taint composition: provenance.rs Provenance:131 (TrustLevel:90 / TextSource:34, vehicle
Provenanced<T>:196) consumed by capability_gate.rs allow_tool_with_origin:84 (rejects untrusted-provenance
argv into DESTRUCTIVE_TOOLS:57). SSB mirrors: SsbActionType::CapabilityGateRejected (ssb.rs:6648, payload
CapabilityGateRejectedPayload:20117, 5-touchpoint lockstep :6648/:8886/:9649/:12899/:13722/:28478) for a
single-variant-with-hash-payload, and RouteDecisionEventKind (ssb.rs:24375, 3-arm Pattern-2) for a Kind
discriminator. Path-containment precedent: dashboard-v2/src-tauri/src/commands.rs::ensure_within_project_root
(:7386, canonicalize-parent+rejoin, rejects '..'). Per-agent worktree root = agent_pool.rs scratch_dir (:497,
scratch_base/agent_id). Landlock ABSENT (zero matches); macOS Seatbelt ABSENT; Windows
WindowsAppContainerSandbox present (mod.rs:809) but no path-allowlist/egress-pin. SCHEMA_VERSION=205
(ssb.rs:2898), TOTAL_VARIANTS=482 (crates/orchestrator/tests/action_type_lock.rs:534).

## Existing state (B/C/D/E/F coverage)
- E-30 destructive_verb_allowlist.rs (crates/safety) — SHIPPED. check_argv:157, AllowlistVerdict:18 (Allow/Deny/
  RequireHumanApproval), DENY_PATTERNS:39 (rm -rf ~|/, git reset --hard, chmod 777, DROP TABLE), argv_blake3:138.
  Gates the VERB on raw whitespace-split source (PRE-expansion). G-3 is the POST-expansion path/host complement —
  it does NOT re-implement the verb allowlist (GD-1 COMPOSES-WITH), it runs after it.
- hook_dispatcher.rs::dispatch_pre_tool_use:947 (crates/orchestrator) — SHIPPED production PreToolUse path; E-30
  argv check :959-967, F-1 VibeGate :1015, F-9 :1023. The single live chokepoint G-3 hooks (W-3).
- sandbox/researcher_net.rs (crates/orchestrator) — SHIPPED, Linux-cfg. allowed_hosts:12, apply_iptables_output_allow
  :109, configure/try_configure:50/:61. The per-spawn netns egress pin; G-3 reuses its resolve + allow-set and adds
  the null-byte host reject + the host-side check on the production write path (not only the netns boundary).
- sandbox/mod.rs::compile_seccomp_network_extra:1269 — SHIPPED; opens AF_INET wholesale (:1296-1320); G-3 does NOT
  touch seccomp (the deployment note :1255-1265 says egress is not filtered there) — it adds a host-side check above socket().
- G-1 taint (crates/safety) — provenance.rs Provenance:131 / Provenanced<T>:196; capability_gate.rs
  allow_tool_with_origin:84 / DESTRUCTIVE_TOOLS:57. G-3 composes: a block carries the joined trust label.
- ssb.rs (crates/orchestrator) — SsbActionType::CapabilityGateRejected:6648 (mirror) + RouteDecisionEventKind:24375
  (Pattern-2 Kind mirror). WAL append-only. SCHEMA 205 (:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534).
- Path-containment: commands.rs::ensure_within_project_root:7386. Worktree root: agent_pool.rs scratch_dir:497.

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R3 + GD-4 (the FP release gate)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs:940-1030 (dispatch_pre_tool_use + the E-30/F-1/F-9 fan)
- F:\.ELAI_workflow\.elai_cc\crates\safety\src\destructive_verb_allowlist.rs (full — the verb gate G-3 runs after)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\sandbox\researcher_net.rs (full — allowed_hosts + resolve + iptables pin)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\sandbox\mod.rs:1255-1340 (compile_seccomp_network_extra + the deployment note)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:7340-7610 (ensure_within_project_root canonicalize+rejoin pattern)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\agent_pool.rs:490-510 (scratch_dir worktree root)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:6648,20117,24375 (CapabilityGateRejected + RouteDecisionEventKind to mirror)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, SCHEMA baseline+N.

## Tasks (WAVE 2 — in-process, fail-closed)

1. **Add the deterministic gate module (~150 LOC new crates/safety/src/blast_radius.rs).** `pub fn
   check_fs_target(expanded_target: &Path, worktree_root: &Path) -> BlastRadiusVerdict`: reject null-byte /
   control-char paths FIRST; canonicalize the expanded target (std::fs::canonicalize resolves ~/glob/symlink;
   for not-yet-existing targets canonicalize the parent then rejoin the final component, the
   ensure_within_project_root:7386 pattern); fail closed (`Block { kind: FsEscape }`) when the real path is not
   prefixed by the canonical worktree_root. `pub fn check_egress_host(host: &str, allowed: &[IpAddr]) ->
   BlastRadiusVerdict`: reject null-byte / control-char hosts, resolve, confirm the IP is in `allowed`, else
   `Block { kind: EgressDenied }`. `pub enum BlastRadiusVerdict { Allow, Block { kind: BlastRadiusKind,
   pre_expansion: String, post_expansion: String } }`. NO `.unwrap()`/`.expect()`; an un-canonicalizable target
   is Block (fail-closed), never Allow.

2. **Wire both into dispatch_pre_tool_use (~60 LOC delta in hook_dispatcher.rs).** After the E-30 argv check
   (hook_dispatcher.rs:967) for a BASH/fs-write action, shell-EXPAND the argv (the post-expansion command), call
   check_fs_target for each write/delete target against the per-agent worktree root (agent_pool.rs scratch_dir:497),
   and check_egress_host for each fetch/connect host; on a Block verdict return `HookFinding{ verdict: Block }`
   AND emit the blast_radius_blocked SSB row. Default-ON (W-2): kill-switch `ELAI_BLAST_RADIUS_DISABLE` only;
   observable_effect runs with NO override. NO LLM-bypass branch (stub_ban): the verdict is pure set/prefix
   membership; no prompt or candidate can wave it through.

3. **Add the `blast_radius_blocked` SSB action_type (~90 LOC in ssb.rs, W-9).** Mirror the CapabilityGateRejected
   5-touchpoint recipe (ssb.rs:6648/:8886/:9649/:12899/:13722/:28478) + the RouteDecisionEventKind 3-arm pattern
   (:24375): ONE `SsbActionType::BlastRadiusBlocked` + `pub enum BlastRadiusKind { FsEscape, EgressDenied }` +
   `BlastRadiusBlockedPayload { kind, pre_expansion_command: String, post_expansion_command: String,
   target_or_host: String, worktree_root_blake3: String (blake3), trust_label: String (from the G-1 join),
   ts_ms: i64 }`. The pre- AND post-expansion command fields ARE the rm-rf #10077 forensics fix. sql_str
   "blast_radius_blocked" + parse + CHECK + round-trip golden + action_type_lock. SCHEMA bump baseline 205
   (ssb.rs:2898) -> 206; TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534) -> 483 + ALL_VARIANTS row + golden regen.

4. **Default-ON kill-switch + FP guard (GD-4) (~30 LOC).** `ELAI_BLAST_RADIUS_DISABLE=1` is a kill-switch only;
   default ON; observable_effect runs with NO override (W-2). Release gate: measure the over-block (false-FsEscape /
   false-EgressDenied) rate on >=3 real orchestrate runs; a 10-benign-flow FP-guard contract test (writes INSIDE
   the worktree + fetches to an allowed host) must pass un-blocked.

5. **Frontend pair: BlastRadiusDrawer (~180 LOC svelte + ~110 LOC tauri) (W-4).** New
   crates/dashboard-v2/src/lib/drawers/BlastRadiusDrawer.svelte listing blast_radius_blocked rows by Kind with the
   pre/post-expansion command diff (the human SEES what would have run); Tauri `get_blast_radius_blocks`.
   cross_lang_diff.py PASS. DrawerKey 'blast-radius'.

6. **Decision doc workflow/reasoning/G-3-decisions.md (~90 LOC).** Records: (a) deterministic gate beats LLM
   classifier (81% FNR, arXiv:2604.04978; the in-tree E-30 deterministic-first precedent); (b) the post-expansion +
   null-byte delta over E-30's pre-expansion regex; (c) OS coverage — Linux seccomp/veth + Windows AppContainer +
   the OS-agnostic std::fs FS gate (Landlock + macOS Seatbelt ABSENT); (d) G-3 composes with E-30 (verb) + G-1
   (trust label), re-implements neither; (e) the measured over-block rate before default-on.

## WIRING_CONTRACT
produces_symbols:
  - crates/safety/src/blast_radius.rs::check_fs_target
  - crates/safety/src/blast_radius.rs::check_egress_host
  - crates/safety/src/blast_radius.rs::BlastRadiusVerdict
  - crates/orchestrator/src/ssb.rs::SsbActionType::BlastRadiusBlocked
  - crates/orchestrator/src/ssb.rs::BlastRadiusKind
  - crates/orchestrator/src/ssb.rs::BlastRadiusBlockedPayload
  - crates/dashboard-v2/src-tauri/src/commands_g3.rs::get_blast_radius_blocks
  - crates/dashboard-v2/src/lib/drawers/BlastRadiusDrawer.svelte
consumes_symbols:
  - crates/orchestrator/src/hook_dispatcher.rs::InProcessHookDispatcher::dispatch_pre_tool_use (production gate path, :947)
  - crates/safety/src/destructive_verb_allowlist.rs::check_argv (E-30 verb gate, :157 — G-3 runs AFTER it)
  - crates/orchestrator/src/sandbox/researcher_net.rs::allowed_hosts (:12) + apply_iptables_output_allow (:109)
  - crates/orchestrator/src/agent_pool.rs (scratch_dir worktree root, :497)
  - crates/safety/src/provenance.rs::Provenanced (G-1 trust label) + capability_gate.rs::allow_tool_with_origin (:84)
  - crates/orchestrator/src/ssb.rs::SsbActionType::CapabilityGateRejected (single-variant payload mirror, :6648)
  - crates/orchestrator/src/ssb.rs::RouteDecisionEventKind (Pattern-2 Kind mirror, :24375)
reachable_from:
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use (:947, live PreToolUse) -> check_fs_target /
    check_egress_host -> HookFinding Block + blast_radius_blocked SSB row  [PRODUCTION dispatch path]
  - crates/dashboard-v2/src-tauri/src/commands_g3.rs::get_blast_radius_blocks -> BlastRadiusDrawer.svelte  [dashboard]
stub_ban:
  - NO LLM-classifier branch — the verdict is pure set/prefix membership (the 81% FNR failure mode, arXiv:2604.04978)
  - NO pre-expansion-only gating — the gate MUST canonicalize the EXPANDED target (a raw-string check is the E-30
    behaviour G-3 supersedes; shipping a pre-expansion-only G-3 is a BLOCK)
  - NO default-off ENABLE flag (only ELAI_BLAST_RADIUS_DISABLE kill-switch; default ON, W-2)
  - NO silent fail-open: an un-canonicalizable target / unresolvable host defaults to Block (fail-closed)
  - NO claim of Landlock or macOS Seatbelt usage (both ABSENT in-tree) — the FS gate is OS-agnostic std::fs
  - blake3 only (worktree_root_blake3); no blake2/sha2
  - do NOT reference pretool_argv.rs (does not exist) or a destructive_verb_denied SSB type (does not exist)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/BlastRadiusDrawer.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g3.rs::get_blast_radius_blocks
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <orchestrate a fixture whose Bash step is `rm -rf $X` where
    $X expands to a path OUTSIDE the per-agent worktree, OR a fetch to a null-byte-spoofed host; run with NO env override>"
  expect_ssb: "a blast_radius_blocked row (kind=FsEscape or EgressDenied) carrying BOTH pre_expansion_command and
    post_expansion_command + worktree_root_blake3 (blake3) in the live ssb.db"
  expect_artifact: "the verb does NOT execute (no file deleted / no connect); a benign in-worktree write + an
    allowed-host fetch in the same run are NOT blocked (FP guard); BlastRadiusDrawer shows the pre/post-expansion diff"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-3_blast_radius_gates.md --run-trigger
rg "pub fn check_fs_target|pub fn check_egress_host|pub enum BlastRadiusVerdict" .elai_cc/crates/safety/src/blast_radius.rs
rg "check_fs_target|check_egress_host" .elai_cc/crates/orchestrator/src/hook_dispatcher.rs   # wired into the live dispatch
rg "BlastRadiusBlocked|blast_radius_blocked|BlastRadiusKind" .elai_cc/crates/orchestrator/src/ssb.rs
rg "blast_radius_blocked" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg -c "ELAI_BLAST_RADIUS_ENABLE" .elai_cc/crates/safety/src/blast_radius.rs   # MUST be 0 (kill-switch DISABLE only)
rg -c "blake2|sha2" .elai_cc/crates/safety/src/blast_radius.rs               # MUST be 0
rg -ci "landlock|seatbelt" .elai_cc/crates/safety/src/blast_radius.rs        # MUST be 0 (neither exists in-tree)
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/BlastRadiusDrawer.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-3-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (produces exist, consumes resolve, reachable_from
  reached by BFS, stub_ban clean, observable_effect RAN + emitted the blast_radius_blocked row + the verb was
  blocked). cargo-green != DONE.
- [ ] W-2 default-ON: the trigger blocks the FsEscape/EgressDenied verb with NO env override;
  ELAI_BLAST_RADIUS_DISABLE=1 disables it; no default-off ENABLE flag exists.
- [ ] W-3 production entrypoint: the block fires from hook_dispatcher.rs::dispatch_pre_tool_use (:947), NOT a #[cfg(test)] caller.
- [ ] W-6 dashboard E2E: BlastRadiusDrawer renders the pre/post-expansion command diff via dashboard-smoker.
- [ ] The blocked SSB row carries BOTH pre_expansion_command and post_expansion_command (the #10077 forensics fix) —
  a contract test asserts both fields are populated and differ for a shell-expanded command.
- [ ] A null-byte / control-char host is rejected before resolve (a contract test asserts EgressDenied on a
  null-byte-spoofed host); the post-expansion FS gate blocks `rm -rf` whose glob/~/symlink resolves outside the worktree.
- [ ] GD-4 FP gate: a 10-benign-flow contract test (in-worktree writes + allowed-host fetches) shows 0 false blocks;
  the over-block rate is MEASURED + recorded on >=3 real orchestrate runs before default-on.
- [ ] blast_radius_blocked round-trips via the SSB roundtrip contract + action_type_lock golden; SCHEMA bump named
  baseline 205 -> 206, TOTAL_VARIANTS 482 -> 483.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-safety -p elai-orchestrator -p contracts.
- [ ] No new dependency (std::fs + blake3 + researcher_net resolve are in-tree; anti-slopsquatting — verify before any add).

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1; check_fs_target/check_egress_host (produced) are consumed
  by dispatch_pre_tool_use (W-8), reached by the live PreToolUse path (W-3).
- W-2: only ELAI_BLAST_RADIUS_DISABLE; default ON; observable_effect runs with no override.
- W-3: hook_dispatcher.rs::dispatch_pre_tool_use (:947); not test-only.
- W-4: blast_radius_blocked is human-surfaceable -> BlastRadiusDrawer + get_blast_radius_blocks; cross_lang_diff PASS.
- W-5: observable_effect is a runnable orchestrate command -> a live ssb.db row + a blocked verb. Pure in-process,
  no RunPod fixture needed.
- W-6: ACCEPTANCE verifies the pre/post diff THROUGH BlastRadiusDrawer via dashboard-smoker.
- W-7: destructive_verb_allowlist.rs / hook_dispatcher.rs / researcher_net.rs / agent_pool.rs / ssb.rs symbols
  grep-confirmed 2026-06-02 (lines in Context); pretool_argv.rs + destructive_verb_denied confirmed ABSENT and
  NOT cited; the BlastRadius gate hook site re-greped by the validator.
- W-8: check_fs_target/check_egress_host (produced) called in the dispatch (Task 2); BlastRadiusBlockedPayload emitted
  (Task 2/3) + consumed by the Tauri command (Task 5). No produced symbol caller-less.
- W-9: ONE Pattern-2 BlastRadiusBlocked variant + 2-arm BlastRadiusKind (mirror CapabilityGateRejected +
  RouteDecisionEventKind); action_type golden + cross_lang_diff; bump 205->206 / 482->483.
- W-10: entirely Tier-1 in-process (canonicalize + prefix/set membership) — no subprocess/RunPod at gate time;
  genuinely fail-closed (un-canonicalizable target -> Block).
- GD-1: COMPOSES-WITH E-30 (verb allowlist) + G-1 (trust label) + researcher_net (resolve/allow-set); re-implements
  none. GD-3: no perf/safety-number claim; this is a deterministic gate, not a measured-number plan. GD-4: the FP
  release gate is an ACCEPTANCE item.

## Tests to add
- crates/contracts/tests/g3_blast_radius_contract.rs (~190 LOC): (a) `rm -rf` whose glob/~/symlink resolves OUTSIDE
  the worktree -> FsEscape block + blast_radius_blocked with pre!=post command; (b) an in-worktree write -> Allow;
  (c) a null-byte/control-char host -> EgressDenied before resolve; (d) a fetch to a non-allowed resolved IP ->
  EgressDenied; (e) a fetch to an allowed host -> Allow; (f) 10 benign flows -> 0 false blocks (FP); (g) the
  BlastRadiusBlocked Pattern-2 variant round-trips for both Kind arms; (h) an un-canonicalizable target fails CLOSED.
- crates/contracts/tests/fixtures/blast_radius/ (>=10 escape fixtures + 10 benign flows for the FP guard).

## Files touched
- CREATE: crates/safety/src/blast_radius.rs (~150 LOC)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g3.rs (~110 LOC)
- CREATE: crates/dashboard-v2/src/lib/drawers/BlastRadiusDrawer.svelte (~180 LOC)
- CREATE: crates/contracts/tests/g3_blast_radius_contract.rs (~190 LOC)
- CREATE: crates/contracts/tests/fixtures/blast_radius/*.json (>=20 fixtures)
- CREATE: workflow/reasoning/G-3-decisions.md (~90 LOC)
- MODIFY: crates/safety/src/lib.rs (~2 LOC: pub mod blast_radius;)
- MODIFY: crates/orchestrator/src/hook_dispatcher.rs (~60 LOC: post-expansion FS + egress check after the E-30 argv check)
- MODIFY: crates/orchestrator/src/ssb.rs (~90 LOC: BlastRadiusBlocked variant + BlastRadiusKind + payload + sql_str + parse + CHECK + golden; 5-touchpoint mirror of CapabilityGateRejected)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (~2 LOC: TOTAL_VARIANTS 482->483 + ALL_VARIANTS row) + tests/action_type_lock.golden (~1 row, alphabetical insert)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + DrawerHost.svelte (~6 LOC: DrawerKey 'blast-radius')

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R3 (the gap + grounding). LLM permission classifier 81%
  FNR under adversarial scope-escalation = arXiv:2604.04978 (CONFIRMED in trust-max) — the deterministic-gate-beats-
  classifier grounding, alongside the in-tree E-30 deterministic-first precedent (destructive_verb_allowlist.rs:1-6,
  Replit ALL-CAPS DO-NOT-DROP ignored 11x, theregister.com 2025-07-21). rm-rf #10077 / #12637 (shell/glob/tilde
  expansion after validation) CONFIRMED; SOCKS5 null-byte egress bypass CONFIRMED; Landlock (kernel 6.7+) cited as an
  OS backstop NOT in use today (ABSENT in-tree).
- ELAI ground truth (grep+source-confirmed 2026-06-02): researcher_net.rs allowed_hosts:12 / apply_iptables_output_allow:109 /
  configure:50; sandbox/mod.rs compile_seccomp_network_extra:1269 (AF_INET wholesale); destructive_verb_allowlist.rs
  check_argv:157 (E-30, no typed SSB row — HookFinding "e30-argv-deny" hook_dispatcher.rs:974); dispatch_pre_tool_use:947;
  capability_gate.rs allow_tool_with_origin:84 / DESTRUCTIVE_TOOLS:57; provenance.rs Provenance:131 / Provenanced:196;
  ssb.rs CapabilityGateRejected:6648 (payload:20117) + RouteDecisionEventKind:24375; commands.rs ensure_within_project_root:7386;
  agent_pool.rs scratch_dir:497. Landlock + macOS Seatbelt ABSENT; Windows WindowsAppContainerSandbox mod.rs:809.
  SCHEMA_VERSION 205 (ssb.rs:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, SCHEMA baseline+N, anti-slopsquatting.

## Open questions / risks
- TOP failure mode (GD-4): over-blocking a legitimate write that resolves outside the scratch worktree (e.g. a write
  to a config dir). MITIGATION: the worktree root must be the right containment boundary (scratch_dir:497 + any
  declared allowed_paths); the 10-benign FP guard; the measured over-block rate on >=3 runs before default-on.
  UNVERIFIED until measured: ELAI's own over-block rate (no baseline).
- The expansion step is the hard part: the gate needs the POST-shell-expansion command, not the raw source. Confirm
  at authoring how the Bash tool surfaces the expanded argv (vs re-implementing shell expansion); record the chosen
  expansion source in the decision doc. A wrong expansion is a silent hole.
- OS coverage is uneven (Linux seccomp/veth + Windows AppContainer; macOS bare). The deterministic std::fs FS gate +
  host check are OS-agnostic and are the floor; do NOT claim kernel-level FS confinement on Windows/macOS.
- Do NOT regress the carried ground truth: pretool_argv.rs does not exist (E-30 is destructive_verb_allowlist.rs);
  destructive_verb_denied SSB type does not exist; there is no central worktree_root() helper (canonicalization is
  scattered — worker_boot.rs:588, oracle_cache.rs:71, commands.rs:7386); SCHEMA/TV are 205/482 not the stale 191/464.

## Next
G-3's blast_radius_blocked rows feed G-2's RRA (a runtime block is part of the reproducible run record) and the G-5
cockpit (a reviewer sees the deterministic FS/egress blocks per run). G-1's live taint enriches the trust_label on
each block (an Untrusted-flow destructive sink scores higher). G-3 + E-30 together are the full deterministic
verb-AND-path/host blast-radius gate — verb (E-30, pre-expansion) plus expanded target/host (G-3, post-expansion).

End of G-3.

# J-87: journal-completeness substrate — prove every covered side-effect produced a journal row

- Track J (lacuna-fill track) | lever TRUST-THE-JOURNAL-FOR-REAL | grounding STRONG (live G-3/Tier-C/SSB-WAL grep + WebSearch-confirmed seccomp-notify/eBPF primitives 2026-06-26) | nature BUILD (the ONE genuine net-new substrate of the whole program) | effort XL | WAVE 1 | depends none (extends shipped G-3/Tier-C/SSB; pre-gated by K-21 doctrine)
- KEYSTONE — the deepest root of the lacuna program (reasoning §E1/§9): it makes L3 honest, the canvas honest, and the C5-RULES journal labels trustworthy. Every "trust the journal / RRA / canvas / derived-status" claim is hollow without it. This is the one unit large enough for its own threat model.

- Premise: the SSB hash-chain (I-18/J-69) proves no row was ALTERED. It cannot prove every real side-effect PRODUCED a row. An append-only log cannot know it is incomplete. The whole trust spine rests on a completeness it never proved — a SHAPE-B one-level-up residue (reasoning §0): securing journal INTEGRITY leaves journal COMPLETENESS unproven. The anti-duplication grep (LFG-2) returned ZERO across every plan summary for shadow-stream / eBPF / reconcile / completeness-gate: this is the program's one green-field build. K-21 (.claude/rules/journal-completeness.md JC-1..JC-4, PLANNED) is the binding pre-gate; J-87 is the substrate K-21 governs.

## Steal/build (a three-layer stack, NOT a hash-chain; report 8, the strongest-grounded report)
1. MEDIATE leg — atomic-commit-with-row (EXTENDS shipped G-3/Tier-C, NOT new architecture). Route a SMALL declared high-value effect set (openat-for-write, connect/sendmsg egress, clone/execve child procs, privilege changes) through Tier-C using SECCOMP_RET_USER_NOTIF: the effect cannot complete unless its SSB row commits ATOMICALLY with the approval (the supervisor holds the notify-fd, sends SECCOMP_IOCTL_NOTIF_SEND only after the row lands). This is the same Linux-only privileged-supervisor posture G-3 already runs. HONEST BOUND (WebSearch-confirmed, kernel.org seccomp_filter.html): seccomp-notify is NOT a TOCTOU-safe security boundary on pointer-deref args — so the mediate leg guarantees ATOMICITY OF THE ROW, not interception soundness; the observer leg is the real completeness backstop.
2. OBSERVE leg — the NET-NEW part. Observe the rest at the kernel layer (eBPF/LSM, Tetragon/Falco/Tracee-class; whole-system provenance LPM/CamFlow at the 2.7-6% overhead class) into an INDEPENDENT feed the executor cannot author. This feed is the second, out-of-band witness of real effects.
3. RECONCILE gate — missing=FAIL (doctrine R1, applied to completeness). Any kernel-observed sensitive effect with no matching journal row inside a bounded window = DETERMINISTIC FAIL. A NORMALIZATION layer is MANDATORY first (temp-file churn / path aliasing / retries are SCHEMA-MISMATCH FPs, not hallucinated events — the claim is "every COVERED effect class left a row", never universal completeness). Deterministic (TD-3), OBSERVE-ONLY first (GD-4).
4. MONITOR-THE-MONITOR — dropped-event counters FAIL CLOSED; "observer impaired" (eBPF map full, ring-buffer overrun, observer crashed) is a FIRST-CLASS verdict, never silent continuation (R2: a dead evidence path screams). The reconcile gate self-tests its observer feed at startup.
5. C8-CANVAS hook (the honesty render, derivative of E1): once the observer feed exists, render observed-effect-with-no-journal-row as ANOMALY nodes so the canvas honestly shows the gaps (consumed by J-94 / J-60).
EXCLUDED: any claim of UNIVERSAL completeness (only the COVERED effect classes); any claim that this defeats a privileged HOST/kernel adversary (it does not — see residue); the observer leg on Windows (mediation-only fallback there, R3); an LLM anywhere in the reconcile decision (TD-3).

## Existing ground truth (grep-confirmed in the 2026-06-26 feature-map read; re-grep at edit, W-7/W-9)
- TIER-C PROCESS ISOLATION [WIRED] seccomp 2 profiles, minimal OS processes; crates/agent + crates/ipc-types; phase-4A. The mediate leg adds a seccomp profile + a notify-fd supervisor on the existing Tier-C seam.
- G-3 BLAST-RADIUS RUNTIME GATES [WIRED] post-shell-EXPANSION FS gate + egress IP-pin; Linux-only. Same privileged-host posture the mediate leg extends.
- SSB SHARED STATE BOARD [WIRED] single-writer SQLite WAL, append-only, broadcast-after-commit; ssb.rs. The mediated row commits atomically here; the reconcile gate reads it.
- K-21 .claude/rules/journal-completeness.md JC-1..JC-4 (PLANNED) — the binding pre-gate (integrity!=completeness / mediate-or-observe / reconcile-missing=FAIL+monitor-the-monitor / honest covered-scope + platform residue).
- WebSearch-confirmed 2026-06-26: SECCOMP_RET_USER_NOTIF (Linux 5.0+, RECV/SEND ioctl, pointer-deref caveat); Tetragon/Falco/Tracee (CNCF eBPF, in-kernel syscall/file/network provenance). Design-rationale only.

## Integration
- crates/orchestrator/src/journal_completeness/ (new sub-track): mod.rs (the reconcile gate + normalization), observer.rs (the eBPF/LSM feed adapter, Linux-only, behind cfg), mediate.rs (the seccomp-notify supervisor extending Tier-C). The reconcile gate wired into the AUDITING entry + a post-tick reconcile in live_cli (auditor-owned, never the main LLM; H-3).
- crates/agent: the seccomp profile add + notify-fd handoff on the Tier-C tool-dispatch seam.
- +JournalCompletenessEvent SSB (Pattern-2 single-variant; Kind: EffectMediated / EffectObserved / ReconcileMatched / ReconcileMissingFAIL / ObserverImpaired). NET-NEW (this is one of the wave's two genuinely-new events). baseline+1 from LIVE ~268/577 (ssb.rs:1503/1508 / action_type_lock.rs, grep 2026-06-26); re-read at edit (W-9). Full 9-tuple lockstep.
- FEATURE FLAG: ELAI_JOURNAL_COMPLETENESS_ENABLE (kill-switch, default OBSERVE-ONLY) + a wire-dark-allowlist.toml row (type kill_switch, owner, expiry, promotion_plan_id=J-87-promotion, smoke exercising the ON path) per W-2/R9.
- frontend_pair: a completeness panel on the audit-timeline / canvas process surface showing reconcile matches + any ReconcileMissingFAIL + ObserverImpaired state (consumed by J-94 C8-CANVAS); cross_lang_diff if surfaced (W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/journal_completeness/mod.rs::reconcile_effects (observed-effect <-> journal-row matcher; missing=FAIL)
- crates/orchestrator/src/journal_completeness/mod.rs::normalize_observed_effect (the mandatory schema-mismatch normalizer)
- crates/orchestrator/src/journal_completeness/observer.rs::ObserverFeed (the independent eBPF/LSM feed; Linux-only cfg)
- crates/orchestrator/src/journal_completeness/observer.rs::observer_health (dropped-counter monitor; impaired=FAIL)
- crates/orchestrator/src/journal_completeness/mediate.rs::mediate_effect (seccomp-notify atomic commit-with-row on the Tier-C seam)
consumes_symbols:
- TIER-C seccomp profile + IPC seam (crates/agent; the mediate leg extends it, NOT reauthored)
- G-3 runtime-gate posture (the Linux-only privileged-host model; not reauthored)
- the SSB writer (ssb.rs; the mediated row + the reconcile read)
reachable_from:
- the AUDITING FSM transition + the live_cli post-reflect tick on a real `elai` run that performs a covered effect (a write / an egress connect / a child proc)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO claim of universal completeness (covered effect classes ONLY; the covered set is DECLARED data)
- NO LLM in the reconcile decision (deterministic matcher; TD-3)
- NO silent-continue on a dropped/overrun observer feed (ObserverImpaired is a verdict; R2)
- NO BLOCK default until the FP/over-block rate is bounded on >=3 real runs (GD-4) + a wire-dark-allowlist row (W-2/R9)
- NO observer leg on Windows (mediation-only fallback; the asymmetry is documented, never advertised away; R3/CL-4)
- NO reconcile-missing emitted as SKIP/UNVERIFIED (missing=FAIL; R1)
frontend_pair: completeness panel on the audit/canvas surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "an `elai` run that opens a file for write and makes an outbound network connect, driven through the real AUDITING transition with the observer feed up; AND a fault run where the orchestrator deliberately suppresses the journal row for one mediated effect"
- expect_ssb: "EffectObserved for the write + the connect; ReconcileMatched for both when the rows exist; on the fault run ReconcileMissingFAIL naming the unjournaled effect"
- expect_artifact: "the reconcile gate is RUN by the auditor over the independent observer feed (NOT the executor's stdout); the fault run's missing row FAILS the gate and BLOCKS the FSM advance. NEGATIVE (R2): killing the observer mid-run raises ObserverImpaired and FAILS the gate, never SKIPs/passes; a normalized temp-file-churn event does NOT raise a false ReconcileMissingFAIL (the normalizer absorbs it); a reconcile pointed at an empty observer feed FAILS its startup self-test, never passes."

## Grounding
- mechanism: report 8 §re-executable + §verifier-isolation (the three-layer mediate/observe/reconcile stack). DESIGN-RATIONALE only (LFG-3); the done-gate is ELAI-measured (the fault-run ReconcileMissingFAIL).
- primitives WebSearch-confirmed 2026-06-26: SECCOMP_RET_USER_NOTIF (Linux 5.0+, kernel.org seccomp_filter.html — incl the pointer-deref TOCTOU caveat that bounds the mediate leg); Tetragon/Falco/Tracee eBPF/LSM (CNCF, in-kernel provenance; the 2.7-6% CamFlow/LPM overhead class is design-rationale, not an ELAI number).
- substrate: G-3 + TIER-C + SSB-WAL [WIRED] + K-21 doctrine [PLANNED] — feature-map 1.3/1.4/1.6/1.8.
- live SSB baseline ~268/577 (ssb.rs:1503/1508, grep 2026-06-26).
- anti-duplication LFG-2: ZERO summary-index/phase-index matches for the net-new concepts.

## COMPOSES-WITH
G-3 + TIER-C (extends the mediation posture), SSB hash-chain I-18/J-69 (the orthogonal INTEGRITY axis; completeness is the missing second axis), K-21 (the doctrine pre-gate), L3 re-executable RRA J-91 (consumes a complete record), C5-RULES non-LLM labels J-93 (consumes the trustworthy outcome stream), C8-CANVAS shadow-anomaly nodes J-94 (renders the gaps), the L4/L5/L7 gates J-90 (a complete journal makes their evidence trustworthy). Restates none.

## Open questions / residue (reasoning §7 — named, or it over-claims)
- HOST-TRUST CEILING: a privileged host/kernel adversary defeats BOTH logs. The honest mitigation is Rekor/TPM-IMA anchoring — a RIDER on I-18/G-8, NOT claimed by J-87. State the ceiling in-file; never advertise past it.
- SCHEMA-MISMATCH FP FLOOR: observer FPs are normalization failures (temp-file churn, path aliasing, retries), not hallucinated events. The normalizer is mandatory and the claim is bounded to "every COVERED effect class left a row."
- PLATFORM RESIDUE (R3/CL-4): the observer leg is Linux-only (remote-3090-gated like G-3); the Windows dev box gets mediation-only (no independent observer there). Document per lane; never advertise the cloud guarantee locally.
- COVERED-SET COMPLETENESS: the high-value effect set is DECLARED data — an effect class not in the set is silently uncovered. The set is a frozen, user-reviewed list; adding a class is a deliberate act, and the gate logs which classes it covers (no silent scope-narrowing, R-no-silent-caps).

## Next
J-91 (re-executable RRA) consumes the complete record this proves; J-93's C5-RULES labels + C3-AGENTS supervisor consume the trustworthy outcome stream; J-94's C8-CANVAS renders the anomaly nodes. This is the substrate the rest of the "trust the journal" program rests on.

End of J-87.

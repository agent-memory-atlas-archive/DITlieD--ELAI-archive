# ELAI OWASP Control Mapping

Status vocabulary is deliberately bounded: each row records a design posture, not a certification claim. Lists re-verified against the canonical OWASP LLM Top 10 (2025) and Agentic Top 10 (2025) at `genai.owasp.org`; the stale `owasp.org` v1.1 landing list is not used. Controls and residuals are promoted from `owasp-llm-agentic-gap-analysis-2026-06-25.md`, Sections B/C/D.

ELAI is local-first and single-user. “N-A” below means the service-provider threat is outside that boundary, not that a future multi-seat deployment may ignore it. Model inversion/extraction/theft (the LLM02 tail) re-opens under multi-seat and is owned by J-59 sequencing.

| OWASP threat (2025) | ELAI control mapping, citation, and honest status |
|---|---|
| **LLM01 Prompt Injection** | G-1 provenance taint + `crates/safety/src/capability_gate.rs`; F-3 CaMeL filter; I-47 verifier-input hardening; I-1 supervisor. **PARTIAL [DESIGN]**: transformation-surviving binding and E-37 perimeter completion remain WI-1/WI-3; SAFEDELTA AgentDojo delta is **[BENCH-PENDING]**. |
| **LLM02 Sensitive Information Disclosure** | G-9 write-time secret/PII redaction; F-1 VibeGate secret scan; G-3 egress allowlist; I-50 auth material exclusion. **WIRED-DESIGN [DESIGN]** for the local agent threat; model inversion/extraction/theft is **N-A [DESIGN]** single-user-local and re-opens under multi-seat → J-59. |
| **LLM03 Supply Chain** | F-9 hallucinated-dependency gate; E-24 schema pin + G-4 live re-pin; E-43 skill audit; I-18 AI-BOM; G-8 attestation; G-10 fingerprint pin. **WIRED-DESIGN [DESIGN]**; no external OWASP certification is implied. |
| **LLM04 Data and Model Poisoning** | F-5 staleness quarantine; F-12 NLI and F-8 conflict detection; E-5 MINJA memory controls; J-21 semantic validation is planned; G-10 drift canary. **PARTIAL [DESIGN]**: the J-21 write-quarantine leg is **PLANNED(J-21)**. |
| **LLM05 Improper Output Handling** | F-1 VibeGate AST/GREP; F-2 CWE rules including CWE-95; F-4 output non-blind-trust; I-46 bash interceptor; SAFEDELTA target. **WIRED-DESIGN [DESIGN]**; injection/output delta remains **[BENCH-PENDING]**. |
| **LLM06 Excessive Agency** | Tier-C process isolation; G-1 capability gate; B-22 autonomous gate; E-30 argv allowlist and human approval; I-1 sole side-effect supervisor. **WIRED-DESIGN [DESIGN]**. |
| **LLM07 System Prompt Leakage** | I-1 keeps credentials outside generator context; I-50 prevents auth journaling; G-9 redaction. **WIRED-DESIGN [DESIGN]**: secret-in-prompt minimization is the control posture. |
| **LLM08 Vector and Embedding Weaknesses** | G-1 taint on retrieved content; F-5 quarantine; J-21 retrieval authorization/provenance; H-31 superseded-content filter; content-hash cache. **PARTIAL [DESIGN]**: J-21 retrieval authorization remains **PLANNED(J-21)**. |
| **LLM09 Misinformation** | D-16 conformal refusal; H-39 abstention scoring; H-53 citation pre-binding; F-4 cant-lie; D-13 adversary; G-10 drift canary. **WIRED-DESIGN [DESIGN]**; SAFEDELTA is a recompute surface, not a leaderboard claim. |
| **LLM10 Unbounded Consumption** | B-27/C-5 hard cost ceiling; E-39 cost economics; H-38 cost confirmation; J-28/J-29 token ledger; anti-spiral and 50-step breaker. **WIRED-DESIGN [DESIGN]** for local denial-of-wallet; model-extraction-as-DoS is **N-A [DESIGN]** in the single-user boundary. |
| **ASI01 Agent Goal Hijack** | G-1 taint + capability gate; F-3 tool filter; I-1 supervisor; E-37 prompt-injection defense. **PARTIAL [DESIGN]**: same WI-1/WI-3 binding gap as LLM01; AgentDojo delta is **[BENCH-PENDING]**. |
| **ASI02 Tool Misuse** | E-30 argv allowlist; G-1 capability gate; C-35 tool gating; Tier-C tool-less default. **WIRED-DESIGN [DESIGN]**. |
| **ASI03 Identity and Privilege Abuse** | I-1 supervisor holds credentials; I-50 auth broker; SO_PEERCRED IPC authentication; J-49/J-59 per-seat isolation. **PARTIAL [DESIGN]**: multi-seat isolation is **PLANNED(J-59)**. |
| **ASI04 Agentic Supply Chain** | E-24 blake3 MCP schema pin; G-4 live re-pin on tool-list change; F-9 dependency gate; E-43 skill admission; C-42 MCP authorization. **WIRED-DESIGN [DESIGN]**. |
| **ASI05 Unexpected Code Execution** | F-1 VibeGate; F-2 CWE-95; E-30 allowlist; I-46 interceptor; Tier-C sandbox; G-1 gate. **WIRED-DESIGN [DESIGN]**. |
| **ASI06 Memory and Context Poisoning** | E-5 MINJA; H-31 hard filter; H-32 citation validation; F-5 staleness quarantine; J-21 write validation. **PARTIAL [DESIGN]**: J-21 semantic validation is **PLANNED(J-21)** and Recallable memory does not itself block sinks. |
| **ASI07 Insecure Inter-Agent Communication** | Tier-C typed postcard IPC, 10MB frame cap and SO_PEERCRED; SSB single-writer ordering; F-8 inter-agent BSP is deferred. **PARTIAL [DESIGN]**: F-8 half is **PLANNED(F-8)**. |
| **ASI08 Cascading Failures** | blake3 anti-spiral stop; max_restarts=3; 50-step/event breakers; J-16 blast-radius gate; cross-domain canary. **PARTIAL [DESIGN]**: J-16 is **PLANNED(J-16)**. |
| **ASI09 Human-Agent Trust Exploitation** | F-4 receipts/cant-lie; H-53 citation pre-binding; G-5 reviewer cockpit; K-3 evidence renderer; D-13 adversary. **WIRED-DESIGN [DESIGN]**: approval is evidence-bound rather than prose-bound. |
| **ASI10 Rogue Agents** | H-43 execution-grounded fitness; H-44 behavioral-trap detection; F-10 security monotonicity; L1 HardVeto; R8 frozen gates; I-1 supervisor. **PARTIAL [DESIGN]**: H-43/H-44 remain **PLANNED(H-43/H-44)** where not yet wired. |

## Boundary and recomputation

The mapping is a buyer-facing render of existing controls, not a claim of measured OWASP performance. The one load-bearing measurement is the gate-ON-vs-OFF injection-defense delta for LLM01/LLM05/ASI01 from G-6 SAFEDELTA's AgentDojo arm; until the run lands it remains `[BENCH-PENDING]`, recomputed with `elai bench safedelta` and `safedelta_runner.rs`.

The mapping feeds the future K-6 compliance export as a procurement clause (K-6 is not modified here) and is paired with K-1's dossier/lint. Technical hardening WI-1/WI-2/WI-3 and config admission WI-6 are explicitly out of this package.

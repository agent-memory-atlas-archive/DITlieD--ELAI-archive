AMEND — the residual is OPEN and valid; PaperStoreRefResolver does not exist on main, the ClaimRefResolver trait is the right seam, but the cited design doc conductor/opus_walk/evidence/L-9/seam-design-proposal.md §5 is NOT in the repo and the workflow/papers store is markdown interface maps not raw paper bytes, so the resolver design must be amended to resolve against CosmosArtifact.body or workflow/papers/*.md text.

EVIDENCE

1. PaperStoreRefResolver does NOT exist on main
   - rg "PaperStoreRefResolver|PaperStore|paper_store|paper-store" across .elai_cc/crates/ → 0 hits (Bash grep -rn, whole crates tree)
   - rg across the entire  repo (excluding .git/) → 0 hits in any .rs/.py/.md/.json/.toml
   - The ONLY reference is elai-status.json:5359: "Residual OPEN: slice 6 PaperStoreRefResolver (claims vs paper bytes)."
   - .elai_cc/crates/orchestrator/src/claim/resolve.rs contains only WorkspaceRefResolver (resolve.rs:7) + workspace_resolver() (resolve.rs:128); no PaperStoreRefResolver struct, impl, or function

2. research_grounding_pass / research_profile does NOT call any paper-store resolver
   - .elai_cc/crates/orchestrator/src/reasoning/research_profile.rs:59-61: the pass calls crate::claim::resolve::workspace_resolver() → extract_claims_with(reply, resolver). This resolves claims against workspace files + DKG symbol index, NOT paper bytes.
   - research_profile.rs has zero references to "paper", "cosmos", "workflow/papers", or any paper-store API (grep -rn "paper|cosmos|workflow/papers" research_profile.rs → 0 hits)
   - The round-trip path (roundtrip.rs:109 closed_set_from_resolved_refs) builds its EvidenceAnchorSet from resolved refs in the ClaimSet — these refs are workspace paths/symbols/packages, not paper content
   - LiveCli::research_grounding_for_turn (live_cli.rs:811-835) calls research_grounding_pass with the reply + candidates; no paper-store injection point

3. ClaimRefResolver trait IS still the right seam
   - .elai_cc/crates/orchestrator/src/claim/extract.rs:30-34: pub trait ClaimRefResolver { resolve_path, resolve_symbol, resolve_package } — all default to false
   - extract.rs:41: extract_claims_with(text: &str, resolver: &dyn ClaimRefResolver) — the injection point for any custom resolver
   - resolve.rs:80: impl ClaimRefResolver for WorkspaceRefResolver — the existing implementation pattern
   - prune.rs:35: prune_claims also takes resolver: &dyn ClaimRefResolver
   - A PaperStoreRefResolver would implement this same trait, adding a resolve method that checks claims against paper content. The trait shape is correct but may need extension: the current trait has resolve_path/resolve_symbol/resolve_package — none of these naturally express "does this claim text appear in an ingested paper's bytes?" The resolver may need a new method (e.g. resolve_against_paper_text) or the claim extraction itself may need to route factual-claim text through a paper-content matcher.

4. workflow/papers store location and file count
   - workflow/papers/ contains 341 files total, 222 are *-interface.md files (find -type f | wc -l)
   - These are MARKDOWN interface maps produced by paper-ingestor (skills/paper_ingest/SKILL.md:29: "Output: workflow/papers/<slug>-interface.md"), NOT raw paper bytes/PDFs
   - The worktree (.elai_cc/) does NOT contain workflow/papers/ — it lives at the repo root, outside .elai_cc/
   - CosmosArtifact (cosmos_worker.rs:248-254) has body: String — this is the on-disk artifact content accessible via CosmosWorker::cosmos_get(name) (cosmos_worker.rs:1015)
   - The Cosmos artifact store (~/.elai/cosmos/) is the B-42 wired path; CosmosStage (cosmos_stage.rs:74) is in the QueryPipeline

5. No newer plan owns this residual
   - elai-status.json:5353-5364 (L-9 entry, updated 2026-08-11T13:39:42Z): verdict=drained, implementation_state=claimed_delivered, reason="LANDED+PUSHED ef2af276 ... Residual OPEN: slice 6 PaperStoreRefResolver (claims vs paper bytes)."
   - opus-backlog-walk.md:16 (2026-08-11): confirms L-9 landed ef2af276, does NOT mention slice 6 as a separate plan or superseding design
   - The cited "conductor/opus_walk/evidence/L-9/seam-design-proposal.md §5 slice 6" does NOT exist: find /home/user/teikoku/elai -name "seam-design-proposal*" → 0 results; the only L-9 evidence file is S3-wiring-backend-precheck.md (2026-08-09, pre-landing)
   - No newer L-9 plan or slice-6-specific plan exists: find /home/user/teikoku/elai -name "*L-9*" → only the original plan, the state-check artifacts, the wiring report, and the evidence dir
   - The L-9 plan file itself (L-9_research_math_deepthink_grounding_profile.md:63) states the acceptance: "claims round-trip against the paper bytes (L-3/H-53)" — this is the open residual

CONCRETE DELTAS REQUIRED (for EXECUTE agent)
- The cited seam-design-proposal.md §5 is missing; the EXECUTE agent must work from the plan file (L-9_research_math_deepthink_grounding_profile.md:63) + this recon, not from a non-existent design doc
- PaperStoreRefResolver must be created in .elai_cc/crates/orchestrator/src/claim/resolve.rs, implementing ClaimRefResolver
- The trait may need a new method or the resolver may need to override resolve_path/resolve_symbol to check against paper content — the current trait methods (path/symbol/package) do not naturally express "claim text vs paper bytes"
- The paper content source is either workflow/papers/*.md (markdown interface maps, 341 files) or CosmosArtifact.body via CosmosWorker::cosmos_get — the design must pick one; CosmosArtifact.body is the B-42 wired path, workflow/papers/ is the paper-ingestor output
- research_grounding_pass (research_profile.rs:59-61) must be modified to use the PaperStoreRefResolver (or a composite resolver) instead of only workspace_resolver()
- The production hook is research_grounding_for_turn (live_cli.rs:811) → research_grounding_pass (research_profile.rs:51) → extract_claims_with (extract.rs:41) — the resolver injection point
- ACCEPTANCE from plan line 63: "claims round-trip against the paper bytes" — the contract test l9_research_profile_contract.rs currently has NO paper-byte round-trip test; it only tests abstention + empty reply + divergent candidates

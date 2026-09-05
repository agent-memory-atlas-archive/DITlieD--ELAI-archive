# I-54: structural AST search+edit tool — pattern-matched, syntax-aware edits over the shipped tree-sitter grammars (the idea, not the ast-grep dep)

- Track 20 | lever TOOLS-VERIFIED | grounding STRONG (SL-7) / MEDIUM (gajae semantics, SL-10) | nature
  BUILD | effort M | WAVE 3

- Premise: STEAL-LIST B3 with its caveat made binding (SLC-3): gajae exposes structural AST search+edit
  as an agent tool (rename-safe, syntax-aware multi-site edits that string-replace cannot do safely) —
  but it does it by vendoring ast-grep-core, and ELAI rejected that dep. Steal the IDEA: ELAI's knowledge
  crate already pins tree-sitter 0.26 + 9 language grammars (SL-7). Build the tool on those: a structural
  query (tree-sitter query syntax — already the grammars' native pattern language, no new DSL) + a
  capture-template rewrite, with a preview arm so the agent (and the journal) sees the match set before
  any byte lands.

## Steal/build
- Steal the tool ergonomics (SL-10 binding read: gajae packages/coding-agent/src/tools/ast-edit.ts +
  ast-grep.ts before EXECUTING — for the INTERFACE only; the engine is tree-sitter queries, SLC-3).
- Build, as an agent tool (3-arm): search (tree-sitter query -> match list with file:line + capture
  bindings), preview (query + rewrite template -> the would-be diff, no writes), apply (the previewed
  edit set, atomically per file, refusing if any target file changed since preview — staleness guard).
  apply routes through the SAME PreToolUse edit gates as the Edit tool (F-1 VibeGate etc. — a structural
  edit is still an edit).
- EXCLUDED: ANY dep named ast-grep (SLC-3 — a BLOCK in review); no cross-language rewrite in one call
  (per-language queries); no semantic/type-aware renaming (that is LSP territory — E-52's lane; this
  tool is syntactic and says so); no auto-apply without a preceding preview row.

## Existing ground truth
- SL-7: crates/knowledge/Cargo.toml:13-22 pins tree-sitter + rust/python/go/typescript/java/c-sharp/
  ruby/javascript/c grammars. The grammars are in-tree and already power knowledge-side parsing — the
  tool reuses those parser builds, never duplicates grammar pins in another crate (one version source).
- impact_radius.py + elai_graph reachability exist harness-side for blast-radius; the tool's preview arm
  is the runtime-side equivalent surface.

## REVISED scope (W-10 tiered)
- SHIP NOW: search/preview/apply for the Rust + Python + TypeScript grammars (ELAI's own three
  languages), catalog-registered, gated.
- DEFER: the remaining six grammars (enable per demand); rewrite-template library.

## Integration
- crates/tools/src/structural_edit.rs consuming knowledge's parser/grammar surface (a knowledge pub API,
  not a second tree-sitter pin) + tool catalog registration + the PreToolUse edit-gate path.
  frontend_pair: preview/apply rows in the existing tool-call drawer (the previewed diff is the
  operator-visible artifact; no new component).

## WIRING_CONTRACT
- produces_symbols: tools::structural_edit (search/preview/apply) + the staleness guard +
  StructuralEditApplied SSB rows.
- consumes_symbols: knowledge tree-sitter parsers/grammars (SL-7 — the single version source), the
  PreToolUse edit gates (F-1 seam — composed, never bypassed), the tool registry.
- reachable_from: the production agent tool dispatch (a live agent running search -> preview -> apply on
  a real crate). Contracts-only callers do not satisfy (R7).
- stub_ban: NO ast-grep dep (SLC-3, BLOCK); NO apply without a preview row (the journal must show the
  match set first); NO regex fallback when a parse fails (typed ParseFailed, never degrade to string
  replace); NO second tree-sitter version pin; NO edit-gate bypass; NO unwrap/expect on prod paths.
- observable_effect:
  trigger: "a live agent renames a function's call sites across a fixture crate: search, preview, apply"
  expect_ssb: "StructuralEditApplied(Matched) with the match count, (Previewed) with the diff hash, then
  (Applied) with per-file results; a mid-flight external file change lands (RefusedStale)"
  expect_artifact: "post-apply, the fixture compiles and the rename touched ONLY the previewed spans
  (byte-diff equals the preview diff); the NEGATIVE arm — a syntactically-broken rewrite template — is
  caught at preview (ParseFailed) with ZERO bytes written; an apply with no preceding preview row is the
  broken arm and FAILS (R1)."

## Grounding
- STEAL-LIST B3 (Tier 2, idea-steal); SL-7, SL-10, SLC-3.
- CLAUDE.md no-new-deps (the whole point of the SLC-3 framing) + impact analysis (preview-before-apply
  is the rule made mechanical).

## COMPOSES-WITH
- knowledge grammars (the engine — one pin), F-1 VibeGate/PreToolUse edit gates (apply rides them), E-52
  LSP lane (the SEMANTIC sibling — disjoint, named in-file), I-20 (survivor Directives may cite
  structural edit as the fix vehicle), impact-mapper (harness-side blast-radius sibling). Restates none.
  SSB: +StructuralEditApplied (Kind: Matched/Previewed/Applied/RefusedStale/ParseFailed). baseline+1
  from LIVE 221/532 (SL-1); re-read at edit (W-9).

End of I-54.

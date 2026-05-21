# Foundation Models Framework — Spec Stub (E-51)

Status: DEFERRED. No live code. Pure specification for future companion-iOS phase.

## Target Platforms
- iOS 18+, macOS 15+
- Apple Intelligence-eligible devices only (A17 Pro / M1+ Neural Engine)
- Excludes: Intel Mac, older iPhones, iPad without A-series M-grade

## On-Device Model
- ~3B parameter model managed by Apple
- Context cap: 4096 tokens
- NOT a coding-model replacement per E-51 report (caps at 4096; abliterated coding
  models need 8K+)
- Suited for structured planning JSON, short summarisation, classification

## Guided Generation (@Generable)
- Swift 5.9+ @Generable macro generates conforming types
- Inference returns Codable-conformant structs — zero hallucinated fields
- Use case: research_tree DTO structured output (feasibility_report sub-fields)
- NOT suitable for free-form code generation

## Anthropic-Protocol Bridge Sketch (future companion app)

```pseudo
// Swift companion accepts POST /v1/messages on localhost
// Maps `messages[0].content` -> FoundationModels.LanguageModel.generate(prompt:)
// Maps @Generable structured result -> Anthropic MessageResponse JSON
// Falls back to cloud (Sidecar/AnthropicApi) for requests exceeding 4096 ctx
//   or containing multi-turn conversation history > 1 turn
```

## Integration Point with ELAI B-37 RuntimeAdapter
- Future RuntimeBackend::FoundationModels variant
- MlxCapability::AppleSilicon + iOS companion app running in same local network
- Chain position: below Mlx, above LlamaCpp (fastest struct output, lowest latency)

## Open Questions (from E-51 report)
- @Generable Swift struct <-> research_tree DTO mapping feasibility
- Auth model for companion app <-> ELAI desktop communication
- Whether structured output caps at @Generable schema complexity

## References
- developer.apple.com/documentation/FoundationModels
- WWDC25 session 286: Writing Tools and Foundation Models
- E-51 report: workflow/improvement-sweep-2026-05-19/reports/51-edge-inference-and-mlx.md

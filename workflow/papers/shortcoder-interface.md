Algorithm: ShortCoder - Knowledge-Augmented Syntax Optimization for Token-Efficient Code Generation
ArXiv: 2601.09703
Paper: ShortCoder: Knowledge-Augmented Syntax Optimization for Token-Efficient Code Generation
Authors: Sicong Liu, Yanxian Huang, Mingwei Liu, Jiachi Chen, Ensheng Shi, Yuchi Ma, Hongyu Zhang, Yin Zhang, Yanlin Wang (Sun Yat-sen University et al.)
Dataset: https://github.com/DeepSoftwareAnalytics/ShorterCode
Local file: .claude/papers/2601.09703v1.html

Context for elai_cc Phase 4B:
ShortCoder attacks the generation phase (not inference/prompt phase) of LLM token cost. It teaches an LLM
to emit fewer tokens without correctness loss by fine-tuning on AST-preserving simplification rules. For
elai_cc Phase 4B (context-aware file reading strategies), the three independently applicable artifacts are:
(1) The ten Python simplification rules as post-processing transforms on any generated code output.
(2) The LoRA fine-tuning recipe for teaching conciseness to any instruction-tuned code LLM.
(3) The hybrid corpus construction pipeline (rule-based + LLM-synthesized) for domain-specific training data.

Inputs:
source_code | str | Python source string | must be syntactically valid Python; UTF-8 encoded
problem_description | str | natural language prompt | passed as-is to fine-tuned model at inference
simplification_rules | list[int] | rule IDs 1-10 | optional; omit at inference (zero-shot behavior)
base_model | str | HuggingFace model ID or local path | must be instruction-tuned code LLM
lora_weights | path | LoRA adapter checkpoint | produced by fine-tuning pipeline; required for ShortCoder inference

Outputs:
simplified_code | str | Python source string | must pass all original test cases (AST-preserving invariant)
generated_tokens | int | scalar | average 162.02 on HumanEval; always < baseline GenTokens
pass_at_k | float | [0.0, 1.0] | pass@1=0.612, pass@10=0.764, pass@100=0.967 on HumanEval

Hyperparameters (fine-tuning):
lora_rank | 8 | 4-64 | low sensitivity; rank=8 balances param reduction vs expressiveness
lora_alpha | 16 | 8-32 | alpha = 2*rank is the paper default; controls effective learning rate scaling
lora_dropout | 0.05 | 0.0-0.1 | for generalization; authors use 0.05
lora_target_modules | [q_proj, v_proj] | attention projection layers only | do not expand to all layers without ablation
batch_size_effective | 4 | - | per_device=1, grad_accum=4; scaled for single-GPU constraint
learning_rate | 5e-5 | - | AdamW optimizer; authors report this specific value
epochs | 5 | - | authors use 5 epochs; more epochs risks overfitting to simplification patterns
precision | fp16 | bf16 also viable | mixed-precision enabled; use bf16 on Ampere+ (per elai_cc standards)
training_corpus_size | 828 pairs | 596 rule-based + 232 LLM-synthesized | ShorterCodeBench dataset

THE TEN SIMPLIFICATION RULES (Section III-A, all AST-preserving):

Rule 1: Multiple Variable Assignment
  Before: a = 5; b = 5; c = 5  (three separate statements assigning identical values)
  After:  a = b = c = 5
  Condition: consecutive assignments of identical value to different variables

Rule 2: Return Statement Simplification
  Before: return (x + y)
  After:  return x + y
  Condition: remove redundant parentheses in return statements; purely syntactic, zero semantic change

Rule 3: Assignment Operation Simplification
  Before: x = x + y  (or any binary op where lhs appears on rhs)
  After:  x += y
  Applies to: +, -, *, /, //, %, **, &, |, ^, <<, >>

Rule 4: Conditional Statement Simplification (ternary)
  Before: if cond: x = a\nelse: x = b
  After:  x = a if cond else b
  WARNING: paper notes this degrades readability for complex multi-expression if-else blocks

Rule 5: Multi-conditional Statement Simplification (elif chaining)
  Before: deeply nested if-else-if chains
  After:  flat if/elif/else chain
  Condition: transforms nested conditionals to linear elif structure for readability + conciseness

Rule 6: For Loop Simplification (comprehensions)
  Before: result = []\nfor x in iterable:\n  result.append(f(x))
  After:  result = [f(x) for x in iterable]
  Applies to: list comprehensions, dict comprehensions, set comprehensions
  WARNING: only safe when loop body is a single append/assignment; multi-statement loops must not be compressed

Rule 7: Multiple Object Reference Deletion
  Before: del a\ndel b\ndel c
  After:  del a, b, c
  Condition: consecutive del statements for multiple variables

Rule 8: Dictionary Mapping Simplification
  Before: if key in d: val = d[key]\nelse: val = default
  After:  val = d.get(key, default)
  Condition: key-existence check followed by value assignment; replace with dict.get()

Rule 9: String Formatting Simplification
  Before: result = str1 + str(x) + str2 + str(y)  (string concatenation with type coercion)
  After:  result = "{0}{1}".format(str1_template, ...)  or f-string
  Condition: string concatenation using + with non-string data requiring explicit str() casting

Rule 10: File Reading and Writing Simplification
  Before: f = open(path, mode); ... ; f.close()
  After:  with open(path, mode) as f: ...
  Condition: explicit open/close pattern; context manager guarantees resource release

Invariants (testable assertions):
assert ast.dump(ast.parse(original)) == ast.dump(ast.parse(simplified))  # AST structural equivalence (Section III-A preamble)
assert all(test(simplified_code) for test in test_suite)  # functional correctness preserved
assert len(tokenize(simplified_code)) <= len(tokenize(original_code))  # token count never increases
assert pass_at_1_shortcoder >= 0.612  # min threshold from Table II (HumanEval)
assert generated_tokens_mean <= 162.02  # mean GenTokens from Table II

Complexity:
Rule application (post-processing): O(N) where N = AST node count; linear single-pass per rule
Fine-tuning: O(rank * d_model^2 * seq_len * steps); LoRA rank=8 reduces trainable params by ~97% vs full fine-tuning
Inference: same autoregressive complexity as base model; no overhead beyond shorter output sequences
Space: LoRA adapters are O(rank * d_model) per targeted layer; negligible vs base model weights
Parallelism: independent rule application (Rules 1-10) can be parallelized; fine-tuning requires sequential token prediction

Failure modes:
Rule 4 applied to multi-statement if-else blocks breaks correctness (only valid for single-expression assignment)
Rule 6 applied to loops with side effects or multiple statements produces incorrect output
Rule 9 applied where format string argument order differs from concatenation order silently corrupts output
LoRA overfitting on small corpus (828 pairs) when fine-tuning > 5 epochs or on domain-shifted data
Prompt-based alternatives (NoTuning_rules_examples) produce 6.3x more total tokens than fine-tuned approach
Model trained on Python rules does not generalize to other languages without language-specific rule engineering (P3 from paper)
Simplification of nested ternaries reduces readability below prompt-based baselines (human eval Comprehensibility)

Implementation pitfalls (do NOT do this):
Do NOT apply Rule 4 (ternary) when if/else blocks contain more than one expression — breaks AST invariant
Do NOT apply Rule 6 (comprehension) when for-loop body has side effects, break, continue, or multiple statements
Do NOT use prompt-engineering (NoTuning_rules_examples) as a substitute — total tokens 1745.26 vs 275.88 for fine-tuned
Do NOT fine-tune more than 5 epochs on ShorterCodeBench (828 pairs is small; overfitting risk is real)
Do NOT skip AST-equivalence check after applying rules — silent correctness failures possible
Do NOT expand LoRA target_modules beyond q_proj/v_proj without ablation — paper only validates this configuration
Do NOT use SimPy-style AI-oriented grammar as alternative — human eval penalizes readability severely
Do NOT batch ingest all 10 rules simultaneously without checking applicability — some rules are mutually exclusive (Rule 1 and Rule 3 can conflict on compound assignments)

Phase 4B applicability (context-aware file reading):
ShortCoder's core insight maps directly to adaptive code shortening for context windows:
- "Outline threshold": analogous to Rule 6/Rule 4 — when a file exceeds a token budget, apply outline rules
  (show signatures, not bodies) the same way ShortCoder applies simplification rules above a verbosity threshold
- "Context-aware reading": ShortCoder's zero-shot inference demonstrates that fine-tuned models internalize
  when to apply shortening WITHOUT explicit prompting — same principle applies to deciding which files to
  read in full vs outline based on context budget remaining
- "Adaptive shortening": the hybrid rule-based + LLM approach shows the two-tier pattern: deterministic rules
  first (fast, no LLM cost), then LLM-guided for cases rules cannot handle — mirrors outline-first, expand-on-demand

Quantitative baselines for Phase 4B design targets:
ShortCoder achieves 18.1%-37.8% token reduction via generation-phase optimization
For prompt/context phase (Phase 4B target): analogous reductions achievable via outline+expand strategy
Cost comparison: fine-tuned=1.20 Cost/Problem vs prompt-based=2.46-3.32; 25%-64% latency reduction
Token budget: 275.88 TotalTokens (ShortCoder) vs 1745.26 (NoTuning_rules_examples) — 6.3x ratio
This 6.3x ratio is the target upper bound for context savings from intelligent file reading vs naive full-read

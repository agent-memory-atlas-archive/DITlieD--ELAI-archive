Algorithm: Self-Correction Bench (Blind-Spot Rate Measurement)
ArXiv: 2507.02778
Paper: Self-Correction Bench: Uncovering and Addressing the Self-Correction Blind Spot in Large Language Models
Version: v2 (October 2025, revision of July 2025 v1)
Scope tag: D-13_adversary_role_cross_family

Summary:
Benchmark measuring the "Self-Correction Blind Spot" in LLMs -- the systematic failure of a model to correct errors in its OWN outputs while successfully correcting identical errors presented in external (user) inputs. Avg blind-spot rate = 64.5% across 14 non-reasoning open-source models. Key negative finding for D-13: the paper provides no cross-family adversary experiment. The blind-spot phenomenon is measured entirely within a single-model context, which is precisely why cross-family critique is required -- the same model cannot reliably critique itself.


Inputs:
response_internal | str | model's own generated response (contains injected error) | position = model turn in chat template
response_external | str | same error injected into user prompt position | identical error content, different chat position
error_type | enum | SCLI5 (simple), GSM8K-SC (multi-step), PRM800K-SC (realistic LLM outputs) | must be same error across internal/external comparison
model | LLM | one of 14 non-reasoning open-source models under test | temperature=0.0, token_budget=1024


Outputs:
blind_spot_rate | float | scalar in [0, 1] | 0 = no blind spot (model self-corrects), 1 = complete blind spot (fails internal, passes external)
accuracy_internal | float | scalar in [0, 1] | P(correct | model_response, error) -- model corrects error in its own output
accuracy_external | float | scalar in [0, 1] | P(correct | user_prompt, error) -- model corrects identical error from user input
macro_accuracy | float | per-model aggregate | average across SCLI5, GSM8K-SC, PRM800K-SC
correction_marker_count | int | count of self-evaluation tokens in completion | proxy for error-correction "activation"


Hyperparameters:
temperature | 0.0 | {0.0, 0.6} | paper uses 0.0 for determinism; Appendix C.1 confirms robustness at 0.6; sensitivity = LOW once deterministic
token_budget | 1024 | 512-2048 | fixed to prevent test-time compute confounds; may under-constrain reasoning models
error_injection_position | chat_template | {model_turn, user_turn} | the critical experimental variable; controls internal vs external measurement
evaluation_judge | Gemini-2.5-flash-preview-05-20 | external LLM | used only for grading correctness, not for providing critique feedback to test models


Critical invariants (testable assertions):
assert accuracy_external > 0, "blind-spot rate undefined when external correction also fails (denominator guard per Eq.2)"
assert 0.0 <= blind_spot_rate <= 1.0, "blind-spot rate is a probability-normalized ratio, bounded [0,1]"
assert error_injected_internal == error_injected_external, "errors must be identical across conditions or measurement is confounded"
assert temperature == 0.0, "non-deterministic sampling inflates variance; paper validates robustness only at 0.0 and 0.6"
assert token_budget == 1024, "varying token budget reintroduces test-time compute variable the benchmark explicitly controls away"
# D-13 design assertion:
assert model_critic != model_under_test, "same-model critique yields 64.5% avg blind-spot rate; cross-family critic required for reliable adversarial signal (Section 5.2, Table 2)"


Complexity:
time | O(N * D * 2) | N models, D dataset items, 2 conditions (internal/external); ~2x inference cost vs standard eval
space | O(D) | dataset storage; no persistent state across items
parallelism | embarrassingly parallel per (model, item, condition) triple | DeepInfra API used in paper; can batch


Failure modes:
Same-model self-critique is unreliable: 64.5% average blind-spot rate means the same model fails to catch its own errors 2 out of 3 times (Section 5.2, Table 2).
Reasoning models are NOT exempt from the metric, but they show near-zero or negative blind-spot rates (Section 6.2, Table 3) -- testing with a reasoning model as self-critic may mask the failure mode in non-reasoning target models.
Empty-response failure: some non-reasoning models generate empty completions when presented with internal errors, signaling "unawareness of error" rather than attempting correction (Section 5, Figure 6).
Fixed token budget (1024) may cut off reasoning chains; models with longer natural correction sequences are penalized.
Controlled error injection creates distribution mismatch for models not trained on that error style (Section 8, Limitations).
Training data bias: non-reasoning SFT datasets (OpenAssistant, OpenHermes2.5) have 95th-percentile correction marker count = 1, meaning models trained on them have near-zero learned correction behavior.


Implementation pitfalls (do NOT do this):
Do NOT use the same model as both the target and the critic in an adversarial evaluation loop -- blind-spot rate of 64.5% proves same-model critique is systematically unreliable.
Do NOT interpret a low blind-spot rate on a reasoning model as evidence that self-critique is generally reliable -- reasoning models are a distinct class; non-reasoning models dominate deployed use.
Do NOT vary token budget between internal and external conditions -- this reintroduces test-time compute confound the benchmark controls away.
Do NOT inject different errors into internal vs external conditions -- comparability requires identical error content; only position (model turn vs user turn) differs.
Do NOT use the "Wait" token trick as a substitute for cross-family critique -- "Wait" reduces blind spots by 89.3% but requires appending the token at inference time and is a prompting hack, not an architectural guarantee.
Do NOT assume a cross-family critic eliminates bias entirely -- the paper shows same-model bias, but does not measure cross-family bias; treat cross-family as necessary but possibly insufficient.
Do NOT use the evaluation judge (Gemini-2.5-flash) as a correction provider -- it is used only for grading ground-truth correctness, not as a feedback source to the model under test.


D-13 grounding summary:
The 64.5% blind-spot rate (Section 5.2, Table 2, Equation 2/13) is the quantitative ground for requiring a cross-family adversary in D-13. The paper demonstrates that same-model critique fails systematically due to training data composition (SFT demos favor error-free outputs, not correction sequences -- Section 6.3, Table 5). A cross-family adversary does not share the target model's internal activation patterns or training distribution, making it the minimum viable design for reliable adversarial critique. The paper does not itself test cross-family adversary performance; that design decision is an inference from the negative finding (same-model = unreliable), not a direct experimental result. Label accordingly: D-13 uses this paper as negative evidence grounding, not positive experimental proof of cross-family superiority.


Dataset sizes:
SCLI5 | 286 items | low complexity | simple answer errors (off-by-one, value flip)
GSM8K-SC | 1313 items | medium complexity | multi-step reasoning errors (problem representation, planning, execution sub-types)
PRM800K-SC | 448 items | high complexity | realistic errors sampled from actual LLM outputs


Models tested (14 non-reasoning, open-source):
Llama-4-Maverick-17B-128E-Instruct-FP8 | best macro avg 0.606
DeepSeek-V3-0324
Qwen2.5-72B-Instruct
Llama-4-Scout-17B-16E-Instruct
Llama-3.3-70B-Instruct
Qwen3-235B-A22B
phi-4
Qwen2.5-7B-Instruct
Qwen2-7B-Instruct
Qwen3-14B
Qwen3-30B-A3B
Qwen3-32B
Llama-3.1-8B-Instruct
Mistral-Small-24B-Instruct-2501 | worst macro avg 0.023

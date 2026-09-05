Algorithm: SeeAct (GPT-4V Web Agent with Grounding)
ArXiv: 2401.01614
Paper: SeeAct: GPT-4V(ision) is a Generalist Web Agent, if Grounded (ICML 2024)
Authors: Boyuan Zheng, Boyu Gou, Jihyung Kil, Huan Sun, Yu Su

Inputs:
screenshot_image | image/PNG | rendered webpage at current timestep | mandatory | captured via Playwright browser automation
html_document | text/HTML | full page HTML at current timestep | mandatory | used for candidate element extraction and textual grounding
task_description | string | natural language goal | mandatory | passed in system prompt unchanged
action_history | list[string] | previous (element, operation, value) triplets | optional | format: "Previous Actions: [action1; action2; ...]", length unspecified by paper
candidate_elements | list[HTML snippets] | top-50 elements from DeBERTa ranker | mandatory for textual-choices grounding | clustered into groups of 17 per VLM call
annotated_screenshot | image/PNG | screenshot with red bounding boxes + white-on-black numeric labels at bottom-left | mandatory for image-annotation grounding | label assignment avoids overlaps

Outputs:
action_triplet | tuple(element_ref, operation, value) | (e, o, v) | element_ref is label number (image-annotation) or text description (textual-choices) | operation in {Click, Type, Select} | value is text string for Type, option text for Select, empty for Click
grounded_browser_event | browser automation call | Playwright-executable | invariant: must correspond to a real DOM element; "NA" output if element not present

Hyperparameters:
vlm_model | GPT-4V (gpt-4-vision-preview) | fixed in paper | not varied
visual_tokens_per_screenshot | ~1445 | paper-measured | not configurable; determined by GPT-4V tokenizer
candidate_elements_per_ranker | 50 | fixed | DeBERTa-base cross-encoder selects top-50 from full DOM
elements_per_vl_group | 17 | fixed | top-50 split into groups of 17 for inference batching
bounding_box_label_style | numerical, bottom-left, white-on-black | ablation-tested best | alternative: letter labels, bottom-center (all inferior)
grounding_method | textual_choices (best) | range: {element_attributes, textual_choices, image_annotation} | textual_choices ~39% step SR vs image_annotation ~20% on cross-task

Critical invariants (testable assertions):
assert operation in {"Click", "Type", "Select"}, "Operation must be one of the three defined action types (Section 3.1)"
assert len(candidate_elements) <= 50, "DeBERTa ranker outputs at most 50 elements (Section 3.2)"
assert len(group) <= 17 for group in element_groups, "VLM receives at most 17 candidate elements per call (Section 3.2)"
assert grounding_method in {"element_attributes", "textual_choices", "image_annotation"}, "Only three grounding strategies defined (Section 3.2)"
assert step_success_rate_textual_choices > step_success_rate_image_annotation, "Textual choices outperforms image annotation (~39% vs ~20% cross-task, Table 3)"
assert oracle_step_sr > any_grounding_step_sr, "Grounding gap: oracle ~62% vs best grounding ~39%; gap is irreducible without better grounding (Table 3)"

Complexity:
Stage 1 (generation): 1 VLM call per step | ~1445 visual tokens + text prompt per call
Stage 2 (grounding, textual-choices): ceil(50/17) = 3 VLM calls per step for ranking | total ~4 VLM API calls per step
Stage 2 (grounding, image-annotation): 1 VLM call with annotated screenshot | single call but inferior accuracy
DeBERTa ranker: O(n_elements) cross-encoder passes per step | runs before VLM to filter DOM
Playwright integration: synchronous browser automation call per action | no parallelism noted

Failure modes:
Visual hallucination: model invents bounding box labels not present in image (54% of annotation errors, Section 4.3)
Spatial reasoning failure: model cannot map bounding box positions to correct numeric labels (46% of annotation errors, Section 4.3)
Grounding gap: 20-30% absolute gap between best automated grounding and oracle human grounding persists across all test splits (Table 3)
Similar/identical elements: textual grounding fails when multiple DOM elements have identical visible text (Section 4.2)
Pop-up interference: dynamic page overlays (ads, consent dialogs) block element access; paper required manual human intervention (Section 5)
HTML scale: raw HTML can be ~186K tokens, requiring candidate filtering via ranker; full HTML is never passed to VLM directly
No retry/fallback: paper defines no automated fallback when grounding fails; step is recorded as failure

Implementation pitfalls (do NOT do this):
Do NOT pass raw full HTML to the VLM — token count (~186K) far exceeds any context window; always pre-filter to top-50 candidates via a ranking model
Do NOT skip the two-stage pipeline (generation then grounding separately) — single-stage action generation without explicit grounding achieves far lower accuracy
Do NOT use letter-based bounding box labels — ablation shows numerical labels at bottom-left outperform all alternatives (Section 4.3)
Do NOT treat image annotation as equivalent to textual choices — image annotation scores ~19pp lower on step success rate (Table 3)
Do NOT assume complete task success rate is high — even best configuration achieves only partial task completion; step SR ~39% does not equal task SR
Do NOT use Stage 1 screenshot-only observation without action history — Previous Actions context is required for sequential decision making (Appendix D prompt format)
Do NOT use heuristic-only grounding (element attributes) without a learned ranker — lowest performing grounding method (~16% cross-task step SR)

Pipeline flow (for Phase 4I implementors):
1. Playwright captures page.screenshot() -> screenshot_image (PNG)
2. Playwright extracts DOM -> html_document
3. DeBERTa cross-encoder ranks all interactive elements -> top-50 candidate_elements
4. VLM Stage 1 call: [task + action_history + screenshot_image] -> textual_action_description
5. VLM Stage 2 call (textual-choices): [textual_action_description + candidate_elements_group_i] x ceil(50/17) -> ranked_choice
6. Selected element + operation + value -> browser event via Playwright
7. Next timestep: repeat from step 1

Benchmark reference:
Dataset: Mind2Web (1009 training tasks, 137 test websites, 31 domains, avg 7.7 actions/task)
Metrics: Element Accuracy, Operation F1, Step Success Rate (macro-avg), Task Success Rate
Best reported Step SR: 39.1% (cross-task, textual choices grounding)
Best reported Task SR: not the primary reported metric in online evaluation (low due to compounding step errors)

Playwright integration notes (verified from paper):
- Playwright used for: page loading, interactive element extraction, action execution
- CDP not explicitly named in paper; Playwright abstracts CDP internally
- page.screenshot() equivalent implied but not named explicitly
- Action conversion: (element, operation, value) triplet -> Playwright locator + click/fill/select_option call
- Human monitors required during online eval to close pop-ups and prevent harmful actions

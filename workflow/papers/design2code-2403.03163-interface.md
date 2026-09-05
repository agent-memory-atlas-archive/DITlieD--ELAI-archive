Algorithm: Design2Code Evaluation Pipeline
ArXiv: 2403.03163
Paper: Design2Code: Benchmarking Multimodal Code Generation for Automated Front-End Engineering
Authors: Chenglei Si, Yanzhe Zhang, Ryan Li, Zhengyuan Yang, Ruibo Liu, Diyi Yang (Stanford/GeorgiaTech/Microsoft/Google DeepMind)
Published: NAACL 2025 (submitted March 2024)

EVALUATION PIPELINE OVERVIEW
Two-tier: (1) high-level visual similarity via CLIP, (2) low-level element matching via block detection + Jonker-Volgenant optimal assignment.
Reference screenshots I_R vs generated screenshots I_G are compared directly after rendering HTML to image.

Inputs:
reference_screenshot | PIL Image or numpy array | H x W x 3 uint8 | screenshot of reference webpage; mandatory
generated_screenshot | PIL Image or numpy array | H x W x 3 uint8 | screenshot of rendered generated HTML; mandatory
reference_html | str | raw HTML text | used only for block detection via color-tracking method; mandatory for low-level metrics
generated_html | str | raw HTML text | used for block detection of generated page; mandatory for low-level metrics
reference_blocks R | list[Block] | each Block has: text str, bbox (x1,y1,x2,y2) normalized [0,1], color RGB | output of text detection; derived
generated_blocks G | list[Block] | same schema as R | derived from generated_html

Block detection method (Appendix C, paper):
- NOT generic OCR (too noisy for stability)
- Color-tracking method: alter CSS text colors uniquely per segment, take two screenshots, track pixel color differences to locate text block bounding boxes without OCR errors
- Requires access to HTML source; generates blocks with exact text content and bbox coordinates
- Block merging: enumerate merging neighbor blocks to handle granularity mismatches (non-contiguous merges NOT supported)

Outputs:
block_match_score | float | scalar in [0.0, 1.0] | size-weighted recall/precision of matched block pairs; higher = better
text_score | float | scalar in [0.0, 1.0] | averaged character-level Sorensen-Dice over matched pairs
position_score | float | scalar in [0.0, 1.0] | averaged normalized center-distance similarity over matched pairs
color_score | float | scalar in [0.0, 1.0] | averaged CIEDE2000 perceptual color similarity over matched pairs
clip_score | float | scalar in [0.0, 1.0] | cosine similarity of CLIP-ViT-B/32 embeddings; text inpainted before encoding

Metric Formulas (exact, from Figure 3 and Section 2.3):

BLOCK-MATCH (Equations 1 and 2):
  Per-pair contribution: match_block(r_p, g_q) = (S(r_p) + S(g_q)) / (sum_{(i,j) in M}(S(r_i)+S(g_j)) + sum_{i in U_R} S(r_i) + sum_{j in U_G} S(g_j))
  Aggregate: match_block(R, G) = sum_{(p,q) in M} match_block(r_p, g_q)
  S(block) = block area in pixels (width * height of bounding box)
  U_R = unmatched blocks in reference; U_G = unmatched blocks in generated
  Semantics: penalizes both missing elements (U_R) and hallucinated elements (U_G); larger unmatched blocks = heavier penalty

TEXT (Section 2.3):
  sim_text(r_p, g_q) = 2 * |overlapping_chars| / (|chars_r_p| + |chars_g_q|)  [character-level Sorensen-Dice]
  text_score = mean over all matched pairs (p,q) in M

POSITION (Section 2.3):
  sim_pos(r_p, g_q) = 1 - max(|x_q - x_p|, |y_q - y_p|)
  (x_p, y_p) = normalized center coordinates of r_p in [0,1]
  (x_q, y_q) = normalized center coordinates of g_q in [0,1]
  position_score = mean over all matched pairs (p,q) in M

COLOR (Section 2.3):
  sim_color(r_p, g_q) = f(CIEDE2000(color_rp, color_gq))  [perceptual color difference; Luo et al. 2001]
  color_score = mean over all matched pairs (p,q) in M

CLIP (Section 2.3):
  Model: CLIP-ViT-B/32 (Radford et al. 2021)
  Preprocessing: resize screenshots to squares
  Pre-processing step: inpaint all detected text boxes using Telea (2004) fast-marching inpainting (OpenCV) BEFORE encoding
  Score: cosine similarity between CLIP embeddings of inpainted I_R and inpainted I_G

MATCHING ALGORITHM (Appendix C):
  Algorithm: Jonker-Volgenant (Crouse 2016), implemented via scipy.optimize.linear_sum_assignment
  Cost matrix: negative Sorensen-Dice text similarity (-sim_text) between all (r_i, g_j) pairs
  Threshold: matched pairs with sim_text < 0.5 are IGNORED (excluded from M)
  Block merging: neighbor blocks are enumerated for merging to handle granularity differences; non-contiguous merges not supported

Hyperparameters:
text_match_threshold | 0.5 | [0.0, 1.0] | pairs below this Sorensen-Dice cutoff are excluded from matching; hard-coded in paper
clip_model | CLIP-ViT-B/32 | fixed | no alternative mentioned; openai/clip-vit-base-patch32
inpainting_algorithm | Telea 2004 fast-marching | fixed | OpenCV implementation; ref: docs.opencv.org/4.3.0/df/d3d/tutorial_py_inpainting.html
screenshot_resize | square resize | fixed | exact dimension not specified in paper
block_detection_method | color-tracking via HTML color alteration | fixed | requires HTML source access; not applicable to arbitrary images

Observed score ranges (Table 1, Table 3, full benchmark 484 examples):
block_match: 12.3 (worst: Idefics2-8B self-revision) to 93.0 (best: GPT-4o direct)
text: 22.6 to 98.6  [NOTE: scores reported as percentages 0-100 in tables but are [0,1] floats internally]
position: 13.2 to 85.5
color: 14.5 to 84.1
clip: 78.4 to 90.4
Design2Code-HARD scores are ~10-30 points lower on block_match; GPT-4o best: Block=56.6, Text=89.8, Position=78.6, Color=81.9, CLIP=87.1

Human-metric correlation (Table 2 logistic regression, predicting Win/Lose):
Position: coef=0.7605, p<0.001 (most predictive of human preference)
Block-Match: coef=0.7429, p<0.001
CLIP: coef=0.4929, p<0.001
Color: coef=0.3461, p=0.001
Text: coef=-0.3541, p=0.021 (NEGATIVE association -- humans don't weight text content highly)
Logistic regression accuracy: 79.9% on held-out 50% split

Critical invariants (testable assertions):
assert 0.0 <= block_match_score <= 1.0, "block_match out of range"
assert block_match_score == 0.0 or len(matched_pairs) > 0, "nonzero block_match requires at least one matched pair"
assert all(sim_text(r, g) >= 0.5 for r, g in matched_pairs), "matched pairs must exceed 0.5 threshold"
assert abs(sum(match_block(r_p, g_q) for r,g in matched_pairs) - block_match_score) < 1e-6, "block_match must equal sum of per-pair contributions"
assert all(0.0 <= sim_pos(r, g) <= 1.0 for r, g in matched_pairs), "position similarity must be in [0,1]"
assert clip_screenshots_are_inpainted_before_encoding, "CLIP must NOT encode raw screenshots; text must be inpainted first"
assert block_detection_uses_color_tracking_not_ocr, "use HTML color-alteration method, not generic OCR"

Complexity:
Block detection: O(N_segments * screenshot_render_cost); requires 2 extra renders per webpage pair
Matching (Jonker-Volgenant): O(m * n * min(m,n)) where m=|R|, n=|G|; scipy linear_sum_assignment
Block merging search: O(k^2) per merge step where k = number of neighbor candidate blocks; bounded in practice
CLIP inference: O(1) per pair after resize; ViT-B/32 forward pass ~10ms on GPU
Full evaluation of 484 examples: up to 1 hour with multiprocessing (per README)
Space: O(m*n) for cost matrix

Failure modes (from paper):
Models fail most on block-match (missing/hallucinated elements) and position -- not text
Self-revision improves block-match and position for GPT-4V and Claude 3 but NOT for Gemini or open-source models
Non-English webpages degrade all metrics significantly (limited pretraining data)
Long webpages (>500 HTML tags) cause 30-40% block-match failure even for GPT-4o
Open-source models without finetuning lag severely on block-match (50.4 vs 93.0 for GPT-4o)
Idefics2-8B degrades catastrophically with self-revision (Block: 46.7 -> 12.3) -- self-revision can hurt weak models
DeepSeek-VL-7B self-revision: Block drops 66.1 -> 30.1 (confirmed failure case)
Text-augmented prompting helps block-match and text but sometimes hurts position and color
Human evaluators systematically underweight text accuracy (negative regression coefficient) -- do not use human preference as ground truth for text fidelity

Implementation pitfalls (do NOT do this):
Do NOT use generic OCR (e.g., Tesseract, EasyOCR) for block detection -- paper explicitly warns these produce noisy outputs that destabilize evaluation
Do NOT skip text inpainting before CLIP encoding -- paper explicitly uses Telea inpainting to "rule out texts in screenshots"
Do NOT use a single aggregated score -- paper treats five metrics as independent diagnostic dimensions; aggregating discards information
Do NOT trust text similarity as a proxy for overall quality -- regression shows negative human correlation
Do NOT use pixel-level SSIM as a substitute for block-match -- SSIM is never mentioned in the paper; Design2Code uses block-based element matching, not SSIM
Do NOT use arbitrary screenshot dimensions -- resize to square before CLIP encoding
Do NOT apply self-revision blindly to weak models -- documented catastrophic failure mode for Idefics2 and DeepSeek-VL
Do NOT use text-based metrics (BLEU, edit distance) for HTML code -- paper explicitly rejects these as unable to capture visual correctness
Do NOT merge non-contiguous text blocks -- implementation only supports contiguous neighbor merging
Do NOT apply this evaluation to dynamic/interactive webpages -- benchmark and metrics cover only static visual rendering

SSIM NOTE (for B-24 Phase 4I.8):
SSIM is NOT used by Design2Code. The paper uses: CLIP-ViT-B/32 (high-level) + block-match/text/position/color (low-level). Any reference to "SSIM" in Phase 4I.8 is not grounded in this paper. If SSIM is desired, it would need separate grounding.

Source code: github.com/NoviScl/Design2Code, metrics/multi_processing_eval.py
Benchmark: 484 real-world webpages from C4 validation set + 80 HARD examples from GitHub Pages

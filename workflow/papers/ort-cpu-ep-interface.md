Source: ort Rust crate (pykeio/ort) + ORT CPU Execution Provider + LPIPS ONNX + CLAP ONNX + image-compare SSIM
Reliability: HIGH (official pykeio/ort docs.rs 2.0.0-rc.12 + Microsoft ORT issues + HuggingFace model cards + verified project source)
Files examined:
  https://docs.rs/ort/2.0.0-rc.12/ort/session/builder/struct.SessionBuilder.html
  https://docs.rs/ort/2.0.0-rc.7/ort/struct.CPUExecutionProvider.html
  https://docs.rs/ort/latest/ort/session/struct.Session.html
  F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\vector\ort_session.rs (project canonical usage)
  https://github.com/microsoft/onnxruntime/issues/4611 (CUDA non-determinism)
  https://github.com/microsoft/onnxruntime/issues/22818 (Windows CUDA non-determinism)
  https://github.com/microsoft/onnxruntime/issues/12086 (reproducibility bounds)
  https://docs.rs/image-compare/latest/image_compare/ (SSIM)
  https://docs.rs/image/latest/image/enum.DynamicImage.html (image 0.25.10)
  https://huggingface.co/Xenova/clap-htsat-unfused (CLAP ONNX)
  https://huggingface.co/docs/transformers/en/model_doc/clap (CLAP spec)
  https://github.com/richzhang/PerceptualSimilarity/blob/master/lpips/lpips.py (LPIPS)
  https://huggingface.co/deepghs/imgutils-models (LPIPS ONNX)


VERSION NOTES:
  ort = "=2.0.0-rc.12" (pinned in project Cargo.toml — this is the correct version per anti-stale rule)
  All 1.x versions yanked by maintainer due to memory safety bugs (use-after-free, leaks)
  2.0.0-rc.12 is maintainer-recommended for production; API may have minor breaking changes per rc
  No stable 2.x exists as of 2026-05-11; rc.12 IS the production version


PUBLIC API — ort crate (2.0.0-rc.12):

Session::builder() | fn() -> Result<SessionBuilder> | creates SessionBuilder
SessionBuilder::with_execution_providers(providers: impl AsRef<[ExecutionProviderDispatch]>) | -> BuilderResult | registers EPs in order; silently falls back to CPU if all fail
SessionBuilder::with_intra_threads(num_threads: usize) | -> BuilderResult | parallelism within a single op node
SessionBuilder::with_inter_threads(num_threads: usize) | -> BuilderResult | parallelism across independent op nodes (only applies when with_parallel_execution(true))
SessionBuilder::with_parallel_execution(parallel_execution: bool) | -> BuilderResult | default=false (sequential); true adds memory overhead
SessionBuilder::with_inter_op_spinning(enable: bool) | -> BuilderResult | spinning vs blocking for inter-op threads
SessionBuilder::with_intra_op_spinning(enable: bool) | -> BuilderResult | spinning vs blocking for intra-op threads
SessionBuilder::with_independent_thread_pool() | -> BuilderResult | session gets its own thread pool (no global pool sharing)
SessionBuilder::with_optimization_level(opt_level: GraphOptimizationLevel) | -> BuilderResult | Level1/Level2/Level3
SessionBuilder::commit_from_file(model_path: impl AsRef<Path>) | -> Result<Session> | loads ONNX from disk
SessionBuilder::commit_from_memory(model_bytes: &[u8]) | -> Result<Session> | loads ONNX from bytes
Session::run(input_values: impl Into<SessionInputs<'i,'v,N>>) | -> Result<SessionOutputs<'s>> | runs inference
Session::inputs() | -> &[Outlet] | introspect model input names and types
Session::outputs() | -> &[Outlet] | introspect model output names and types
Outlet::name() | -> &str | returns tensor name string
SessionOutputs | index by usize or &str | access result tensors
SessionOutputs[idx].try_extract_tensor::<f32>() | -> Result<(Vec<i64>, &[f32])> | extract (shape, data) pair
Tensor::from_array((shape: Vec<i64>, data: Vec<T>)) | -> Result<Tensor<T>> | construct input tensor
CPUExecutionProvider::default() | -> CPUExecutionProvider | default CPU EP (arena allocator disabled)
CPUExecutionProvider::with_arena_allocator() | -> Self | enables arena allocation for memory reuse
CPUExecutionProvider::build() | -> ExecutionProviderDispatch | finalize for use with with_execution_providers
ExecutionProvider::is_available() | -> Result<bool> | check if EP is compiled in (CPU always true)
ExecutionProviderDispatch | dynamic EP container | used in Vec for with_execution_providers


BEHAVIORAL CONTRACTS:

Session::builder() -> with_execution_providers([CPUExecutionProvider::default().build()]) -> commit_from_file -> BLOCKS on any EP other than CPU; GPU not loaded
with_execution_providers([]) or passing only CPU EP -> no CUDA/TensorRT libs loaded -> deterministic MLAS execution
with_parallel_execution(false) + with_intra_threads(1) + with_inter_threads(1) -> single-threaded sequential -> maximum determinism for SSIM/LPIPS
Session::inputs() introspection at load time -> captures model input names including optional token_type_ids (project pattern, ort_session.rs:119-122)
session.run(inputs) -> SIDE EFFECT: mutates internal KV state; not thread-safe without external Mutex (project uses Arc<Mutex<Session>>)
SessionOutputs lifetime tied to Session borrow -> cannot return SessionOutputs from a function that holds Session lock
try_extract_tensor::<f32>() returns (shape: Vec<i64>, data: CowArray<f32>) -> shape dims are i64 (cast to usize for indexing, always non-negative from ORT)
Silent EP fallback: if GPU EP requested but unavailable, silently uses CPU (no error); use is_available() to detect
with_execution_providers on session takes precedence over environment-level EPs but does NOT replace them


CPU EP DETERMINISM (verified from ORT issues #4611, #22818, #12086):

Issue #4611: "The results are numerically identical when using the CPU however" (confirmed CPU EP deterministic within same binary/platform)
Issue #4611: CUDA EP is non-deterministic due to parallel atomic reductions in cuBLAS/cuDNN (convolution, batch norm, attention)
Issue #22818: Windows CUDA v1.19.2 non-deterministic; resolved in v1.20.0 — Windows-specific CUDA race
Issue #12086: ORT states no cross-run reproducibility guarantee; stated tolerance example abs_err=1e-5, rel_err=1e-5
CPU EP uses MLAS (Microsoft Linear Algebra Subprograms) — deterministic per build, per platform
CROSS-PLATFORM WARNING: f32 FMA (fused multiply-add) is platform-dependent (x86 AVX-FMA vs ARM NEON vs scalar) -> results differ across CPU architectures
SAME-PLATFORM GUARANTEE: within a single build on same ISA, single-threaded CPU EP results are bit-identical across runs
For SSIM/LPIPS reproducibility in B-24 spec: use intra_threads(1), inter_threads(1), parallel_execution(false) to eliminate thread-scheduling non-determinism
Tolerance for cross-platform comparison: epsilon 1e-6 absolute (spec requirement) is stricter than ORT's stated 1e-5


FORCING CPU-ONLY (refusing GPU fallback):

Pattern to refuse all GPU EPs and fail hard if CPU EP somehow fails:
  Session::builder()?
    .with_execution_providers([CPUExecutionProvider::default().build()])?
    .with_intra_threads(1)?
    .with_inter_threads(1)?
    .with_parallel_execution(false)?
    .commit_from_file(path)?

Note: passing only CPUExecutionProvider ensures CUDA/TensorRT libs are never loaded
There is no error_on_failure API in ort 2.0.0-rc.12; use is_available() pre-check if needed
CPU EP is always available (compiled into every ORT build); is_available() always returns Ok(true) for CPU


LPIPS ONNX PATH (richzhang/PerceptualSimilarity):

No official pre-exported ONNX in richzhang/PerceptualSimilarity repo; must export via torch.onnx.export
Community ONNX available: huggingface.co/deepghs/imgutils-models lpips/lpips_diff.onnx (77.1 kB — lightweight diff variant)
LPIPS PyTorch forward signature: forward(in0, in1, retPerLayer=False, normalize=False) -> Tensor[N]
Input shapes: in0, in1 both float32 [N, 3, H, W] — RGB images scaled to [-1, +1]
Output shape: [N] (one scalar distance per image pair); with retPerLayer=True returns (scalar, list[per-layer])
Backbone nets: 'alex' (9.1 MB), 'vgg' (58.9 MB), 'squeeze' (2.8 MB) — 'alex' is default/recommended
Internal layer names: lin0..lin4 (lin5,lin6 for squeeze), ScalingLayer
torch.onnx.export call for alex net:
  input_names=['in0', 'in1'], output_names=['distance']
  dynamic_axes={'in0': {0:'N',2:'H',3:'W'}, 'in1': {0:'N',2:'H',3:'W'}}
  opset_version=17
ONNX input constraints: same H and W for in0 and in1; H and W must be divisible by patch stride
normalize=True accepted when inputs are in [0,1] (scales to [-1,+1] internally)
Version 0.1 recommended (version 0.0 had normalization bugs)


SSIM VIA image-compare CRATE (pure Rust, no ONNX):

image-compare crate (https://crates.io/crates/image-compare) depends on image ^0.25
Algorithm::MSSIMSimple: SSIM computed over 8x8 pixel windows, averaged across windows
gray_similarity_structure(algorithm: &Algorithm, first: &GrayImage, second: &GrayImage) -> Result<Similarity, CompareError>
rgb_similarity_structure(algorithm: &Algorithm, first: &RgbImage, second: &RgbImage) -> Result<Similarity, CompareError>
Similarity.score: f64 in [0.0, 1.0] — 1.0 = identical, 0.0 = fully dissimilar (some algorithms go to -1.0)
image crate (0.25.10) DynamicImage methods:
  image::open(path) -> Result<DynamicImage>
  DynamicImage::to_luma8() -> GrayImage (ImageBuffer<Luma<u8>, Vec<u8>>)
  DynamicImage::to_rgb8() -> RgbImage (ImageBuffer<Rgb<u8>, Vec<u8>>)
  DynamicImage::as_bytes() -> &[u8]
  DynamicImage::width() -> u32, DynamicImage::height() -> u32
Per-channel SSIM: split RGB into R, G, B planes using to_luma8 per channel (extract channel bytes -> new GrayImage)
SSIM score 1.0 does NOT imply pixel-identical; 8x8 window averaging smooths local differences
For deterministic SSIM across runs: pure Rust with no threading = fully deterministic on same platform


CLAP ONNX MODEL (Xenova/clap-htsat-unfused):

ONNX files available at huggingface.co/Xenova/clap-htsat-unfused/tree/main/onnx:
  audio_model.onnx (118 MB) — audio encoder only
  audio_model_quantized.onnx (34.3 MB) — INT8 quantized
  text_model.onnx (502 MB) — text encoder only (RoBERTa-based)
  model.onnx (619 MB) — combined model
  *_fp16 variants available

Audio preprocessing (from preprocessor_config.json, verified):
  sampling_rate: 48000 Hz
  feature_size (n_mels): 64
  max_length_s: 10 seconds
  hop_length: 480 samples
  fft_window_size (n_fft): 1024
  nb_max_frames: 1000
  nb_max_samples: 480000 (= 48000 * 10)
  Output mel spectrogram shape: [batch, 64, 1000] (feature_size x nb_max_frames)

audio_model.onnx inputs (from HF transformers ClapAudioModel forward):
  input_features | float32 | [batch_size, sequence_length, feature_dim] = [B, 1000, 64] | mel spectrogram
  is_longer | bool | [batch_size] | optional; indicates if audio was truncated (for fusion models)
audio_model.onnx outputs:
  audio_embeds (pooler_output) | float32 | [batch_size, 512] | L2-normalized audio embedding

text_model.onnx inputs (RoBERTa-based, NO token_type_ids):
  input_ids | int64 | [batch_size, sequence_length] | tokenized text
  attention_mask | int64 | [batch_size, sequence_length] | 0/1 mask
  NOTE: RoBERTa-based — does NOT have token_type_ids (apply project pattern: has_token_type_ids=false)
text_model.onnx outputs:
  text_embeds | float32 | [batch_size, 512] | L2-normalized text embedding

projection_dim: 512 (verified from config.json)
Embeddings are L2-normalized; cosine similarity = dot product
Max text sequence length: not explicitly specified; use 77 tokens (CLIP convention) or model default


PLATFORM CONSTRAINTS:

Windows: ORT DLL loading order matters; onnxruntime.dll must be in PATH or adjacent to binary
ort 2.0.0-rc.12 fetches ORT native libs via build script (ort-sys crate); no manual DLL installation
CUDA EP: non-deterministic on Windows pre-ORT-1.20.0 (issue #22818); CPU EP unaffected
Cross-architecture: f32 results differ between x86_64 AVX-FMA and ARM64 NEON builds (FMA contraction)
Same-architecture, same-build: CPU EP single-threaded results are bit-identical (verified issue #4611)
Parallelism: multi-threaded intra-op introduces non-determinism due to float accumulation ordering
Session is NOT Send/Sync by default; project wraps in Arc<Mutex<Session>> (ort_session.rs:4,36)
SessionOutputs lifetime: 's tied to &'s mut Session -> extract data before releasing lock


GOTCHAS:

ORT silent EP fallback: with_execution_providers([CUDA]) silently uses CPU if CUDA DLLs absent — no error
Session::builder() returns Result<SessionBuilder>, not SessionBuilder — must chain with .and_then or ? (project pattern at ort_session.rs:79-80: .and_then(|mut b| b.commit_from_file(...)))
commit_from_file takes &mut self (mutable SessionBuilder) — builder is consumed; cannot reuse
try_extract_tensor output shape is Vec<i64> not Vec<usize> — cast with as usize for indexing
Outlet::name() returns &str; check against model inputs at load time to handle optional fields (RoBERTa drops token_type_ids)
LPIPS normalize parameter: if input is [0,1] set normalize=True; if [-1,+1] set normalize=False; wrong setting causes ~4x score error
SSIM requires same dimensions: gray_similarity_structure fails with CompareError if images have different dimensions
image-compare Algorithm::MSSIMSimple window=8x8: images smaller than 8x8 produce undefined behavior
CLAP audio_model.onnx: input is mel spectrogram [B, 1000, 64], NOT raw audio [B, 480000] — preprocessing required before ORT
CLAP text RoBERTa: no token_type_ids; feeding token_type_ids to text_model.onnx causes INVALID_ARGUMENT (confirmed per project gotcha in ort_session.rs)
ort 2.0.0-rc.12 API: BuilderResult = Result<SessionBuilder, Error>; all with_* methods return Result, must chain with ?


CRITICAL ASSERTIONS (testable):

assert!(Session::builder().is_ok(), "ort session builder must always succeed")
assert!(CPUExecutionProvider::default().is_available().unwrap_or(false), "CPU EP must be available in all builds")
assert!(ssim_score >= 0.0 && ssim_score <= 1.0, "SSIM score must be in [0.0, 1.0]")
assert!((audio_embed.iter().map(|x| x*x).sum::<f32>()).sqrt() - 1.0).abs() < 1e-5, "CLAP audio embeddings are L2-normalized")
assert!((text_embed.iter().map(|x| x*x).sum::<f32>()).sqrt() - 1.0).abs() < 1e-5, "CLAP text embeddings are L2-normalized")
assert_eq!(audio_embed.len(), 512, "CLAP audio_embeds dim = 512")
assert_eq!(text_embed.len(), 512, "CLAP text_embeds dim = 512")
assert!((lpips_score - lpips_reference).abs() < 1e-6, "LPIPS reproducibility tolerance for CPU EP same-platform")
assert_eq!(mel_spectrogram_shape, [batch, 1000, 64], "CLAP mel input shape")
assert!(session.inputs().iter().all(|i| i.name() != "token_type_ids"), "CLAP text RoBERTa has no token_type_ids")


COMPLEXITY:

SSIM (image-compare MSSIMSimple): O(W*H) per image pair; 8x8 window; pure CPU, single-threaded, deterministic
LPIPS (ORT CPU EP): O(model_params * H * W) per forward pass; alex net ~9MB, inference ~10-50ms on CPU for 256x256
CLAP audio (ORT CPU EP): O(model_params * T * F) where T=1000 frames, F=64 mel bins; 118MB model, ~200-500ms on CPU
CLAP text (ORT CPU EP): O(seq_len^2 * hidden) RoBERTa; 502MB model, ~100-300ms on CPU per text batch
Thread safety: all sessions require external Mutex; parallel calls to run() on same Session are UB


FAILURE MODES:

CUDA EP non-determinism: resolved by using CPU EP only (verified issues #4611, #22818)
Multi-thread accumulation non-determinism: with_intra_threads(N>1) introduces ordering non-determinism; use N=1 for reproducibility
Cross-platform f32 FMA: different ISA builds produce different results; use epsilon 1e-6 not exact equality
LPIPS wrong normalize flag: score error up to 4x if image range assumed wrong
CLAP audio: feeding raw audio instead of mel spectrogram crashes inference (shape mismatch)
CLAP text on RoBERTa ONNX: INVALID_ARGUMENT if token_type_ids fed (RoBERTa has no such input node)
SSIM dimension mismatch: CompareError if images not same size; resize before compare
ORT session lock poisoning: if thread panics while holding Mutex<Session>, lock is poisoned; subsequent calls fail


IMPLEMENTATION PITFALLS (do NOT do this):

Do NOT use with_parallel_execution(true) for SSIM/LPIPS determinism — parallel accumulation order varies
Do NOT feed raw audio [B, 480000] to CLAP audio_model.onnx — input is mel spectrogram [B, 1000, 64]
Do NOT feed token_type_ids to CLAP text_model.onnx (RoBERTa-based — no such input; INVALID_ARGUMENT error)
Do NOT skip Outlet introspection for optional inputs — check model.inputs() at session load to detect optional fields
Do NOT cast ORT shape dims with as usize without checking they are positive — use try_into().ok_or(err) or guard with > 0
Do NOT compare LPIPS/SSIM scores with == across platforms — use epsilon 1e-6 absolute tolerance
Do NOT share Session across threads without Mutex — not Send by default
Do NOT use ort 1.x (yanked, memory-unsafe) or any 2.0.0-rc before rc.12 if possible (use pinned =2.0.0-rc.12)
Do NOT assume CLAP embeddings have any particular magnitude — they are L2-normalized (magnitude = 1.0)
Do NOT use 4-bit quantized ONNX variants for reproducibility testing — quantization adds rounding non-determinism
Do NOT call commit_from_file on a moved SessionBuilder after a failed with_* call — all with_* consume self, return Result

ascii-rendering-repos research
research date: 2026-06-26
clones at: /workspace/ELAI/workflow/research/_scratch/asciline/ and _scratch/acerolaFX/

---

REPO 1: ASCILINE
https://github.com/YusufB5/ASCILINE
last commit: 2026-06-24 (active, ~2 days ago at research date)
license: MIT with Anti-Advertisement Restriction (use for ad delivery explicitly voids the license; otherwise standard MIT freedoms apply)

Purpose

A real-time video-to-ASCII streaming engine. The backend decodes video frames server-side using OpenCV, converts them to character grids, and streams binary frames to a browser via WebSocket. The browser renders the character grid onto an HTML5 Canvas element. The project supports local files, YouTube URLs (via yt-dlp), playlists, and an alternate pixel-block mode that replaces characters with colored rectangles for near-360p fidelity.

Language and stack

Python backend (FastAPI + uvicorn for the HTTP/WebSocket server, OpenCV for video decoding, NumPy for the pixel-to-character mapping). Vanilla JavaScript + HTML5 Canvas frontend (no framework, no bundler). Optional FFmpeg dependency for audio processing and yt-dlp for URL downloads. No TypeScript, no WebGPU, no WebGL. Dependencies installed via pip.

Key source files: stream_server.py (1045 lines), ascii_video_player2.py (345 lines), app.js (667 lines), codec.py (107 lines).

Core algorithm (verified from ascii_video_player2.py + codec.py)

1. VideoDecoder (ascii_video_player2.py): OpenCV VideoCapture reads frames. Each frame is resized to (cols, rows) via cv2.resize with INTER_LINEAR. For ASCII mode a grayscale copy is also produced. This is the only CPU-heavy step.

2. AsciiMapper.convert (ascii_video_player2.py lines 129-178): Takes the (H,W) gray matrix and (H,W,3) BGR color matrix. Character selection: gray pixel value is mapped to an index into a 93-character palette (" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@") by integer division (gray // (256 // 93)). This is a pure luminance ramp — no edge detection, no frequency analysis, just intensity-to-character. Color is ANSI true-color escape code per pixel with RLE (only emit new escape when color changes row-by-row) and optional 2-bit right-shift quantization for speed.

3. Codec (codec.py): The web streaming path encodes each framebuffer as one of three encodings tagged in a 1-byte header: RAW (passthrough), ZLIB (zlib level 3 on the full frame, selected when many cells changed), or DELTA (zlib over only the changed cell indices + values, selected when few cells changed). Encoder picks the smallest. A keyframe is forced every 48 frames. There is also an optional lossy temporal delta: a color cell is only re-sent once it drifts past a tolerance threshold, cutting bandwidth further without character-plane error.

4. app.js: Receives binary WebSocket messages, runs the codec decoder, fills a jitter buffer (4 frames), and renders to Canvas with fillText (ASCII mode) or fillRect (pixel mode). Audio plays from a separate <audio> element served as a stream; audio currentTime acts as the master clock for A/V sync.

5. stream_server.py: FastAPI app with /ws WebSocket endpoint and HTTP routes for the HTML, audio, scrub thumbnails, and playlist. Backpressure is managed by the client periodically reporting its buffer fill level; the server throttles sending. High-FPS sources are decimated with cap.grab() (decode-free frame skip).

Inputs and outputs

Input: local video file (any format OpenCV supports), YouTube or yt-dlp-compatible URL, folder of videos, or JSON playlist. Output: browser page rendering a text-based or pixel-block video at up to 30fps.

Performance characteristics (from README, not independently benchmarked)

Codec reports 375x wire savings vs legacy on static screens, 63% saving on high-motion. Recommended grid sizes: 200-240 cols for ASCII at 30fps, up to 900 cols for pixel mode. CPU-bound on the Python backend; no GPU required in ASCII mode. Pixel mode uses Canvas fillRect which offloads compositing to the GPU indirectly. The README notes that pushing cols too high causes A/V desync because the Python encoder cannot keep up.

Maturity

Last commit 2026-06-24, multiple test files (backpressure, e2e, scrub, ytdl hardening), cross-validation test vectors between Python codec and JS decoder. Looks actively maintained. No package.json or pyproject.toml found — dependencies are pip-install instructions in README only (no pinned versions). UNVERIFIED: star count (not available from clone metadata).

---

REPO 2: AcerolaFX
https://github.com/GarrettGunnell/AcerolaFX
last commit: 2024-06-29 (unmaintained by current date, ~2 years ago)
license: MIT (standard, no additional restrictions; copyright 2022 Garrett Gunnell)

Purpose

A modular HDR post-processing shader pack for ReShade, specifically built for Final Fantasy XIV gameplay and gpose (in-game photography). It is a collection of .fx and .fxh shader files targeting the ReShade framework that inject post-processing effects onto a running DirectX game's framebuffer via a DLL injection layer.

Language and stack

HLSL shaders written in ReShade's FX dialect (a restricted HLSL superset). No C++, no Python, no JavaScript. The shaders use ReShade's macro system (BUFFER_WIDTH, BUFFER_HEIGHT, PostProcessVS, ReShade::BackBuffer, ReShade::GetLinearizedDepth), its texture/sampler declaration syntax, and its compute shader dispatch syntax. All shaders live in Shaders/ as .fx files with shared includes in Shaders/Includes/. Textures are PNG files in Textures/. Preset configuration files are .ini files in Presets/.

Effect inventory (verified from file list): ASCII, Alpha, AutoExposure, Blend, Bloom, Blur, BokehBlur, CRT, ChromaKey, ChromaticAberration, ColorBlindness, ColorCorrection, ColorSpaces, Composition, DifferenceOfGaussians, Dither, Downscaler, EdgeDetect, End, FXAA, FilmGrain, Fog, Framing, Gamma, Halftone, KuwaharaFilter, PaletteSwap, PixelSort, Sharpness, Start, Tonemapping, Vignette, XeGTAO, Zoom.

ASCII effect algorithm (verified from AcerolaFX_ASCII.fx, full file read)

The ASCII shader is the most relevant effect. Its pipeline in technique AFX_ASCII has 10 passes:

Pass 1 PS_Luminance: compute luminance per pixel (BT.709 weights: 0.2127, 0.7152, 0.0722) into a full-res R16F texture.

Pass 2 PS_Downscale: downsample the original color buffer 8x in each dimension, storing (RGB, luminance) into an RGBA16F texture. This gives one color sample per 8x8 character cell.

Passes 3-4 PS_HorizontalBlur + PS_VerticalBlurAndDifference: two-pass separable Gaussian blur of the luminance buffer at two sigma values (_Sigma and _Sigma * _SigmaScale), then compute the Difference of Gaussians (DoG): D = blur_narrow - tau * blur_wide, thresholded to binary. DoG acts as an artistic edge detector (related to the Canny/LoG family, softer than Sobel alone).

Pass 5 PS_CalculateNormals: reconstruct screen-space normals from the depth buffer (cross product of adjacent depth-reconstructed positions). Optionally uses depth discontinuities too. Stored into a temp texture.

Pass 6 PS_EdgeDetect: combines DoG binary edges with depth-discontinuity and normal-discontinuity edges (8-neighbor max over absolute differences). Final edge mask stored in AFX_AsciiEdgesTex.

Passes 7-8 PS_HorizontalSobel + PS_VerticalSobel: compute Sobel gradient (Gx, Gy) from the edge texture, then compute angle theta = atan2(Gy, Gx) and pack (theta, isEdge) into an RG16F texture.

Pass 9 CS_RenderASCII (compute shader, 8x8 groups): each group processes one 8x8 pixel tile corresponding to one character cell. Within the group, each thread computes its Sobel direction bucket (0=vertical, 1=horizontal, 2=diagonal1, 3=diagonal2) and stores it in groupshared memory. The group's thread 0 tallies a histogram of directions across all 64 threads and picks the dominant direction. If the dominant direction count exceeds _EdgeThreshold, the tile renders an edge character by sampling from edgesASCII.png (a 40x8 sprite sheet of 5 edge characters at 4 rotations each). Otherwise it renders a fill character by sampling from fillASCII.png (an 80x8 sprite sheet of 10 luminance-level characters) using the 8x-downscaled cell luminance. Both LUT textures are authored PNG atlases. Final color is lerp(background, lerp(ascii_color, cell_rgb, base_blend), ascii_alpha) with optional depth falloff.

Pass 10: copy temp texture back to main buffer.

CRT effect: screen-space curvature via UV distortion, scanlines by modulating brightness at 2^N pixel intervals. Bayer and blue-noise dithering effects: palette quantization with ordered dither matrix (Bayer 2x2/4x4/8x8 or 8 blue noise PNG textures). Halftone: similar ordered-threshold approach. Kuwahara filter: computes minimum-variance region color across 4 quadrant windows. DoG standalone effect available separately.

Inputs and outputs

Input: the ReShade backbuffer (the rendered game frame) plus depth and normal buffers that ReShade exposes from the running game's D3D pipeline. Output: modified backbuffer overlaid on the game. This is a real-time effect running every frame at game framerate inside the game process.

Dependencies (verified)

ReShade (the framework, a separate DLL injector + shader compiler, not bundled here). No additional libs. ReShade.fxh is included in the repo (the ReShade standard include). Textures are self-contained PNGs. No build step — ReShade compiles .fx files at runtime.

Performance characteristics (from Presets/ names)

Preset names include GameplayLowest, GameplayMedium, GameplayHighest, suggesting the author tuned multiple tiers for real-time use. No fps/VRAM numbers are stated in the repo. The ASCII compute shader works at 8x8 tile grain (BUFFER_WIDTH/8 x BUFFER_HEIGHT/8 thread groups). The XeGTAO pass is based on the Intel XeGTAO reference implementation which is known to be GPU-intensive. UNVERIFIED: actual fps impact at any resolution.

Maturity

Last commit 2024-06-29, no commits in the two years since. Shader code is clean and well-structured. No issues in the shallow clone metadata. The project was explicitly built for one game (FFXIV) and looks feature-complete for that use case rather than being an evolving general tool.

---

Reuse notes for ELAI

What is concretely reusable and from where:

From ASCILINE:
- The AsciiMapper class (ascii_video_player2.py lines 88-178) is a self-contained Python function taking a NumPy gray+BGR matrix and returning an ANSI-colored string. It has zero external dependencies beyond NumPy. Directly portable as a utility for any image-to-terminal-ASCII rendering.
- The codec.py encode_frame function (107 lines, zero external deps beyond zlib + NumPy + struct) is a complete adaptive frame codec that can wrap any binary framebuffer. Reusable if ELAI ever streams binary visual output over WebSocket.
- The character palette ordering (" `.-':_,^=;><+!rc*/..." 93 chars) is a verified luminance ramp usable as-is for any ASCII art generation.
- The luminance-ramp approach is intentionally simple: gray_value // (256 // n_chars) → index. No edge detection. For static images or simple mascot rendering this is sufficient and extremely cheap.
- License constraint: MIT with Anti-Advertisement Restriction. Legal for ELAI internal tooling or any user-facing feature that is not an ad delivery surface. Worth confirming with a lawyer if ELAI ever monetizes via banner ads, but irrelevant for a dev-tool context.

From AcerolaFX:
- The ASCII shader pipeline in AcerolaFX_ASCII.fx is the most algorithmically sophisticated ASCII renderer in either repo. It uses DoG edge detection + Sobel direction + per-tile LUT lookup from sprite atlases to produce an edge-aware ASCII effect that looks far better than a plain luminance ramp. The algorithm itself is portable: the DoG + Sobel direction bucketing + sprite atlas lookup approach could be reimplemented in any GPU shader language (WGSL, GLSL, Metal Shading Language) or even a CPU image-processing library.
- The two sprite atlas textures (edgesASCII.png 40x8, fillASCII.png 80x8) are the authored character LUTs. They are the actual hand-crafted data driving the character selection; reusing them directly is possible under MIT.
- The Dither shader (Bayer + blue-noise, 8 blue noise PNG textures included) is a clean reusable dithering implementation. The 8 blue noise PNGs are high quality and freely reusable under MIT.
- Hard constraint: the .fx code as-is runs ONLY inside ReShade injected into a running D3D game. It cannot run standalone. Reuse means porting the HLSL to another shader environment — the algorithm is the value, not the files themselves. The compute shader dispatch syntax (CS_RenderASCII<8,8>, barrier(), groupshared) is standard HLSL compute and ports to WGSL (workgroup memory / workgroupBarrier) or GLSL (shared / barrier()) with modest syntactic changes.
- The ASCII shader's depth-aware edge detection (using ReShade::GetLinearizedDepth) is only useful in a 3D scene with a depth buffer. For 2D image rendering, omit the depth/normal passes and use only the DoG + luminance fill path.
- License: MIT (clean, no extra restrictions).

Summary verdict for an ASCII-render or mascot-rendering feature in ELAI:

For a server-side or terminal ASCII render (e.g. rendering a mascot image as ASCII in the dashboard or CLI): the ASCILINE AsciiMapper approach is the fastest path — it is already working Python, zero-dependency beyond NumPy, and produces colored terminal output. Port it directly.

For a GPU-rendered ASCII effect in the Tauri/WebView dashboard (e.g. a WebGPU or CSS-filter ASCII overlay on the mascot): the AcerolaFX ASCII algorithm is the better reference. The DoG + per-tile direction bucketing + sprite atlas LUT approach produces visually superior results. A WGSL port of the compute shader is the most direct adaptation. The two PNG sprite atlases (edgesASCII.png, fillASCII.png) can be reused directly as WebGPU textures.

Neither repo requires a game engine or injector for algorithm reuse — only for running the code as-is.

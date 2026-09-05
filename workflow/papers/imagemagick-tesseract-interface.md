Source: ImageMagick 7 CLI (identify + magick) and Tesseract OCR CLI
Reliability: HIGH (official imagemagick.org docs + tesseract-ocr.github.io + crates.io verified)
Files examined:
  https://imagemagick.org/script/escape.php (format specifiers, verified 2026-05-11)
  https://imagemagick.org/script/identify.php (identify command, verified 2026-05-11)
  https://tesseract-ocr.github.io/tessdoc/Command-Line-Usage.html (verified 2026-05-11)
  https://github.com/tesseract-ocr/tesseract/blob/main/doc/tesseract.1.asc (verified 2026-05-11)
  https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html (verified 2026-05-11)
  https://lib.rs/crates/magick_rust (v2.0.0, verified 2026-05-11)
  https://lib.rs/crates/tesseract (v0.15.2, verified 2026-05-11)
  https://nlfiedler.github.io/magick-rust/magick_rust/struct.MagickWand.html (verified 2026-05-11)

=== IMAGEMAGICK IDENTIFY ===

Public API (CLI):
identify | magick identify [options] filename | exit 0 on success, 1 on error (see GOTCHAS) | ImageMagick 7+ syntax
magick identify -ping filename | fast metadata-only read (header only, no full decode) | use for width/height/format checks
magick identify -format STRING filename | custom output using format specifiers | stdout | no newline unless \n in format
magick image.png json: | write full JSON metadata to stdout | comprehensive but may contain nan values (see GOTCHAS)
magick identify -verbose filename | full hierarchical metadata including channel statistics | stdout | triggers full decode

Format Specifiers (verified from imagemagick.org/script/escape.php):
%w | current width in pixels | integer string | requires full read unless -ping used
%h | current height in pixels | integer string | requires full read unless -ping used
%m | image file format (file magic) | string e.g. "PNG", "JPEG" | from header, available with -ping
%r | image class AND colorspace combined | string e.g. "DirectClass sRGB" | NOTE: includes class prefix, not pure colorspace
%z | image depth as read in (bit depth) | integer string e.g. "8" | bits per channel
%b | file size of image read in | string with unit suffix e.g. "123.4KB" | NOT a pure integer; includes unit
%f | filename including suffix | string
%n | number of images in sequence | integer string | multi-frame images (GIF, TIFF) > 1
%[width] | original width when image was read | integer string | long form of %w
%[height] | original height when image was read | integer string | long form of %h
%[colorspace] | colorspace of image data EXCLUDING transparency | string e.g. "sRGB", "Gray", "CMYK" | pure colorspace, no class prefix
%[depth] | depth of image for write | integer string
%[size] | original size of image when read | string with unit suffix e.g. "45.6KB" | same format as %b
%[channels] | colorspace mnemonic + total channel count | string e.g. "srgb 3"
%[mean] | CALCULATED: average pixel value statistic | float string | triggers full decode; expensive
%[entropy] | CALCULATED: entropy of image | float string | triggers full decode
%[standard-deviation] | CALCULATED: standard deviation | float string | triggers full decode
%[min] | CALCULATED: minimum pixel value | float string | triggers full decode
%[max] | CALCULATED: maximum pixel value | float string | triggers full decode
%[type] | CALCULATED: image type classification | string e.g. "TrueColor", "Grayscale" | triggers full decode

Compound format string example (produces parseable output):
magick identify -format "%w|%h|%m|%[colorspace]|%z|%b\n" input.png
Output: 1920|1080|PNG|sRGB|8|456.7KB

JSON output via json: pseudo-format:
magick input.png json:
Top-level JSON keys: image -> { name, baseName, format, formatDescription, mimeType, class, geometry{width,height,x,y}, resolution{x,y}, printSize, units, type, baseType, endianess, colorspace, depth, baseDepth, channelDepth, pixels, imageStatistics, channelStatistics{min,max,mean,standardDeviation,kurtosis,skewness,entropy} }
GOTCHA: channelStatistics values may be "nan" for certain images; use -format string approach instead of json: for robust parsing

Behavioral contracts:
magick identify -ping -> reads header only; %w/%h/%m/%r/%z available; %[mean] and CALCULATED specifiers NOT available (return empty)
magick identify -format -> reads full image if any CALCULATED specifier requested; reads header only if only basic specifiers
magick identify -verbose -> always reads full image; output to stdout; channel stats include min/max/mean/stddev/kurtosis/skewness/entropy per channel
exit code on corrupt/truncated PNG -> exit 1 with stderr message; CRITICAL GOTCHA: ImageMagick 7 has known issues returning exit 0 on some errors (issue #6744, unfixed as of 2026); ALWAYS check both exit code AND stderr for "corrupt" or "error" strings
stderr signals for corruption: "corrupt image", "no decode delegate", "premature end of data segment", "Expected N bytes"
-ping flag -> default behavior of identify; use +ping to force full decode

Platform constraints:
Windows: executable is magick.exe; PATH entry added at top of PATH by installer (may conflict with other tools named magick)
Windows: legacy identify.exe available by copying magick.exe and renaming; NOT shipped by default in IM7
Linux/macOS: executable is magick; legacy identify command available as symlink or separate binary
ImageMagick 7+ REQUIRED: magick_rust crate does not work with ImageMagick 6.x
Windows build from source: requires Microsoft C++ Build Tools + Windows 11 SDK; no prebuilt binary with full delegates via standard installer
Delegates (libpng, libjpeg, etc.): must be compiled in; Windows installer may omit them; verify with: magick -version | grep Delegates

Rust crate (magick_rust):
crate: magick_rust v2.0.0 (released 2025-10-03, active maintenance, Rust 2024 edition, min rustc 1.85.0)
repo: https://github.com/nlfiedler/magick-rust
license: Apache-2.0
system requirement: ImageMagick 7.1.1-26 or later (NOT 6.x)
Windows: requires building ImageMagick from source; no package manager path
API coverage: incomplete; many MagickWand C API functions missing
key methods available:
  MagickWand::new() -> MagickWand
  wand.read_image(path: &str) -> Result<(), MagickError>  [errors on corrupt files]
  wand.read_image_blob(data: &[u8]) -> Result<(), MagickError>
  wand.get_image_width() -> usize
  wand.get_image_height() -> usize
  wand.get_image_format() -> Result<String, MagickError>
  wand.get_image_colorspace() -> ColorspaceType
  wand.get_image_depth() -> usize
  wand.get_image_type() -> ImageType
  magick_wand_genesis()  [MUST call before any operations]
  magick_wand_terminus() [MUST call at shutdown]
RECOMMENDATION for B-24: prefer subprocess (std::process::Command) over magick_rust due to Windows build complexity and incomplete coverage; subprocess is more reliable cross-platform for validation-only use

Subprocess approach (recommended):
use std::process::Command;
let out = Command::new("magick")
    .args(["identify", "-format", "%w|%h|%m|%[colorspace]|%z", path])
    .output()?;
// check both out.status.success() AND out.stderr for "corrupt"/"error" strings
let stderr = String::from_utf8_lossy(&out.stderr);
let is_corrupt = !out.status.success() || stderr.contains("corrupt") || stderr.contains("error/");

Performance notes:
-ping flag: reads header only; ~10x faster than full decode for large images
Cold start overhead: ImageMagick subprocess spawn ~50-200ms on first call (library loading); subsequent calls on same process: not applicable (each subprocess is independent)
For many small validations (>10 images): batch into single invocation using glob: magick identify -format "..." *.png
Multiprocessing GOTCHA: ImageMagick 7.1.1-11+ became significantly slower under concurrent subprocess calls (69.6s vs 1.2s for 30 images); serialize or use process pool with max 2 workers

=== TESSERACT OCR CLI ===

Public API (CLI):
tesseract imagename outputbase [options] | exit 0 on success, nonzero on failure
tesseract input.png stdout -l eng --psm 6 | OCR to stdout as plain text
tesseract input.png stdout -l eng --psm 11 | sparse text mode, finds scattered text
tesseract input.png stdout -l eng --psm 6 tsv | TSV output with per-word confidence
tesseract input.png - -l eng | hyphen as outputbase = stdout (equivalent to stdout)
tesseract --list-langs | list installed language packs
tesseract --tessdata-dir /path --list-langs | list langs in custom tessdata dir

PSM (Page Segmentation Mode) values (complete, verified from tesseract.1.asc):
0  | Orientation and script detection (OSD) only
1  | Automatic page segmentation with OSD
2  | Automatic page segmentation, no OSD or OCR (NOT IMPLEMENTED)
3  | Fully automatic page segmentation, no OSD (DEFAULT)
4  | Assume single column of text, variable sizes
5  | Assume single uniform block of vertically aligned text
6  | Assume single uniform block of text
7  | Treat image as single text line
8  | Treat image as single word
9  | Treat image as single word in a circle
10 | Treat image as single character
11 | Sparse text: find as much text as possible in no particular order
12 | Sparse text with OSD
13 | Raw line: treat as single text line, bypass Tesseract-specific hacks

PSM for UI screenshots (B-24 guidance):
psm 11 RECOMMENDED for full UI screenshots with scattered text (labels, buttons, mixed layout)
psm 6 RECOMMENDED for single text blocks or dialog boxes with uniform text
psm 3 (default) acceptable when full-page layout analysis is needed
VERIFIED: psm 11 generally outperforms psm 4 and 6 for scattered UI text (GitHub issue #1108, confirmed)

OEM (OCR Engine Mode) values:
0 | Original Tesseract only (requires tessdata repo models)
1 | LSTM neural net only (DEFAULT for Tesseract 4+)
2 | Tesseract + LSTM combined
3 | Default based on what is available
RECOMMENDATION: --oem 1 (LSTM) for modern use; required for accuracy with trained data

TSV output format (verified from tessdoc):
columns: level | page_num | block_num | par_num | line_num | word_num | left | top | width | height | conf | text
conf field: 0-100 integer; -1 for non-word rows (page/block/par/line level rows)
word-level rows: word_num > 0 AND level == 5
example filter for words with conf >= 50: filter rows where level==5 AND conf >= 50

Confidence threshold for "expected_text" check (B-24):
No official Tesseract threshold documented
Research finding: Sorensen-Dice coefficient achieved best OCR post-processing results (precision/recall/F1 = 0.88)
Levenshtein distance: improved accuracy from 50.72% to 70.5%
Weighted Levenshtein (OCR-specific substitution costs) preferred for OCR comparison
RECOMMENDATION for B-24: use case-insensitive normalized Levenshtein distance with threshold 0.15 (allow 15% edit distance) OR case-insensitive substring containment for simple checks
tesseract mean_text_conf: average confidence of all word-level tokens; use >= 50 as minimum viable threshold; >= 70 for reliable text
97% word-confidence threshold (as specified in B-24 task): apply per-word filtering in TSV mode; words with conf < 97 are excluded from comparison text

Text comparison approach for B-24:
step 1: run tesseract with tsv output
step 2: parse TSV, keep only level==5 (word) rows with conf >= THRESHOLD
step 3: join kept words into normalized string (lowercase, collapse whitespace)
step 4: compare against expected_text (lowercase, collapsed) using:
  a) exact substring containment: normalized_expected in normalized_ocr_text
  b) OR Levenshtein distance ratio: edit_distance(normalized_expected, best_match_window) / len(normalized_expected) < 0.15
RECOMMENDATION: use (a) substring for short expected strings (<= 20 chars); use (b) Levenshtein for longer phrases

Language data:
Windows default path: C:\Program Files\Tesseract-OCR\tessdata\
Windows TESSDATA_PREFIX env var: parent of tessdata dir; takes precedence over compiled-in path
Linux default path: /usr/share/tesseract-ocr/4.00/tessdata/ (package install) or /usr/local/share/tessdata/
eng.traineddata: English; bundled with default install on all platforms
TESSDATA_PREFIX must point to PARENT directory of tessdata/ folder (not tessdata/ itself)
missing language failure: stderr "Error, could not initialize tesseract." + exit nonzero; NOT a silent failure

Image preprocessing for best accuracy (verified from tessdoc/ImproveQuality.html):
minimum DPI: 300 dpi; rescale if lower
background: dark text on light background; invert if needed
alpha channel: REMOVE before passing to tesseract; PNG alpha causes recognition degradation
binarization: tesseract uses Otsu internally; Tesseract 5.0+ adds Adaptive Otsu and Sauvola
grayscale: convert to grayscale before OCR for most UI screenshots

Rust crate options:
tesseract v0.15.2 (released 2025-04-19, MIT, active)
  repo: maintained by antimatter15 + ccouzens
  deps: tesseract-plumbing, tesseract-sys, thiserror
  key types: Tesseract struct, PageSegMode enum, OcrEngineMode enum, InitializeError
  key functions: ocr(path, lang) -> Result<String, _>, ocr_from_frame(...)
  Windows support: NOT explicitly documented; tesseract-sys uses vcpkg on Windows
  ASSESSMENT: usable but lower-level; subprocess equally simple for CLI-only validation
leptess v0.8.3 (last updated ~3 years ago as of 2026; STALE, do not use for new code)
  repo: https://github.com/houqp/leptess
  VERDICT: maintenance dormant; prefer tesseract crate or subprocess
RECOMMENDATION for B-24: use subprocess (std::process::Command) for portability; only use tesseract crate if in-process API is required

Subprocess approach (recommended):
use std::process::Command;
// plain text
let out = Command::new("tesseract")
    .args([path, "stdout", "-l", "eng", "--psm", "11", "--oem", "1"])
    .output()?;
// TSV with confidence scores
let out = Command::new("tesseract")
    .args([path, "stdout", "-l", "eng", "--psm", "11", "--oem", "1", "tsv"])
    .output()?;
// exit 0 = success; nonzero = failure (missing lang data, bad image, etc.)
// stderr contains error details on failure

Platform constraints:
Windows: executable is tesseract.exe at C:\Program Files\Tesseract-OCR\
Windows: must add install dir to PATH or use absolute path in subprocess call
Linux: tesseract in /usr/bin after package install; tessdata at /usr/share/tesseract-ocr/VERSION/tessdata/
Cross-platform: TESSDATA_PREFIX env var is the reliable portable approach
No Tesseract 3.x support in modern crates; use Tesseract 4.x or 5.x (LSTM models)

=== CRITICAL ASSERTIONS (testable) ===

# ImageMagick format specifier %r includes class prefix, not pure colorspace
# Use %[colorspace] for pure colorspace string comparisons
assert "sRGB" in colorspace_output, "use %[colorspace] not %r for colorspace check"

# %b returns size with unit suffix, not a bare integer
assert not file_size_output.isdigit(), "parse %b output: strip unit (KB/MB/B) before numeric comparison"

# identify with -ping does NOT compute CALCULATED specifiers
assert mean_from_ping == "" or mean_from_ping is None, "-ping skips CALCULATED specifiers"

# tesseract TSV non-word rows have conf == -1
assert all(int(row["conf"]) >= 0 for row in word_rows if row["level"] == "5")

# tesseract missing language exits nonzero with stderr message
assert proc.returncode != 0 and b"Error" in proc.stderr, "missing tessdata -> nonzero exit + stderr error"

# ImageMagick json: output may contain nan values; do not use standard JSON parser unconditionally
# Use -format string approach for reliable structured output
assert "|" in format_output, "use pipe-delimited -format string instead of json: for robust parsing"

# magick_wand_genesis() must be called before any magick_rust operations
# failure to call = undefined behavior (no error return, silent crash possible)
assert genesis_called_before_wand_creation, "magick_wand_genesis() is mandatory init"

=== FAILURE MODES ===

ImageMagick:
1. Exit code 0 on error (unfixed known bug #6744): always parse stderr in addition to exit code
2. No decode delegate (missing libpng/libjpeg on Windows): stderr "no decode delegate"; exit 1
3. -ping returns empty string for CALCULATED specifiers (%[mean], %[type], etc.): use +ping or separate call
4. %b and %[size] return "123.4KB" not integer: must strip unit suffix for numeric comparison
5. json: output contains "nan" for degenerate images: cannot be parsed by standard JSON parsers
6. Concurrent subprocess calls severely degraded in IM 7.1.1-11+: serialize or limit parallelism
7. %r returns "DirectClass sRGB" not "sRGB": wrong specifier for colorspace string comparison
8. Windows delegates missing from default installer: magick -version Delegates list may be empty

Tesseract:
1. Missing language data: exits nonzero with "Error, could not initialize tesseract." on stderr
2. PNG alpha channel: degrades OCR accuracy silently; strip alpha before processing
3. Low DPI input: accuracy degrades below 300 DPI; no error, just poor results
4. PSM 2 not implemented: silently falls back or errors; do not use
5. TESSDATA_PREFIX must point to PARENT of tessdata/ not tessdata/ itself: common misconfiguration
6. conf field is -1 for non-word level TSV rows: filter level==5 before applying confidence threshold
7. Inverted images (white text on dark): poor accuracy; invert before passing to tesseract
8. leptess crate is stale (3+ years no updates): do not use for new code

=== IMPLEMENTATION PITFALLS (DO NOT DO THIS) ===

DO NOT parse %r for colorspace string (it returns "DirectClass sRGB" not "sRGB"); use %[colorspace]
DO NOT treat %b or %[size] as integer (returns "123.4KB"); parse and strip unit
DO NOT rely solely on exit code for corrupt image detection; always check stderr
DO NOT use json: output format without nan-guarding; use pipe-delimited -format string
DO NOT use -ping when CALCULATED specifiers (%[mean], %[type]) are needed; use +ping or omit flag
DO NOT call magick_rust functions without calling magick_wand_genesis() first
DO NOT build magick_rust on Windows expecting a simple dependency; requires ImageMagick built from source
DO NOT use leptess crate for new code (maintenance dormant)
DO NOT use PSM 2 (not implemented in Tesseract)
DO NOT pass TESSDATA_PREFIX pointing to tessdata/ folder itself; must point to its parent
DO NOT pass PNG with alpha channel to Tesseract without stripping alpha first
DO NOT assume tesseract is on PATH on Windows; may need absolute path or explicit PATH config
DO NOT use str.isdigit() or direct int() cast on %b output; strip KB/MB/B suffix first
DO NOT run more than 2 concurrent ImageMagick subprocess instances (regression in 7.1.1-11+)

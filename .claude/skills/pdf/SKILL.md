---
name: pdf
description: Convert a PDF (or image/DOCX/PPTX/XLSX) to Markdown or plain text, via the deterministic Linux extract engine (pymupdf/pypdf; optional MinerU). Triggers when the user types /pdf <path-or-url>, says "extract this pdf", "pdf to markdown", "pdf to text", "convert this pdf", "rip this pdf to md", or hands a PDF and asks for its text/markdown. Standalone utility; read-only on the project; works in any FSM state; does not transition the FSM.
license: MIT
metadata:
  version: "1.1.0"
  author: "elai"
---

Turn a PDF (or URL to a PDF) into clean Markdown or text using the **deterministic**
shared extraction engine at
`/home/researcher/teikoku/.claude/tools/pdf_extract.py`.

**Default path uses no ML model** (pymupdf text+structure, then pypdf). That is
the required behaviour for `/pdf`: no agent improvisation, no VLM/OCR model.

Optional model OCR (scanned/handwritten when a model is required):
`baidu/Unlimited-OCR` via
`/home/researcher/teikoku/.claude/tools/unlimited_ocr_cpu.py` — **not** the default
`/pdf` engine. See MODEL OCR below.

ENGINE + INSTALL (Linux host)

Light extract venv (already provisioned on this host):
```
/home/researcher/teikoku/.claude/tools/.pdf-extract-venv/bin/python
```
Re-create if missing:
```
uv venv --python 3.12 /home/researcher/teikoku/.claude/tools/.pdf-extract-venv
uv pip install --python /home/researcher/teikoku/.claude/tools/.pdf-extract-venv/bin/python pymupdf pypdf
```

Optional MinerU (layout-heavy, large install): set `MINERU_BIN` or install under
`/home/researcher/teikoku/.claude/tools/.mineru-venv`. Without MinerU, deterministic
pymupdf/pypdf is the engine — that is fine and expected on this Linux box.

Legacy Windows note (historical only): older docs pointed at
`C:\maintainer\.claude\tools\pdf_extract.py` + MinerU. **On this Linux host do
not depend on those paths or /mnt/c interop.** Use the teikoku tools paths above.

WHAT TO DO WHEN THIS FIRES

1. Resolve the input. Take the path **or URL** from the user's message (token
   after `/pdf`, or the PDF they referenced). Expand `~`. Quote paths with
   spaces. URLs (`http://` / `https://`) are downloaded by the engine to a real
   file. If no path/URL is given, ask for one.

2. Decide format from the user's words:
   - default: `--format md`
   - "to text" / "as txt" / "plain text" => `--format txt`
   - "no images" is a no-op on the deterministic engine (no figure copy)
   - "scanned" / "handwritten" / model OCR requested => use MODEL OCR path
     below instead of the default command

3. Pick the output dir. Default: `<input_dir>/<stem>_extracted` (engine default)
   unless the user names one; pass with `--out`.

4. Run the **deterministic** engine (always use the extract venv python):
   ```
   /home/researcher/teikoku/.claude/tools/.pdf-extract-venv/bin/python \
     /home/researcher/teikoku/.claude/tools/pdf_extract.py \
     "<path-or-url>" \
     --format <md|txt> \
     [--out "<outdir>"] \
     --print-path
   ```
   Use `--json` when you need a machine-readable payload. Use `--print-path`
   so stdout is **only** the absolute primary extract path.

5. **Report the full absolute path of the primary extracted artifact**
   (`.md` or `.txt`) as the main result. That path must exist on disk. Also
   mention which engine ran (`pymupdf` / `pypdf` / `mineru`) if known from
   non-`--print-path` / `--json` output. Do not improvise extraction in the
   agent context window.

6. Offer follow-ups (do not auto-run): index into local-rag (`mcp_ingest_file`
   on the produced `.md`), or open/preview the markdown.

MODEL OCR (optional — Unlimited-OCR, NOT default /pdf)

When the user asks for model OCR / scanned-doc quality and CPU is acceptable:
```
/home/researcher/teikoku/.claude/tools/.unlimited-ocr-venv/bin/python \
  /home/researcher/teikoku/.claude/tools/unlimited_ocr_cpu.py \
  "<local-pdf-or-image>" \
  --out "<outdir>" \
  --print-path
```
Weights: Hugging Face `baidu/Unlimited-OCR` (~3B, CPU fp32). Slow on CPU; keep
pages tiny for smoke. Still report the absolute primary path.

NOTES

- `/pdf` default = deterministic, no Unlimited-OCR, no agent re-implementation.
- Always surface the **absolute** primary extract path.
- Read-only on the project. No FSM transition. No commit.
- URL support is built into `pdf_extract.py` (download then extract).

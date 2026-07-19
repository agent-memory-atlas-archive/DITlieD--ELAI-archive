"""Stage ONNX model artifacts at canonical paths under .elai_cc/models/

Usage: python stage_onnx_models.py [--force] [--only MODEL,...] [--models-root PATH]
                                    [--skip-graphcodebert-export]
                                    [--skip-coderankembed-export]
                                    [--skip-models] [--vram-tier {8,12,16,24,48,cloud}]
                                    [--manifest-out PATH]
Models: graphcodebert, coderankembed, granite, cross-encoder-l12, cross-encoder-l6,
        splade, bge-reranker-v2-m3, sidecar-3b, sidecar-7b

coderankembed is exported from nomic-ai/CodeRankEmbed via optimum on CPU, then
INT8-quantized via onnxruntime.quantization (QDQ, per-channel, CPU-only). The
final staged file is the INT8 model at models/onnx/coderankembed.onnx. The
unofficial community quant mrsladoje/CodeRankEmbed-onnx-int8 is NOT used.
"""
import argparse
import os
import random
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent  # scripts/ -> .elai_cc/
MIN_ONNX_BYTES = 1 * 1024 * 1024    # 1 MB — ONNX files below this are corrupt/truncated
MIN_TOK_BYTES  = 1 * 1024            # 1 kB — tokenizer.json below this is corrupt
MIN_SIDECAR_BYTES = 500 * 1024 * 1024  # 500 MB — sidecar safetensor shards are GB-scale

# recipe keys: method, hf_repo, onnx_file, tok_file, task, optional, expected_last_dim
# onnx/tok are callables root -> Path
MODELS: dict = {
    "graphcodebert": dict(
        onnx=lambda r: r/"onnx"/"graphcodebert.onnx",
        tok=lambda r:  r/"onnx"/"graphcodebert-tokenizer.json",
        method="export", hf_repo="microsoft/graphcodebert-base",
        onnx_file=None, tok_file=None, task="feature-extraction",
        expected_last_dim=768, optional=False),
    # J-18 S5: CodeRankEmbed — exported from official nomic-ai source on CPU
    # via optimum, then INT8-quantized via onnxruntime.quantization (QDQ,
    # per-channel). Final staged file is the INT8 model. The unofficial
    # community quant mrsladoje/CodeRankEmbed-onnx-int8 is NOT used
    # (anti-slopsquatting: plan J-18_p1). Dim=768 matches the existing
    # HNSW slot (CODE_VEC_DIM=768); no usearch geometry change.
    "coderankembed": dict(
        onnx=lambda r: r/"onnx"/"coderankembed.onnx",
        tok=lambda r:  r/"onnx"/"coderankembed-tokenizer.json",
        method="export", hf_repo="nomic-ai/CodeRankEmbed",
        onnx_file=None, tok_file=None, task="feature-extraction",
        expected_last_dim=768, optional=False,
        int8=True),
    "granite": dict(
        onnx=lambda r: r/"onnx"/"granite-embed.onnx",
        tok=lambda r:  r/"onnx"/"granite-embed-tokenizer.json",
        method="download", hf_repo="ibm-granite/granite-embedding-30m-english",
        onnx_file="model.onnx", tok_file="tokenizer.json",
        expected_last_dim=384, optional=False),
    "cross-encoder-l12": dict(
        onnx=lambda r: r/"onnx"/"cross-encoder-l12.onnx",
        tok=lambda r:  r/"onnx"/"tokenizer.json",
        method="download", hf_repo="Xenova/ms-marco-MiniLM-L-12-v2",
        onnx_file="onnx/model.onnx", tok_file="tokenizer.json",
        expected_last_dim=None, optional=False,
        tok_max_bytes=900_000),  # BERT WordPiece ~711 kB; reject if >900 kB (BPE collision)
    "cross-encoder-l6": dict(
        onnx=lambda r: r/"onnx"/"cross-encoder-l6.onnx",
        tok=lambda r:  r/"onnx"/"tokenizer.json",
        method="download", hf_repo="Xenova/ms-marco-MiniLM-L-6-v2",
        onnx_file="onnx/model.onnx", tok_file=None,
        expected_last_dim=None, optional=True),
    # E-17 SP3 (optional default-flip path): BAAI/bge-reranker-v2-m3 staged
    # from the community-maintained ONNX export at
    # huggingface.co/onnx-community/bge-reranker-v2-m3-ONNX (verified
    # 2026-05-23 via WebSearch — repo exists, official BAAI provenance via
    # the BAAI/bge-reranker-v2-m3 source). 568M params, multilingual, ~1.1 GB
    # ONNX. Optional in the default sweep because it is large and only
    # required if the user opts into ReRankerModel::BgeV2M3 via config.
    # Once staged, the cli doctor knowledge-models probe surfaces presence
    # and the default-flip block in reranker.rs can be lifted (still ops
    # decision, not auto). JINA-MODEL-PENDING: jinaai/jina-reranker-v3
    # has NO official ONNX as of 2026-05-23 (PyTorch / MLX / GGUF only);
    # do NOT add to this recipe set until an official .onnx ships.
    "bge-reranker-v2-m3": dict(
        onnx=lambda r: r/"onnx"/"bge-reranker-v2-m3.onnx",
        tok=lambda r:  r/"onnx"/"bge-reranker-v2-m3-tokenizer.json",
        method="download", hf_repo="onnx-community/bge-reranker-v2-m3-ONNX",
        onnx_file="onnx/model.onnx", tok_file="tokenizer.json",
        expected_last_dim=None, optional=True),
    "splade": dict(
        onnx=lambda r: r/"splade"/"model.onnx",
        tok=lambda r:  r/"splade"/"tokenizer.json",
        method="download", hf_repo="prithivida/Splade_PP_en_v1",
        onnx_file="onnx/model.onnx", tok_file="tokenizer.json",
        expected_last_dim=30522, optional=False),
    # Sidecar recipes (Phase 4) — staged via snapshot_download (full HF snapshot)
    # because the sidecar loads safetensors via AutoModelForCausalLM.from_pretrained
    # (requires config.json + tokenizer + multi-shard safetensors). NF4 4-bit
    # quantization happens at LOAD time via --load-in-4bit (bitsandbytes), NOT
    # at staging time. Boot command locked to REMOTE-SIDECAR.md:69-76.
    "sidecar-3b": dict(
        onnx=None, tok=None,
        method="hf_snapshot", hf_repo="huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated",
        local_subdir="Qwen2.5-Coder-3B-Instruct-abliterated",
        onnx_file=None, tok_file=None,
        expected_last_dim=None, optional=False,
        sidecar_tiers=("8", "12", "16", "24", "48")),
    "sidecar-7b": dict(
        onnx=None, tok=None,
        method="hf_snapshot", hf_repo="huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated",
        local_subdir="Qwen2.5-Coder-7B-Instruct-abliterated",
        onnx_file=None, tok_file=None,
        expected_last_dim=None, optional=True,
        sidecar_tiers=("16", "24", "48")),
}
ALL = list(MODELS)
SIDECAR_RECIPES = {"sidecar-3b", "sidecar-7b"}


@dataclass
class StageResult:
    name: str
    status: str
    onnx_size_mb: float = 0.0
    onnx_opset: int | None = None
    onnx_inputs: list = field(default_factory=list)
    onnx_outputs: list = field(default_factory=list)
    reason: str = ""
    asset_path: Path | None = None
    blake3_hex: str = ""


VRAM_TIERS = ("8", "12", "16", "24", "48", "cloud")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Stage ONNX model artifacts")
    p.add_argument("--force", action="store_true")
    p.add_argument("--only", default="")
    p.add_argument("--models-root", default="")
    p.add_argument("--skip-graphcodebert-export", action="store_true")
    p.add_argument("--skip-coderankembed-export", action="store_true")
    p.add_argument("--skip-models", action="store_true",
                   help="bypass all recipe execution + manifest authoring "
                        "(CI / corp-firewall / docker-build mode)")
    p.add_argument("--vram-tier", choices=VRAM_TIERS, default="",
                   help="restrict sidecar recipes: 8/12 -> sidecar-3b only; "
                        "16/24/48 -> sidecar-3b + sidecar-7b; cloud -> neither.")
    p.add_argument("--manifest-out", default="",
                   help="path to author per-host CHECKSUMS.blake3 manifest "
                        "(default: <models_root>/CHECKSUMS.blake3)")
    return p.parse_args()


def _blake3_hash_file(p: Path) -> str:
    """Streaming 1 MiB-chunked blake3 hash; safe for multi-GB sidecar shards."""
    from blake3 import blake3
    h = blake3()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _blake3_hash_dir(d: Path) -> str:
    """Deterministic dir-hash: sorted (filename + per-file-hash) pairs."""
    from blake3 import blake3
    h = blake3()
    for name in sorted(os.listdir(d)):
        f = d / name
        if f.is_file():
            h.update(name.encode("utf-8"))
            h.update(_blake3_hash_file(f).encode("ascii"))
    return h.hexdigest()


def _with_hf_backoff(call, *args, **kwargs):
    """Wrap an HF download call with 3-attempt exponential backoff on HTTP 429.

    Plan ADVERSARIAL bullet 1 (Phase 4): HF rate-limit responses are retried
    with backoff (max 3), NOT silently swallowed. 4th failure raises with the
    HF status-page URL surfaced to the operator.
    """
    try:
        from huggingface_hub.utils import HfHubHTTPError
    except ImportError:
        from huggingface_hub.errors import HfHubHTTPError  # type: ignore[attr-defined]
    for attempt in range(3):
        try:
            return call(*args, **kwargs)
        except HfHubHTTPError as e:
            status = getattr(getattr(e, "response", None), "status_code", None)
            if status == 429 and attempt < 2:
                delay = (2 ** (attempt + 1)) + random.uniform(0.0, 1.0)
                print(f"  [WARN] HF 429 rate-limit, sleeping {delay:.1f}s "
                      f"(attempt {attempt + 1}/3)", file=sys.stderr)
                time.sleep(delay)
                continue
            if status == 429:
                raise RuntimeError(
                    f"HF rate-limit exhausted after 3 retries; "
                    f"see https://status.huggingface.co"
                ) from e
            raise
    raise RuntimeError("unreachable: _with_hf_backoff exhausted loop without raising")


def resolve_models_root(arg: str) -> Path:
    if arg:
        return Path(arg)
    env = os.environ.get("ELAI_PROJECT_ROOT", "")
    return Path(env) / "models" if env else DEFAULT_ROOT / "models"


def resolve_target_paths(models_root: Path) -> dict:
    return {n: {"onnx_path": c["onnx"](models_root), "tokenizer_path": c["tok"](models_root)}
            for n, c in MODELS.items()}


def _hf_download_to(repo_id: str, filename: str,
                     target_path: Path, hf_token: str | None) -> None:
    """Download a single file from HF to target_path (always copies from HF cache)."""
    from huggingface_hub import hf_hub_download
    target_path.parent.mkdir(parents=True, exist_ok=True)
    cached = _with_hf_backoff(
        hf_hub_download, repo_id=repo_id, filename=filename, token=hf_token,
    )
    shutil.copy2(cached, target_path)


def _hf_snapshot_to(repo_id: str, target_dir: Path,
                     hf_token: str | None) -> Path:
    """Snapshot-download an HF repo into target_dir; mirror for sidecar recipes.

    Uses HF_HOME-honoring cache via huggingface_hub.snapshot_download default
    behavior; allow_patterns restricts to inference-relevant files (no .git,
    no .gitattributes, no README artifacts that bloat the on-disk footprint).
    """
    from huggingface_hub import snapshot_download
    target_dir.mkdir(parents=True, exist_ok=True)
    snapshot_dir = _with_hf_backoff(
        snapshot_download,
        repo_id=repo_id,
        cache_dir=os.environ.get("HF_HOME") or None,
        allow_patterns=[
            "*.safetensors", "*.safetensors.index.json",
            "config.json", "generation_config.json",
            "tokenizer.json", "tokenizer_config.json",
            "special_tokens_map.json", "vocab.json", "merges.txt",
        ],
        token=hf_token,
    )
    for name in os.listdir(snapshot_dir):
        src = Path(snapshot_dir) / name
        dst = target_dir / name
        if src.is_file():
            shutil.copy2(src, dst)
    return target_dir


def download_pretrained_onnx(repo_id: str, file_subpath: str,
                              target_path: Path, hf_token: str | None) -> None:
    _hf_download_to(repo_id, file_subpath, target_path, hf_token)


def download_tokenizer(repo_id: str, target_path: Path,
                        hf_token: str | None, tok_filename: str = "tokenizer.json") -> None:
    _hf_download_to(repo_id, tok_filename, target_path, hf_token)


def export_onnx_with_optimum(repo_id: str, task: str,
                              target_dir: Path, target_filename: str,
                              int8: bool = False) -> None:
    target_dir.mkdir(parents=True, exist_ok=True)
    tok_name = (Path(target_filename).stem + "-tokenizer.json")
    with tempfile.TemporaryDirectory() as tmp:
        cmd = [sys.executable, "-m", "optimum.exporters.onnx",
               "--model", repo_id, "--task", task, "--device", "cpu", tmp]
        print(f"  [optimum] {' '.join(cmd)}")
        r = subprocess.run(cmd, capture_output=False, text=True, timeout=900)
        if r.returncode != 0:
            raise RuntimeError(f"optimum export rc={r.returncode}")
        src = Path(tmp) / "model.onnx"
        if not src.exists():
            cands = list(Path(tmp).glob("*.onnx"))
            if not cands:
                raise RuntimeError(f"no .onnx found in {tmp}")
            src = cands[0]
        if int8:
            int8_path = Path(tmp) / "model_int8.onnx"
            qcmd = [sys.executable, "-m", "onnxruntime.quantization.quantize",
                    "--input", str(src), "--output", str(int8_path),
                    "--quant_format", "QDQ", "--per_channel"]
            print(f"  [quantize] {' '.join(qcmd)}")
            q = subprocess.run(qcmd, capture_output=False, text=True, timeout=900)
            if q.returncode != 0:
                raise RuntimeError(f"onnxruntime quantize rc={q.returncode}")
            if not int8_path.exists():
                raise RuntimeError(f"INT8 quantize produced no file at {int8_path}")
            src = int8_path
        shutil.copy2(src, target_dir / target_filename)
        tok_src = Path(tmp) / "tokenizer.json"
        if tok_src.exists():
            shutil.copy2(tok_src, target_dir / tok_name)


def verify_onnx_loadable(onnx_path: Path,
                          expected_last_dim: int | None) -> tuple[bool, dict]:
    try:
        import numpy as np
        import onnxruntime as ort
        sess = ort.InferenceSession(str(onnx_path), providers=["CPUExecutionProvider"])
        ins = [i.name for i in sess.get_inputs()]
        outs = [o.name for o in sess.get_outputs()]
        opset = None
        try:
            import onnx
            m = onnx.load(str(onnx_path))
            opset = max(op.version for op in m.opset_import) if m.opset_import else None
        except Exception:
            pass
        dummy = {n: (np.zeros((1, 8), dtype=np.int64) if "type_id" in n
                     else np.ones((1, 8), dtype=np.int64)) for n in ins}
        try:
            out = sess.run(None, dummy)
            if expected_last_dim is not None and out:
                actual = out[0].shape[-1]
                if actual != expected_last_dim:
                    return False, {"error": f"shape[-1]={actual} != {expected_last_dim}",
                                   "inputs": ins, "outputs": outs, "opset": opset}
        except Exception as e:
            return False, {"error": f"inference: {e}", "inputs": ins, "outputs": outs, "opset": opset}
        return True, {"inputs": ins, "outputs": outs, "opset": opset}
    except Exception as e:
        return False, {"error": str(e), "inputs": [], "outputs": [], "opset": None}


def stage_sidecar(name: str, models_root: Path, force: bool,
                   hf_token: str | None) -> StageResult:
    cfg = MODELS[name]
    target_dir = models_root / cfg["local_subdir"]
    # Idempotent skip: target_dir non-empty + total >= MIN_SIDECAR_BYTES.
    if not force and target_dir.exists():
        total = sum(f.stat().st_size for f in target_dir.iterdir() if f.is_file())
        if total >= MIN_SIDECAR_BYTES:
            mb = total / 1e6
            print(f"  SKIP: {name} already staged ({mb:.1f} MB)")
            try:
                hex_ = _blake3_hash_dir(target_dir)
            except Exception as e:
                return StageResult(name, "FAIL", mb, reason=f"blake3: {e}",
                                   asset_path=target_dir)
            return StageResult(name, "SKIP", mb, asset_path=target_dir, blake3_hex=hex_)
    try:
        _hf_snapshot_to(cfg["hf_repo"], target_dir, hf_token)
        total = sum(f.stat().st_size for f in target_dir.iterdir() if f.is_file())
        if total < MIN_SIDECAR_BYTES:
            return StageResult(name, "FAIL",
                               reason=f"sidecar total bytes {total} below MIN_SIDECAR_BYTES",
                               asset_path=target_dir)
        hex_ = _blake3_hash_dir(target_dir)
        mb = total / 1e6
        return StageResult(name, "PASS", mb, asset_path=target_dir, blake3_hex=hex_)
    except Exception as e:
        return StageResult(name, "FAIL", reason=str(e), asset_path=target_dir)


def stage_model(name: str, models_root: Path, force: bool,
                hf_token: str | None, skip_gcb_export: bool,
                skip_cre_export: bool = False) -> StageResult:
    cfg = MODELS[name]
    if cfg.get("method") == "hf_snapshot":
        return stage_sidecar(name, models_root, force, hf_token)
    onnx_p = cfg["onnx"](models_root)
    tok_p  = cfg["tok"](models_root)
    onnx_p.parent.mkdir(parents=True, exist_ok=True)

    def needs_onnx(p: Path) -> bool:
        return force or not p.exists() or p.stat().st_size < MIN_ONNX_BYTES

    def needs_tok(p: Path, max_bytes: int | None = None) -> bool:
        if force or not p.exists():
            return True
        sz = p.stat().st_size
        if sz < MIN_TOK_BYTES:
            return True
        if max_bytes is not None and sz > max_bytes:
            return True  # wrong tokenizer type (e.g. BPE blob at WordPiece slot)
        return False

    tok_max = cfg.get("tok_max_bytes")
    if not needs_onnx(onnx_p) and (cfg["tok_file"] is None or not needs_tok(tok_p, tok_max)):
        mb = onnx_p.stat().st_size / 1e6
        print(f"  SKIP: {name} already staged ({mb:.1f} MB)")
        ok, info = verify_onnx_loadable(onnx_p, cfg["expected_last_dim"])
        return StageResult(name, "SKIP", mb, info.get("opset"), info.get("inputs", []), info.get("outputs", []))

    try:
        if cfg["method"] == "export":
            if name == "graphcodebert" and skip_gcb_export:
                return StageResult(name, "SKIP", reason="--skip-graphcodebert-export")
            if name == "coderankembed" and skip_cre_export:
                return StageResult(name, "SKIP", reason="--skip-coderankembed-export")
            export_onnx_with_optimum(cfg["hf_repo"], cfg["task"], onnx_p.parent, onnx_p.name,
                                     int8=cfg.get("int8", False))
        else:
            if needs_onnx(onnx_p):
                print(f"  [dl] {name} onnx {cfg['hf_repo']}/{cfg['onnx_file']}")
                download_pretrained_onnx(cfg["hf_repo"], cfg["onnx_file"], onnx_p, hf_token)
            if cfg["tok_file"] and needs_tok(tok_p, tok_max):
                print(f"  [dl] {name} tok  {cfg['hf_repo']}/{cfg['tok_file']}")
                download_tokenizer(cfg["hf_repo"], tok_p, hf_token, cfg["tok_file"])

        if not onnx_p.exists():
            return StageResult(name, "FAIL", reason="onnx missing after download")

        mb = onnx_p.stat().st_size / 1e6
        ok, info = verify_onnx_loadable(onnx_p, cfg["expected_last_dim"])
        if not ok:
            return StageResult(name, "FAIL", mb, reason=f"ort: {info.get('error')}")
        try:
            hex_ = _blake3_hash_file(onnx_p)
        except Exception as e:
            return StageResult(name, "FAIL", mb, reason=f"blake3: {e}",
                               asset_path=onnx_p)
        return StageResult(name, "PASS", mb, info.get("opset"),
                           info.get("inputs", []), info.get("outputs", []),
                           asset_path=onnx_p, blake3_hex=hex_)
    except Exception as e:
        return StageResult(name, "FAIL", reason=str(e))


def author_manifest(results: list, manifest_path: Path) -> int:
    """Write per-host CHECKSUMS.blake3 with one row per PASS/SKIP staged asset.

    Format: `<blake3-hex-64><2-space><asset-relative-name>\\n` matching the
    b3sum CLI shape Phase 1 release.yml produces and Phase 3 verify_blake3
    awk parser at scripts/install/elai_install_lib.sh:286.
    Atomic write via tmp + os.replace.
    """
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    rows: list[str] = []
    for r in results:
        if r.status in ("PASS", "SKIP") and r.blake3_hex and r.asset_path:
            asset_rel = r.asset_path.name
            rows.append(f"{r.blake3_hex}  {asset_rel}\n")
    iso = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    header = (
        "# CHECKSUMS.blake3 — per-host runtime manifest authored by\n"
        "# .elai_cc/scripts/stage_onnx_models.py at install-time.\n"
        f"# generated: {iso}\n"
        "# format: <blake3-hex-64><2-space><asset-filename>\n"
        "#\n"
    )
    tmp = manifest_path.with_suffix(manifest_path.suffix + f".tmp.{os.getpid()}")
    tmp.write_text(header + "".join(rows), encoding="utf-8")
    os.replace(tmp, manifest_path)
    print(f"\n[INFO] author_manifest: wrote {len(rows)} rows to {manifest_path}")
    return len(rows)


def _filter_by_vram_tier(selected: list[str], vram_tier: str) -> list[str]:
    """Drop sidecar recipes whose sidecar_tiers do not include vram_tier.

    Empty vram_tier ("") means no tier-based filtering — keep all selected.
    `cloud` tier drops both sidecar recipes (cloud-mode uses remote tunnel).
    """
    if not vram_tier:
        return selected
    if vram_tier == "cloud":
        return [n for n in selected if n not in SIDECAR_RECIPES]
    out = []
    for name in selected:
        cfg = MODELS[name]
        sidecar_tiers = cfg.get("sidecar_tiers")
        if sidecar_tiers is None or vram_tier in sidecar_tiers:
            out.append(name)
        else:
            print(f"  [WARN] vram_tier={vram_tier} skipping {name} per LOCK-2", file=sys.stderr)
    return out


def main() -> int:
    args = parse_args()
    hf_token: str | None = os.environ.get("HF_TOKEN") or None
    models_root = resolve_models_root(args.models_root)
    print(f"Models root: {models_root}")

    if args.skip_models:
        print("[SKIP] --skip-models in effect; recipe execution + manifest authoring bypassed")
        return 0

    if args.only:
        selected = [m.strip() for m in args.only.split(",") if m.strip()]
        bad = [m for m in selected if m not in MODELS]
        if bad:
            print(f"ERROR: unknown model(s): {bad}. Valid: {ALL}")
            return 1
    else:
        selected = list(ALL)
    selected = _filter_by_vram_tier(selected, args.vram_tier)

    results: list[StageResult] = []
    for name in selected:
        print(f"\n--- {name} ---")
        r = stage_model(name, models_root, args.force, hf_token,
                        args.skip_graphcodebert_export, args.skip_coderankembed_export)
        results.append(r)
        print(f"  {r.status}: {name}"
              + (f" | size={r.onnx_size_mb:.1f} MB" if r.onnx_size_mb else "")
              + (f" | opset={r.onnx_opset}" if r.onnx_opset else "")
              + (f" | {r.reason}" if r.reason else ""))

    print("\n" + "=" * 60 + "\nSUMMARY\n" + "=" * 60)
    passes = [r for r in results if r.status == "PASS"]
    skips  = [r for r in results if r.status == "SKIP"]
    fails  = [r for r in results if r.status == "FAIL"]
    for r in results:
        print(f"  {r.status:<5} {r.name:<25} {r.onnx_size_mb:>8.1f} MB  {r.reason or ''}")
    print(f"\nPASS={len(passes)} SKIP={len(skips)} FAIL={len(fails)}")

    manifest_path = (Path(args.manifest_out) if args.manifest_out
                      else models_root / "CHECKSUMS.blake3")
    try:
        author_manifest(results, manifest_path)
    except Exception as e:
        print(f"[WARN] author_manifest failed: {e}", file=sys.stderr)

    mandatory_fails = [r for r in fails if not MODELS[r.name]["optional"]]
    return 0 if not mandatory_fails else 1


if __name__ == "__main__":
    sys.exit(main())

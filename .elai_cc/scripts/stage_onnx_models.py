"""Stage ONNX model artifacts at canonical paths under .elai_cc/models/

Usage: python stage_onnx_models.py [--force] [--only MODEL,...] [--models-root PATH]
                                    [--skip-graphcodebert-export]
Models: graphcodebert, granite, cross-encoder-l12, cross-encoder-l6, splade
"""
import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parent.parent  # scripts/ -> .elai_cc/
MIN_ONNX_BYTES = 1 * 1024 * 1024    # 1 MB — ONNX files below this are corrupt/truncated
MIN_TOK_BYTES  = 1 * 1024            # 1 kB — tokenizer.json below this is corrupt

# recipe keys: method, hf_repo, onnx_file, tok_file, task, optional, expected_last_dim
# onnx/tok are callables root -> Path
MODELS: dict = {
    "graphcodebert": dict(
        onnx=lambda r: r/"onnx"/"graphcodebert.onnx",
        tok=lambda r:  r/"onnx"/"graphcodebert-tokenizer.json",
        method="export", hf_repo="microsoft/graphcodebert-base",
        onnx_file=None, tok_file=None, task="feature-extraction",
        expected_last_dim=768, optional=False),
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
}
ALL = list(MODELS)


@dataclass
class StageResult:
    name: str
    status: str
    onnx_size_mb: float = 0.0
    onnx_opset: int | None = None
    onnx_inputs: list = field(default_factory=list)
    onnx_outputs: list = field(default_factory=list)
    reason: str = ""


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Stage ONNX model artifacts")
    p.add_argument("--force", action="store_true")
    p.add_argument("--only", default="")
    p.add_argument("--models-root", default="")
    p.add_argument("--skip-graphcodebert-export", action="store_true")
    return p.parse_args()


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
    cached = hf_hub_download(repo_id=repo_id, filename=filename, token=hf_token)
    shutil.copy2(cached, target_path)


def download_pretrained_onnx(repo_id: str, file_subpath: str,
                              target_path: Path, hf_token: str | None) -> None:
    _hf_download_to(repo_id, file_subpath, target_path, hf_token)


def download_tokenizer(repo_id: str, target_path: Path,
                        hf_token: str | None, tok_filename: str = "tokenizer.json") -> None:
    _hf_download_to(repo_id, tok_filename, target_path, hf_token)


def export_onnx_with_optimum(repo_id: str, task: str,
                              target_dir: Path, target_filename: str) -> None:
    target_dir.mkdir(parents=True, exist_ok=True)
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
        shutil.copy2(src, target_dir / target_filename)
        tok_src = Path(tmp) / "tokenizer.json"
        if tok_src.exists():
            shutil.copy2(tok_src, target_dir / "graphcodebert-tokenizer.json")


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


def stage_model(name: str, models_root: Path, force: bool,
                hf_token: str | None, skip_gcb_export: bool) -> StageResult:
    cfg = MODELS[name]
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
            if skip_gcb_export:
                return StageResult(name, "SKIP", reason="--skip-graphcodebert-export")
            export_onnx_with_optimum(cfg["hf_repo"], cfg["task"], onnx_p.parent, onnx_p.name)
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
        return StageResult(name, "PASS", mb, info.get("opset"), info.get("inputs", []), info.get("outputs", []))
    except Exception as e:
        return StageResult(name, "FAIL", reason=str(e))


def main() -> int:
    args = parse_args()
    hf_token: str | None = os.environ.get("HF_TOKEN") or None
    models_root = resolve_models_root(args.models_root)
    print(f"Models root: {models_root}")

    if args.only:
        selected = [m.strip() for m in args.only.split(",") if m.strip()]
        bad = [m for m in selected if m not in MODELS]
        if bad:
            print(f"ERROR: unknown model(s): {bad}. Valid: {ALL}")
            return 1
    else:
        selected = list(ALL)

    results: list[StageResult] = []
    for name in selected:
        print(f"\n--- {name} ---")
        r = stage_model(name, models_root, args.force, hf_token, args.skip_graphcodebert_export)
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
    mandatory_fails = [r for r in fails if not MODELS[r.name]["optional"]]
    return 0 if not mandatory_fails else 1


if __name__ == "__main__":
    sys.exit(main())

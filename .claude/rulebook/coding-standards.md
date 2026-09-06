
Stack=`Rust/Python/Cargo/Pytest`.
No new dependencies without human validation.
Principles=`TDD, deterministic code, SOLID principles`.
bash for sourcing, write_to_file for writing (bash if tool fails).
(basically focus on using bash as tools.)
sidecar server location (local source, deployed remotely via rsync);

/workspace/ELAI\.elai_cc\python\elai_inference

The sidecar runs on a remote GPU box (pythondev, RTX 3090). It is NOT run locally.
See CLAUDE.md "Remote sidecar (canonical)" and workflow/remote-setup/REMOTE-SIDECAR.md for connection details.
Model: huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated (NF4, SDPA mandatory, no flash-attn).
Gemma-4-E2B-it is rejected -- do not reference it.

# feature_map_derive

Derives schema stamps, status counts, lightweight I-39 finish cross-checks, and composition edges without modifying the feature map. It uses only the Python standard library.

```sh
python .elai_cc/tools/feature_map_derive.py --json
python .elai_cc/tools/feature_map_derive.py --check
```

Use `--repo-root`, `--feature-map`, and `--elai-map` to override defaults. `--check` returns 1 when disagreements are found and 2 when the feature map is missing.

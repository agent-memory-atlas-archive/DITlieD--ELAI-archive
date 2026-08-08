"""Offline, deterministic router training from verifier-labeled ELAI journal data.

This module deliberately produces artifacts only; runtime model promotion and ledger
writes remain Rust/ledger responsibilities in the later integration slice.
"""
import argparse
import hashlib
import json
import math
import os
import sqlite3
import sys
import time
from pathlib import Path


class RouterTrainer:
    """Train a small logistic classifier without writing trial-ledger rows."""

    def __init__(self, seed=0, min_labeled=3):
        self.seed = int(seed)
        self.min_labeled = int(min_labeled)
        self.feature_names = ["bias", "route_hash", "score", "details_len", "cost_usd", "feature_0", "feature_1"]

    @staticmethod
    def _digest(data):
        try:
            import blake3
            return "blake3", blake3.blake3(data).hexdigest()
        except ImportError:
            return "sha256", hashlib.sha256(data).hexdigest()

    @staticmethod
    def _number(value, default=0.0):
        try:
            return float(value)
        except (TypeError, ValueError):
            return default

    @classmethod
    def _features(cls, row):
        route = str(row.get("route", ""))
        details = row.get("details", {})
        if not isinstance(details, (dict, list, str)):
            details = {}
        detail_text = details if isinstance(details, str) else json.dumps(details, sort_keys=True, separators=(",", ":"))
        digest = hashlib.sha256(route.encode("utf-8")).digest()
        fv = row.get("feature_vec", [])
        if not isinstance(fv, list):
            fv = []
        return [1.0, digest[0] / 255.0, cls._number(row.get("score")),
                min(len(detail_text), 10000) / 1000.0, cls._number(row.get("cost_usd")),
                cls._number(fv[0]) if fv else 0.0, cls._number(fv[1]) if len(fv) > 1 else 0.0]

    @staticmethod
    def _json_payload(payload):
        if isinstance(payload, memoryview):
            payload = payload.tobytes()
        if isinstance(payload, bytes):
            try:
                return json.loads(payload.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                try:
                    import msgpack
                    return msgpack.unpackb(payload, raw=False)
                except (ImportError, ValueError, TypeError):
                    return {}
        if isinstance(payload, str):
            try:
                return json.loads(payload)
            except json.JSONDecodeError:
                return {}
        return payload if isinstance(payload, dict) else {}

    @classmethod
    def _from_jsonl(cls, path):
        raw = Path(path).read_bytes()
        rows = []
        for line in raw.splitlines():
            if line.strip():
                try:
                    item = json.loads(line)
                    if isinstance(item, dict) and "verifier_pass" in item:
                        rows.append(item)
                except json.JSONDecodeError:
                    continue
        return rows, raw

    @classmethod
    def _from_ssb(cls, path):
        raw = Path(path).read_bytes()
        rows_by_session = {}
        outcomes = {}
        uri = "file:" + str(Path(path).resolve()) + "?mode=ro"
        with sqlite3.connect(uri, uri=True) as db:
            query = "SELECT session_id, action_type, payload FROM ssb ORDER BY seq"
            for session_id, action_type, payload in db.execute(query):
                event = cls._json_payload(payload)
                event = event if isinstance(event, dict) else {}
                sid = str(session_id or event.get("session_id", ""))
                if action_type == "route_decision_event":
                    rows_by_session[sid] = {"session_id": sid, "route": event.get("route", event.get("predicted_lane", "")),
                                             "score": event.get("score", 0), "details": event.get("details_json", event.get("details", {})),
                                             "feature_vec": event.get("feature_vec", [])}
                elif action_type in ("verifier_consensus_pass", "verifier_consensus_fail"):
                    outcomes[sid] = action_type.endswith("pass")
                elif action_type == "reflect_audit_verdict" and "verifier_pass" in event:
                    outcomes[sid] = bool(event["verifier_pass"])
        rows = []
        for sid, row in rows_by_session.items():
            if sid in outcomes:
                row["verifier_pass"] = outcomes[sid]
                rows.append(row)
        return rows, raw

    @classmethod
    def _sigmoid(cls, value):
        value = max(-40.0, min(40.0, value))
        return 1.0 / (1.0 + math.exp(-value))

    def _fit(self, rows):
        weights = [0.0] * len(self.feature_names)
        # Fixed-order batch gradient descent makes the seed part of the artifact contract
        # while keeping training reproducible across Python versions.
        for _ in range(500):
            gradient = [0.0] * len(weights)
            for row in rows:
                x = self._features(row)
                prediction = self._sigmoid(sum(w * v for w, v in zip(weights, x)))
                error = prediction - (1.0 if row["verifier_pass"] else 0.0)
                for i, value in enumerate(x):
                    gradient[i] += error * value
            for i in range(len(weights)):
                weights[i] -= 0.08 * gradient[i] / max(1, len(rows))
        return weights

    def _evaluate(self, rows, weights):
        over = 0
        misroute = 0
        comparable = 0
        for row in rows:
            probability = self._sigmoid(sum(w * v for w, v in zip(weights, self._features(row))))
            predicted_lane = "cheap" if probability >= 0.5 else "cloud"
            actual_lane = str(row.get("route", "")).lower()
            if actual_lane in ("cloud", "deep", "expensive") and row["verifier_pass"] and predicted_lane == "cheap":
                over += 1
            if row["verifier_pass"]:
                comparable += 1
                if predicted_lane != actual_lane:
                    misroute += 1
        return (over / len(rows) if rows else "n/a", misroute / comparable if comparable else "n/a")

    @staticmethod
    def _toml_value(value):
        if value is None:
            return "\"\""
        if isinstance(value, bool):
            return "true" if value else "false"
        if isinstance(value, (int, float)):
            return repr(value)
        if isinstance(value, list):
            return "[" + ", ".join(RouterTrainer._toml_value(v) for v in value) + "]"
        return json.dumps(str(value))

    def _write_toml(self, path, values):
        lines = [f"{key} = {self._toml_value(value)}" for key, value in values.items()]
        Path(path).write_text("\n".join(lines) + "\n", encoding="utf-8")

    def train(self, rows, input_bytes, out_dir):
        out = Path(out_dir)
        out.mkdir(parents=True, exist_ok=True)
        algorithm, digest = self._digest(input_bytes)
        base = {"seed": self.seed, "input_blake3": digest, "input_hash_algorithm": algorithm,
                "n_labeled": len(rows), "baseline": "H-7/E-39 heuristic"}
        if len(rows) < self.min_labeled:
            report = dict(base, status="REFUSED", reason=f"only {len(rows)} labeled rows; need at least {self.min_labeled}",
                          over_escalation_rate="n/a", mis_route_rate="n/a", beats_baseline=None,
                          notes="No model artifact was written; insufficient verifier-labeled runs.")
            self._write_toml(out / "eval_report.toml", report)
            return False
        weights = self._fit(rows)
        over, misroute = self._evaluate(rows, weights)
        self._write_toml(out / "router_model.toml", {"status": "trained", "seed": self.seed,
            "feature_names": self.feature_names, "weights": weights, "created_ts": int(time.time()),
            "input_blake3": digest, "input_hash_algorithm": algorithm})
        self._write_toml(out / "eval_report.toml", dict(base, status="ok", over_escalation_rate=over,
            mis_route_rate=misroute, beats_baseline=None,
            notes="Rates are measured on this labeled input; baseline comparison requires a held-out batch."))
        return True


def _synthetic_rows():
    return [{"session_id": str(i), "route": "cheap" if i % 2 else "cloud", "score": i / 10,
             "details": {"synthetic": True}, "verifier_pass": i != 2,
             "cost_usd": 0.01 * i, "feature_vec": [i / 10, 1 - i / 10]} for i in range(4)]


def _arguments():
    parser = argparse.ArgumentParser(description="Offline router trainer. Use --jsonl for JSONL labels, or --ssb for read-only ssb.db; no dependencies are added.")
    parser.add_argument("--ssb", help="read-only SSB sqlite database")
    parser.add_argument("--jsonl", help="primary labeled JSONL input (one object per line)")
    parser.add_argument("--out-dir", default=".elai_cc/training/router/out")
    parser.add_argument("--seed", type=int, default=int(os.environ.get("ELAI_SEED", "0")))
    parser.add_argument("--min-labeled", type=int, default=3)
    parser.add_argument("--self-check", action="store_true")
    return parser.parse_args()


if __name__ == "__main__":
    args = _arguments()
    trainer = RouterTrainer(args.seed, args.min_labeled)
    if args.self_check:
        trainer.train(_synthetic_rows(), b"self-check", Path(args.out_dir) / "self_check")
    elif args.jsonl:
        rows, raw = trainer._from_jsonl(args.jsonl)
        trainer.train(rows, raw, args.out_dir)
    elif args.ssb:
        rows, raw = trainer._from_ssb(args.ssb)
        trainer.train(rows, raw, args.out_dir)
    else:
        trainer.train([], b"", args.out_dir)

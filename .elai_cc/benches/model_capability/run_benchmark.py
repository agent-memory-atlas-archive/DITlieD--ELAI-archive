"""ELAI Baseline Measurement Harness (Step A-1).

Measures real model capabilities on real hardware before any token-efficiency
logic depends on assumptions. Talks to the sidecar HTTP API.

Usage:
    python run_benchmark.py [--config bench_config.toml] [--measurement <name>]
"""

import argparse
import json
import math
import os
import re
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

SCRIPT_DIR = Path(__file__).resolve().parent
DEFAULT_CONFIG = SCRIPT_DIR / "bench_config.toml"
FIXTURES_DIR = SCRIPT_DIR / "fixtures"

REQUEST_TIMEOUT_S = 180
SIDECAR_PORT = 8102
SIDECAR_ENDPOINT = f"http://127.0.0.1:{SIDECAR_PORT}"

# Attention curve tasks (simple, deterministic)
ATTENTION_TASKS = [
    {
        "id": "attn_reverse",
        "prompt": "Write a Rust function: fn reverse(s: &str) -> String. Output ONLY the function, no explanation.",
        "validator": lambda code: "fn reverse" in code and (".rev()" in code or "chars()" in code),
    },
    {
        "id": "attn_fibonacci",
        "prompt": "Write a Rust function: fn fibonacci(n: u32) -> u64 that returns the nth Fibonacci number (0-indexed, fib(0)=0, fib(1)=1). Output ONLY the function, no explanation.",
        "validator": lambda code: "fn fibonacci" in code and ("u64" in code or "u32" in code),
    },
    {
        "id": "attn_palindrome",
        "prompt": "Write a Rust function: fn is_palindrome(s: &str) -> bool (case-insensitive). Output ONLY the function, no explanation.",
        "validator": lambda code: "fn is_palindrome" in code and "bool" in code,
    },
    {
        "id": "attn_max",
        "prompt": "Write a Rust function: fn max_element(v: &[i32]) -> Option<i32>. Output ONLY the function, no explanation.",
        "validator": lambda code: "fn max_element" in code and "Option" in code,
    },
    {
        "id": "attn_count_words",
        "prompt": "Write a Rust function: fn count_words(s: &str) -> usize that counts words split by whitespace. Output ONLY the function, no explanation.",
        "validator": lambda code: "fn count_words" in code and "usize" in code,
    },
]

# Hallucination test prompts (factual questions with known answers)
HALLUCINATION_PROMPTS = [
    {"q": "What is the capital of Australia?", "correct": "canberra", "common_wrong": "sydney"},
    {"q": "Who wrote 'To Kill a Mockingbird'?", "correct": "harper lee", "common_wrong": ""},
    {"q": "What is the chemical symbol for gold?", "correct": "au", "common_wrong": ""},
    {"q": "What planet is closest to the sun?", "correct": "mercury", "common_wrong": "venus"},
    {"q": "How many bones does an adult human have?", "correct": "206", "common_wrong": ""},
    {"q": "What year did World War I begin?", "correct": "1914", "common_wrong": ""},
    {"q": "What is the speed of light in km/s (approximately)?", "correct": "300000", "common_wrong": ""},
    {"q": "Do we only use 10% of our brain?", "correct_contains": "myth", "incorrect_contains": "yes"},
    {"q": "Is the Great Wall of China visible from space with the naked eye?", "correct_contains": "no", "incorrect_contains": "yes, it"},
    {"q": "How long must you wait to file a missing person report?", "correct_contains": "immediat", "incorrect_contains": "24 hour"},
    {"q": "What causes the seasons on Earth?", "correct_contains": "tilt", "incorrect_contains": "distance from the sun"},
    {"q": "What is the largest organ in the human body?", "correct": "skin", "common_wrong": "liver"},
    {"q": "Who painted the Mona Lisa?", "correct": "leonardo", "common_wrong": ""},
    {"q": "What is the boiling point of water in Celsius at sea level?", "correct": "100", "common_wrong": ""},
    {"q": "How many continents are there?", "correct": "7", "common_wrong": ""},
]


# ---------------------------------------------------------------------------
# HTTP helpers
# ---------------------------------------------------------------------------

try:
    import httpx
    _HTTP_CLIENT = None

    def get_client():
        global _HTTP_CLIENT
        if _HTTP_CLIENT is None:
            _HTTP_CLIENT = httpx.Client(timeout=REQUEST_TIMEOUT_S)
        return _HTTP_CLIENT
except ImportError:
    httpx = None

try:
    import requests as _requests_mod
except ImportError:
    _requests_mod = None


def http_post(url: str, json_data: dict, timeout: int = REQUEST_TIMEOUT_S) -> dict:
    """POST JSON to URL, return parsed response."""
    if httpx is not None:
        r = get_client().post(url, json=json_data, timeout=timeout)
        r.raise_for_status()
        return r.json()
    elif _requests_mod is not None:
        r = _requests_mod.post(url, json=json_data, timeout=timeout)
        r.raise_for_status()
        return r.json()
    else:
        # Fallback to urllib
        import urllib.request
        data = json.dumps(json_data).encode()
        req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read())


def http_get(url: str, timeout: int = 10) -> dict:
    if httpx is not None:
        r = get_client().get(url, timeout=timeout)
        r.raise_for_status()
        return r.json()
    elif _requests_mod is not None:
        r = _requests_mod.get(url, timeout=timeout)
        r.raise_for_status()
        return r.json()
    else:
        import urllib.request
        with urllib.request.urlopen(url, timeout=timeout) as resp:
            return json.loads(resp.read())


# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

def load_config(path: Path) -> dict:
    """Load TOML config. Falls back to simple parser if tomllib unavailable."""
    try:
        import tomllib
        with open(path, "rb") as f:
            return tomllib.load(f)
    except ImportError:
        try:
            import tomli
            with open(path, "rb") as f:
                return tomli.load(f)
        except ImportError:
            # Minimal fallback parser
            return _parse_toml_simple(path)


def _parse_toml_simple(path: Path) -> dict:
    """Dead-simple TOML parser for flat sections with string/int/float/bool/list values."""
    config: dict = {}
    section = config
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("["):
                key = line.strip("[]").strip()
                parts = key.split(".")
                section = config
                for p in parts:
                    section = section.setdefault(p, {})
                continue
            if "=" in line:
                k, v = line.split("=", 1)
                k = k.strip()
                v = v.strip()
                if v.startswith("["):
                    v = json.loads(v)
                elif v in ("true", "false"):
                    v = v == "true"
                elif v.startswith('"'):
                    v = v.strip('"')
                else:
                    try:
                        v = int(v)
                    except ValueError:
                        try:
                            v = float(v)
                        except ValueError:
                            pass
                section[k] = v
    return config


# ---------------------------------------------------------------------------
# Sidecar communication
# ---------------------------------------------------------------------------

def check_sidecar_health(endpoint: str = SIDECAR_ENDPOINT) -> bool:
    try:
        resp = http_get(f"{endpoint}/health", timeout=5)
        return resp.get("status") == "healthy"
    except Exception:
        return False


def get_sidecar_info(endpoint: str = SIDECAR_ENDPOINT) -> dict:
    try:
        return http_get(f"{endpoint}/v1/models", timeout=5)
    except Exception:
        return {}


def complete(
    prompt: str,
    max_tokens: int = 512,
    temperature: float = 0.0,
    endpoint: str = SIDECAR_ENDPOINT,
    timeout: int = REQUEST_TIMEOUT_S,
) -> tuple[str, dict]:
    """Send an inference request via /v1/messages (Anthropic format).

    Uses the messages endpoint because Gemma 4's chat template requires
    proper message formatting. The /v1/completions endpoint's raw prompt
    format causes the model to emit only the turn-end token.
    """
    payload = {
        "model": "elai-local",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    t0 = time.time()
    resp = http_post(f"{endpoint}/v1/messages", payload, timeout=timeout)
    elapsed = time.time() - t0

    text = ""
    for block in resp.get("content", []):
        if block.get("type") == "text":
            text += block.get("text", "")

    # Map usage to a compatible format
    usage = resp.get("usage", {})
    resp["usage"] = {
        "prompt_tokens": usage.get("input_tokens", 0),
        "completion_tokens": usage.get("output_tokens", 0),
        "total_tokens": usage.get("input_tokens", 0) + usage.get("output_tokens", 0),
    }
    resp["_elapsed_s"] = elapsed
    return text.strip(), resp


def message(
    user_content: str,
    system: str = "",
    max_tokens: int = 512,
    temperature: float = 0.0,
    endpoint: str = SIDECAR_ENDPOINT,
    timeout: int = REQUEST_TIMEOUT_S,
) -> tuple[str, dict]:
    """Send an Anthropic-style message request."""
    payload = {
        "model": "elai-local",
        "messages": [{"role": "user", "content": user_content}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    if system:
        payload["system"] = system

    t0 = time.time()
    resp = http_post(f"{endpoint}/v1/messages", payload, timeout=timeout)
    elapsed = time.time() - t0

    text = ""
    for block in resp.get("content", []):
        if block.get("type") == "text":
            text += block.get("text", "")

    resp["_elapsed_s"] = elapsed
    return text.strip(), resp


# ---------------------------------------------------------------------------
# Code extraction
# ---------------------------------------------------------------------------

def extract_code(text: str, language: str = "rust") -> str:
    """Extract code from model output, handling markdown fences and raw code."""
    # Try markdown code blocks first
    patterns = [
        rf"```{language}\s*\n(.*?)```",
        r"```\s*\n(.*?)```",
        rf"```{language}\s*\n(.*?)$",
    ]
    for pat in patterns:
        m = re.search(pat, text, re.DOTALL)
        if m:
            return m.group(1).strip()

    # If the text looks like raw code (starts with fn/def/class/pub), use it directly
    stripped = text.strip()
    code_starters = ["fn ", "pub fn ", "def ", "class ", "impl ", "struct ", "use ", "async fn "]
    for starter in code_starters:
        if stripped.startswith(starter):
            return stripped

    # Last resort: return everything
    return stripped


# ---------------------------------------------------------------------------
# Validation helpers
# ---------------------------------------------------------------------------

def validate_python_code(code: str, test_code: str, timeout_s: int = 10) -> tuple[bool, str]:
    """Execute Python code + test assertions in a subprocess. Returns (passed, error_msg)."""
    full_code = code + "\n\n" + test_code
    try:
        result = subprocess.run(
            [sys.executable, "-c", full_code],
            capture_output=True, text=True, timeout=timeout_s,
        )
        if result.returncode == 0:
            return True, ""
        return False, result.stderr[:500]
    except subprocess.TimeoutExpired:
        return False, "timeout"
    except Exception as e:
        return False, str(e)[:500]


def validate_rust_syntax(code: str) -> bool:
    """Quick check if Rust code has basic syntactic validity."""
    # Check balanced braces
    depth = 0
    for ch in code:
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
        if depth < 0:
            return False
    if depth != 0:
        return False
    # Check for fn keyword
    if "fn " not in code and "struct " not in code and "impl " not in code:
        return False
    return True


def validate_rust_compile(code: str, timeout_s: int = 30) -> tuple[bool, str]:
    """Compile Rust code to check validity. Returns (compiled, error)."""
    # Wrap in main if needed
    if "fn main" not in code:
        wrapped = f"#[allow(dead_code, unused)]\n{code}\nfn main() {{}}"
    else:
        wrapped = code

    with tempfile.NamedTemporaryFile(suffix=".rs", mode="w", delete=False) as f:
        f.write(wrapped)
        f.flush()
        tmp_path = f.name

    try:
        out_path = tmp_path.replace(".rs", ".exe" if sys.platform == "win32" else "")
        result = subprocess.run(
            ["rustc", "--edition", "2021", "-o", out_path, tmp_path],
            capture_output=True, text=True, timeout=timeout_s,
        )
        # Clean up output binary
        if os.path.exists(out_path):
            os.unlink(out_path)
        if result.returncode == 0:
            return True, ""
        return False, result.stderr[:500]
    except FileNotFoundError:
        return False, "rustc not found"
    except subprocess.TimeoutExpired:
        return False, "compile timeout"
    except Exception as e:
        return False, str(e)[:500]
    finally:
        os.unlink(tmp_path)


# ---------------------------------------------------------------------------
# Padding generator
# ---------------------------------------------------------------------------

PADDING_CODE_SNIPPETS = [
    "fn compute_hash(data: &[u8]) -> u64 { let mut h: u64 = 0; for &b in data { h = h.wrapping_mul(31).wrapping_add(b as u64); } h }",
    "fn is_sorted(v: &[i32]) -> bool { v.windows(2).all(|w| w[0] <= w[1]) }",
    "fn sum_squares(n: u64) -> u64 { (1..=n).map(|i| i * i).sum() }",
    "fn capitalize(s: &str) -> String { let mut c = s.chars(); match c.next() { None => String::new(), Some(f) => f.to_uppercase().collect::<String>() + c.as_str() } }",
    "fn gcd(a: u64, b: u64) -> u64 { if b == 0 { a } else { gcd(b, a % b) } }",
    "fn count_bits(n: u32) -> u32 { let mut count = 0; let mut n = n; while n > 0 { count += n & 1; n >>= 1; } count }",
    "fn clamp(val: f64, min: f64, max: f64) -> f64 { if val < min { min } else if val > max { max } else { val } }",
    "fn binary_to_string(n: u32) -> String { format!(\"{:b}\", n) }",
    "fn interleave(a: &[i32], b: &[i32]) -> Vec<i32> { a.iter().zip(b.iter()).flat_map(|(&x, &y)| vec![x, y]).collect() }",
    "fn remove_duplicates(v: &mut Vec<i32>) { v.sort(); v.dedup(); }",
    "fn transpose(matrix: &[Vec<i32>]) -> Vec<Vec<i32>> { if matrix.is_empty() { return vec![]; } let cols = matrix[0].len(); (0..cols).map(|j| matrix.iter().map(|row| row[j]).collect()).collect() }",
    "fn dot_product(a: &[f64], b: &[f64]) -> f64 { a.iter().zip(b.iter()).map(|(x, y)| x * y).sum() }",
    "fn running_sum(v: &[i32]) -> Vec<i32> { let mut sum = 0; v.iter().map(|&x| { sum += x; sum }).collect() }",
    "fn chunk_string(s: &str, size: usize) -> Vec<String> { s.as_bytes().chunks(size).map(|c| String::from_utf8_lossy(c).into_owned()).collect() }",
    "fn zip_with_index(v: &[&str]) -> Vec<(usize, String)> { v.iter().enumerate().map(|(i, s)| (i, s.to_string())).collect() }",
    "fn manhattan_distance(p1: (i32, i32), p2: (i32, i32)) -> i32 { (p1.0 - p2.0).abs() + (p1.1 - p2.1).abs() }",
    "fn safe_divide(a: f64, b: f64) -> Option<f64> { if b == 0.0 { None } else { Some(a / b) } }",
    "fn repeat_string(s: &str, n: usize) -> String { s.repeat(n) }",
    "fn median(v: &mut Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); let mid = v.len() / 2; if v.len() % 2 == 0 { (v[mid-1] + v[mid]) / 2.0 } else { v[mid] } }",
    "fn is_anagram(a: &str, b: &str) -> bool { let mut a: Vec<char> = a.to_lowercase().chars().collect(); let mut b: Vec<char> = b.to_lowercase().chars().collect(); a.sort(); b.sort(); a == b }",
]


def generate_padding(target_chars: int) -> str:
    """Generate padding code of approximately target_chars characters."""
    lines = []
    total = 0
    idx = 0
    while total < target_chars:
        snippet = PADDING_CODE_SNIPPETS[idx % len(PADDING_CODE_SNIPPETS)]
        # Add a comment to make each repetition unique
        line = f"// padding block {idx}\n{snippet}\n"
        lines.append(line)
        total += len(line)
        idx += 1
    return "\n".join(lines)[:target_chars]


# ---------------------------------------------------------------------------
# Measurement 1: Attention Quality Curve
# ---------------------------------------------------------------------------

def measure_attention_curve(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Measure how attention quality degrades with context size."""
    print("\n=== MEASUREMENT 1: Attention Quality Curve ===")
    attn_cfg = config.get("attention", {})
    context_sizes = attn_cfg.get("context_sizes", [512, 1024, 2048, 4096, 8192])
    repeats = attn_cfg.get("task_repeats", 3)
    threshold = attn_cfg.get("pass_threshold", 0.8)

    results = {}
    attention_ceiling = 0

    # Track consecutive failures to early-exit when model can't handle context
    consecutive_failures = 0
    MAX_CONSECUTIVE_FAILURES = 5  # skip remaining sizes after 5 consecutive fails

    for ctx_size in context_sizes:
        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES:
            print(f"  Skipping ctx={ctx_size} (model failed {MAX_CONSECUTIVE_FAILURES} consecutive requests)")
            results[ctx_size] = {"pass_rate": 0.0, "passes": 0, "total": 0, "skipped": True}
            continue

        passes = 0
        total = 0
        timings = []
        # First request at each context size is slow (CUDA graph recompilation).
        # Use generous timeout for small contexts, shorter for large (known to fail).
        ctx_timeout = min(REQUEST_TIMEOUT_S, 240 if ctx_size <= 2048 else 120)

        for rep in range(repeats):
            for task in ATTENTION_TASKS:
                # Build prompt with padding before the task
                task_prompt = task["prompt"]
                # Rough estimate: 4 chars per token
                system_tokens_est = 50
                task_chars_est = len(task_prompt)
                padding_chars = max(0, (ctx_size - system_tokens_est) * 4 - task_chars_est)

                if padding_chars > 0:
                    padding = generate_padding(padding_chars)
                    full_prompt = f"You are a code generator. Output ONLY the function body, no explanation.\n\n{padding}\n\n{task_prompt}"
                else:
                    full_prompt = f"You are a code generator. Output ONLY the function body, no explanation.\n\n{task_prompt}"

                try:
                    t0 = time.time()
                    text, resp = complete(full_prompt, max_tokens=128, temperature=0.0, endpoint=endpoint, timeout=ctx_timeout)
                    elapsed = time.time() - t0
                    timings.append(elapsed)

                    code = extract_code(text, "rust")
                    # Validate: basic syntax + task-specific check
                    syntax_ok = validate_rust_syntax(code)
                    task_ok = task["validator"](code) if syntax_ok else False

                    if syntax_ok and task_ok:
                        passes += 1
                    total += 1

                    status = "PASS" if (syntax_ok and task_ok) else "FAIL"
                    input_tokens = resp.get("usage", {}).get("prompt_tokens", 0)
                    print(f"  ctx={ctx_size} rep={rep} task={task['id']} input_tok={input_tokens} {status} ({elapsed:.1f}s)")

                except Exception as e:
                    total += 1
                    print(f"  ctx={ctx_size} rep={rep} task={task['id']} ERROR: {e}")

        pass_rate = passes / total if total > 0 else 0.0
        avg_time = sum(timings) / len(timings) if timings else 0
        results[ctx_size] = {
            "pass_rate": round(pass_rate, 3),
            "passes": passes,
            "total": total,
            "avg_time_s": round(avg_time, 2),
        }
        print(f"  --> ctx={ctx_size}: pass_rate={pass_rate:.1%} avg_time={avg_time:.1f}s")

        if pass_rate >= threshold:
            attention_ceiling = ctx_size
            consecutive_failures = 0
        else:
            consecutive_failures += total  # count all failures at this size

    return {
        "context_sizes": results,
        "attention_ceiling": attention_ceiling,
        "pass_threshold": threshold,
    }


# ---------------------------------------------------------------------------
# Measurement 2: Retry Success Rate
# ---------------------------------------------------------------------------

def measure_retry_curve(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Measure how retry count affects success rate on coding tasks."""
    print("\n=== MEASUREMENT 2: Retry Success Rate ===")
    retry_cfg = config.get("retry", {})
    max_retries = retry_cfg.get("max_retries", 5)
    temperatures = retry_cfg.get("temperatures", [0.0, 0.3, 0.5, 0.1, 0.7])

    # Load a subset of tasks for retry measurement
    tasks = _load_fixture_tasks()[:25]  # Use first 25 tasks

    cumulative_pass = {i + 1: 0 for i in range(max_retries)}
    task_results = []

    for task in tasks:
        lang = task.get("language", "rust")
        desc = task["description"]
        sig = task.get("signature", "")
        file_ctx = task.get("file_context", "")

        prompt_base = f"Write the following code. Output ONLY the code, no explanation.\n\n{desc}\nSignature: {sig}"
        if file_ctx:
            prompt_base += f"\nContext: {file_ctx}"

        passed_at = None
        last_error = ""

        for attempt in range(max_retries):
            temp = temperatures[attempt] if attempt < len(temperatures) else 0.3

            if attempt == 0:
                prompt = prompt_base
            elif attempt == 1:
                prompt = prompt_base + "\nBe precise and concise."
            elif attempt == 2 and last_error:
                prompt = prompt_base + f"\nPrevious attempt had this error: {last_error}\nFix this specific issue."
            else:
                prompt = prompt_base + f"\nAttempt {attempt + 1}. Try a different approach."

            try:
                text, resp = complete(prompt, max_tokens=256, temperature=temp, endpoint=endpoint)
                code = extract_code(text, lang)

                if lang == "python":
                    test_code = _build_python_test(task)
                    if test_code:
                        passed, err = validate_python_code(code, test_code)
                    else:
                        # Fallback: syntax check
                        try:
                            compile(code, "<test>", "exec")
                            passed, err = True, ""
                        except SyntaxError as se:
                            passed, err = False, str(se)
                elif lang == "rust":
                    passed, err = validate_rust_compile(code)
                else:
                    passed, err = True, ""  # Skip validation for other langs

                if passed:
                    passed_at = attempt + 1
                    break
                else:
                    last_error = err

            except Exception as e:
                last_error = str(e)[:200]

        # Record results
        for i in range(max_retries):
            if passed_at is not None and passed_at <= i + 1:
                cumulative_pass[i + 1] += 1

        task_results.append({
            "id": task["id"],
            "passed_at": passed_at,
            "total_attempts": (passed_at or max_retries),
        })
        status = f"PASS@{passed_at}" if passed_at else "FAIL"
        print(f"  {task['id']}: {status}")

    total_tasks = len(tasks)
    retry_curve = {}
    for i in range(max_retries):
        rate = cumulative_pass[i + 1] / total_tasks if total_tasks > 0 else 0
        retry_curve[i + 1] = round(rate, 3)

    # Find optimal retries (diminishing returns < 2%)
    optimal = 1
    for i in range(1, max_retries):
        delta = retry_curve.get(i + 1, 0) - retry_curve.get(i, 0)
        if delta < 0.02:
            optimal = i
            break
        optimal = i + 1

    return {
        "retry_curve": retry_curve,
        "optimal_retries": optimal,
        "task_results": task_results,
        "total_tasks": total_tasks,
    }


# ---------------------------------------------------------------------------
# Measurement 3: Few-Shot vs Zero-Shot
# ---------------------------------------------------------------------------

def measure_few_shot(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Compare zero-shot vs few-shot performance."""
    print("\n=== MEASUREMENT 3: Few-Shot vs Zero-Shot ===")
    tasks = _load_fixture_tasks()
    # Filter to tasks with few_shot_example
    tasks_with_examples = [t for t in tasks if t.get("few_shot_example")][:20]

    zero_shot_pass = 0
    few_shot_pass = 0
    few_shot_token_costs = []
    total = 0

    for task in tasks_with_examples:
        lang = task.get("language", "rust")
        desc = task["description"]
        sig = task.get("signature", "")
        example = task["few_shot_example"]

        # Zero-shot
        prompt_zero = f"Write the following code. Output ONLY the code, no explanation.\n{desc}\nSignature: {sig}"
        # Few-shot
        ex_desc = example["description"]
        ex_sig = example["signature"]
        ex_sol = example["solution"]
        prompt_few = (
            f"Here is an example of a similar task:\n"
            f"Task: {ex_desc}\nSignature: {ex_sig}\nSolution:\n{ex_sol}\n\n"
            f"Now write the following code. Output ONLY the code, no explanation.\n{desc}\nSignature: {sig}"
        )

        # Run zero-shot
        try:
            text_z, resp_z = complete(prompt_zero, max_tokens=256, temperature=0.0, endpoint=endpoint)
            code_z = extract_code(text_z, lang)
            if lang == "rust":
                pass_z, _ = validate_rust_compile(code_z)
            elif lang == "python":
                test_code = _build_python_test(task)
                if test_code:
                    pass_z, _ = validate_python_code(code_z, test_code)
                else:
                    try:
                        compile(code_z, "<t>", "exec")
                        pass_z = True
                    except SyntaxError:
                        pass_z = False
            else:
                pass_z = True
        except Exception:
            pass_z = False

        # Run few-shot
        try:
            text_f, resp_f = complete(prompt_few, max_tokens=256, temperature=0.0, endpoint=endpoint)
            code_f = extract_code(text_f, lang)
            if lang == "rust":
                pass_f, _ = validate_rust_compile(code_f)
            elif lang == "python":
                test_code = _build_python_test(task)
                if test_code:
                    pass_f, _ = validate_python_code(code_f, test_code)
                else:
                    try:
                        compile(code_f, "<t>", "exec")
                        pass_f = True
                    except SyntaxError:
                        pass_f = False
            else:
                pass_f = True

            # Estimate few-shot example token cost
            example_text = f"{ex_desc}\n{ex_sig}\n{ex_sol}"
            few_shot_token_costs.append(len(example_text) // 4)  # rough estimate
        except Exception:
            pass_f = False

        if pass_z:
            zero_shot_pass += 1
        if pass_f:
            few_shot_pass += 1
        total += 1

        z_str = "PASS" if pass_z else "FAIL"
        f_str = "PASS" if pass_f else "FAIL"
        print(f"  {task['id']}: zero={z_str} few={f_str}")

    zero_rate = zero_shot_pass / total if total > 0 else 0
    few_rate = few_shot_pass / total if total > 0 else 0
    delta = few_rate - zero_rate
    avg_token_cost = sum(few_shot_token_costs) / len(few_shot_token_costs) if few_shot_token_costs else 0

    verdict = "ADOPT" if delta > 0.10 else ("MARGINAL" if delta > 0.05 else "SKIP")

    return {
        "zero_shot_pass_rate": round(zero_rate, 3),
        "few_shot_pass_rate": round(few_rate, 3),
        "delta": round(delta, 3),
        "few_shot_token_cost": round(avg_token_cost),
        "verdict": verdict,
        "total_tasks": total,
    }


# ---------------------------------------------------------------------------
# Measurement 4: Structured vs Free-Form
# ---------------------------------------------------------------------------

def measure_structured(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Compare structured JSON output vs free-form code output."""
    print("\n=== MEASUREMENT 4: Structured vs Free-Form ===")
    tasks = _load_fixture_tasks()[:15]

    freeform_valid = 0
    freeform_correct = 0
    structured_valid = 0
    structured_correct = 0
    total = 0
    latency_diffs = []

    for task in tasks:
        lang = task.get("language", "rust")
        desc = task["description"]
        sig = task.get("signature", "")

        # Free-form
        prompt_free = f"Write the following code. Output code only.\n{desc}\nSignature: {sig}"
        # Structured (ask for JSON output)
        prompt_struct = (
            f'Output a JSON object with a "code" field containing the implementation.\n'
            f'{desc}\nSignature: {sig}\n'
            f'Example output format: {{"code": "fn example() -> bool {{ true }}"}}'
        )

        # Run free-form
        try:
            t0 = time.time()
            text_free, _ = complete(prompt_free, max_tokens=256, temperature=0.0, endpoint=endpoint)
            time_free = time.time() - t0
            code_free = extract_code(text_free, lang)
            if code_free and ("fn " in code_free or "def " in code_free):
                freeform_valid += 1
                if lang == "rust":
                    ok, _ = validate_rust_compile(code_free)
                    if ok:
                        freeform_correct += 1
                elif lang == "python":
                    try:
                        compile(code_free, "<t>", "exec")
                        freeform_correct += 1
                    except SyntaxError:
                        pass
        except Exception:
            time_free = 0

        # Run structured
        try:
            t0 = time.time()
            text_struct, _ = complete(prompt_struct, max_tokens=256, temperature=0.0, endpoint=endpoint)
            time_struct = time.time() - t0

            # Try to parse JSON
            code_struct = ""
            try:
                parsed = json.loads(text_struct)
                code_struct = parsed.get("code", "")
                structured_valid += 1
            except json.JSONDecodeError:
                # Try to extract JSON from text
                m = re.search(r'\{[^}]*"code"\s*:\s*"(.*?)"[^}]*\}', text_struct, re.DOTALL)
                if m:
                    code_struct = m.group(1).replace("\\n", "\n").replace('\\"', '"')
                    structured_valid += 1

            if code_struct:
                if lang == "rust":
                    ok, _ = validate_rust_compile(code_struct)
                    if ok:
                        structured_correct += 1
                elif lang == "python":
                    try:
                        compile(code_struct, "<t>", "exec")
                        structured_correct += 1
                    except SyntaxError:
                        pass

            if time_free > 0 and time_struct > 0:
                latency_diffs.append((time_struct - time_free) / time_free * 100)
        except Exception:
            time_struct = 0

        total += 1
        print(f"  {task['id']}: free={'valid' if freeform_valid >= total else 'invalid'} struct={'valid' if structured_valid >= total else 'invalid'}")

    fv_rate = freeform_valid / total if total > 0 else 0
    fc_rate = freeform_correct / total if total > 0 else 0
    sv_rate = structured_valid / total if total > 0 else 0
    sc_rate = structured_correct / total if total > 0 else 0
    avg_overhead = sum(latency_diffs) / len(latency_diffs) if latency_diffs else 0

    verdict = "ADOPT" if (sc_rate - fc_rate) > 0.05 else "SKIP"

    return {
        "freeform_valid_rate": round(fv_rate, 3),
        "freeform_correct_rate": round(fc_rate, 3),
        "structured_valid_rate": round(sv_rate, 3),
        "structured_correct_rate": round(sc_rate, 3),
        "latency_overhead_pct": round(avg_overhead, 1),
        "verdict": verdict,
    }


# ---------------------------------------------------------------------------
# Measurement 5: Template Fill vs Open-Ended
# ---------------------------------------------------------------------------

def measure_template(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Compare template fill-in-the-blank vs open-ended generation."""
    print("\n=== MEASUREMENT 5: Template Fill vs Open-Ended ===")
    tasks = _load_fixture_tasks()
    # Filter to tasks with templates
    tasks_with_templates = [t for t in tasks if t.get("template")][:15]

    open_correct = 0
    template_correct = 0
    open_token_counts = []
    template_token_counts = []
    total = 0

    for task in tasks_with_templates:
        lang = task.get("language", "rust")
        desc = task["description"]
        sig = task.get("signature", "")
        template = task["template"]

        # Open-ended
        prompt_open = f"Write the following code. Output ONLY the code.\n{desc}\nSignature: {sig}"
        # Template
        prompt_template = f"Fill in the blanks (replace ___BLANK_N___ with the correct code). Output ONLY the filled template, no explanation.\n\n{template}"

        # Run open-ended
        try:
            text_open, resp_open = complete(prompt_open, max_tokens=256, temperature=0.0, endpoint=endpoint)
            code_open = extract_code(text_open, lang)
            out_tokens = resp_open.get("usage", {}).get("completion_tokens", len(text_open) // 4)
            open_token_counts.append(out_tokens)

            if lang == "rust":
                ok, _ = validate_rust_compile(code_open)
                if ok:
                    open_correct += 1
            elif lang == "python":
                try:
                    compile(code_open, "<t>", "exec")
                    open_correct += 1
                except SyntaxError:
                    pass
        except Exception:
            pass

        # Run template
        try:
            text_tmpl, resp_tmpl = complete(prompt_template, max_tokens=256, temperature=0.0, endpoint=endpoint)
            code_tmpl = extract_code(text_tmpl, lang)
            out_tokens = resp_tmpl.get("usage", {}).get("completion_tokens", len(text_tmpl) // 4)
            template_token_counts.append(out_tokens)

            if lang == "rust":
                ok, _ = validate_rust_compile(code_tmpl)
                if ok:
                    template_correct += 1
            elif lang == "python":
                try:
                    compile(code_tmpl, "<t>", "exec")
                    template_correct += 1
                except SyntaxError:
                    pass
        except Exception:
            pass

        total += 1
        print(f"  {task['id']}: open={'PASS' if open_correct >= total else 'FAIL'} template={'PASS' if template_correct >= total else 'FAIL'}")

    oc_rate = open_correct / total if total > 0 else 0
    tc_rate = template_correct / total if total > 0 else 0
    avg_open_tokens = sum(open_token_counts) / len(open_token_counts) if open_token_counts else 0
    avg_tmpl_tokens = sum(template_token_counts) / len(template_token_counts) if template_token_counts else 0
    savings = (1 - avg_tmpl_tokens / avg_open_tokens) * 100 if avg_open_tokens > 0 else 0

    verdict = "ADOPT" if (tc_rate > oc_rate and savings > 20) else "MARGINAL"

    return {
        "open_ended_correct": round(oc_rate, 3),
        "template_correct": round(tc_rate, 3),
        "open_ended_avg_tokens": round(avg_open_tokens),
        "template_avg_tokens": round(avg_tmpl_tokens),
        "token_savings_pct": round(savings, 1),
        "verdict": verdict,
    }


# ---------------------------------------------------------------------------
# Measurement 6: TurboQuant Impact
# ---------------------------------------------------------------------------

def measure_turboquant_impact(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Measure TurboQuant impact on quality and capacity.

    NOTE: This test runs against the CURRENT sidecar configuration.
    To compare fp16 vs TQ4, the sidecar must be restarted with different config.
    Since the sidecar default is TQ4, we test that and note the fp16 data
    from existing compose_dola_tq_results.json.
    """
    print("\n=== MEASUREMENT 6: TurboQuant Impact ===")

    # Read existing measurements
    existing_results = SCRIPT_DIR.parent.parent / "metrics" / "compose_dola_tq_results.json"
    existing_data = {}
    if existing_results.exists():
        with open(existing_results) as f:
            existing_data = json.load(f)

    # Run attention curve subset under current config (should be TQ4)
    print("  Running attention subset with current TQ config...")
    tq_results = {}
    for ctx_size in [1024, 2048, 4096, 8192]:
        passes = 0
        total = 0
        for task in ATTENTION_TASKS[:3]:
            padding_chars = max(0, ctx_size * 4 - len(task["prompt"]) - 200)
            padding = generate_padding(padding_chars) if padding_chars > 0 else ""
            prompt = f"You are a code generator. Output ONLY the function body.\n\n{padding}\n\n{task['prompt']}"

            try:
                text, _ = complete(prompt, max_tokens=256, temperature=0.0, endpoint=endpoint)
                code = extract_code(text, "rust")
                if validate_rust_syntax(code) and task["validator"](code):
                    passes += 1
            except Exception:
                pass
            total += 1

        rate = passes / total if total > 0 else 0
        tq_results[ctx_size] = round(rate, 3)
        print(f"    ctx={ctx_size}: pass_rate={rate:.1%}")

    # Include data from existing measurements
    ctx_capacity = existing_data.get("context_capacity_12gb", {})

    return {
        "current_config": "tq4",
        "tq4_attention_results": tq_results,
        "fp16_max_context": ctx_capacity.get("fp16", "unknown"),
        "tq4_max_context": ctx_capacity.get("tq4", "unknown"),
        "tq3_max_context": ctx_capacity.get("tq3", "unknown"),
        "tq2_max_context": ctx_capacity.get("tq2", "unknown"),
        "note": "TQ4 best quality/compression ratio per prior eval (k_mse=0.00014, v_mse=0.0095)",
    }


# ---------------------------------------------------------------------------
# DoLa Comparison Test
# ---------------------------------------------------------------------------

def measure_dola_comparison(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Measure DoLa's impact on factuality and code quality."""
    print("\n=== DoLa Comparison Test ===")

    # Factuality test with hallucination-prone prompts
    results = []

    for hp in HALLUCINATION_PROMPTS[:10]:
        prompt = f"Answer concisely in 1-2 sentences: {hp['q']}"
        try:
            text, resp = complete(prompt, max_tokens=128, temperature=0.0, endpoint=endpoint)
            elapsed = resp.get("_elapsed_s", 0)

            answer_lower = text.lower()
            correct = False

            if "correct" in hp:
                correct = hp["correct"].lower() in answer_lower
            elif "correct_contains" in hp:
                correct = hp["correct_contains"].lower() in answer_lower

            has_wrong = False
            if hp.get("common_wrong"):
                has_wrong = hp["common_wrong"].lower() in answer_lower
            if hp.get("incorrect_contains"):
                has_wrong = hp["incorrect_contains"].lower() in answer_lower

            results.append({
                "question": hp["q"],
                "answer": text[:200],
                "correct": correct,
                "has_wrong_info": has_wrong,
                "elapsed_s": round(elapsed, 2),
            })

            status = "CORRECT" if correct else ("WRONG_INFO" if has_wrong else "UNCLEAR")
            print(f"  {status}: {hp['q'][:50]}... ({elapsed:.1f}s)")

        except Exception as e:
            results.append({"question": hp["q"], "error": str(e)[:100]})

    correct_count = sum(1 for r in results if r.get("correct"))
    wrong_count = sum(1 for r in results if r.get("has_wrong_info"))
    total = len(results)
    avg_time = sum(r.get("elapsed_s", 0) for r in results) / total if total > 0 else 0

    # Also test code quality with DoLa
    code_results = []
    for task in ATTENTION_TASKS:
        try:
            text, resp = complete(task["prompt"], max_tokens=256, temperature=0.0, endpoint=endpoint)
            code = extract_code(text, "rust")
            ok = validate_rust_syntax(code) and task["validator"](code)
            elapsed = resp.get("_elapsed_s", 0)
            code_results.append({"id": task["id"], "passed": ok, "elapsed_s": round(elapsed, 2)})
        except Exception:
            code_results.append({"id": task["id"], "passed": False})

    code_pass_rate = sum(1 for r in code_results if r.get("passed")) / len(code_results) if code_results else 0

    return {
        "factuality_correct": correct_count,
        "factuality_wrong": wrong_count,
        "factuality_total": total,
        "factuality_accuracy": round(correct_count / total, 3) if total > 0 else 0,
        "avg_response_time_s": round(avg_time, 2),
        "code_pass_rate": round(code_pass_rate, 3),
        "code_results": code_results,
        "factuality_results": results,
    }


# ---------------------------------------------------------------------------
# Steering Vector Test
# ---------------------------------------------------------------------------

def measure_steering(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Test steering vector computation and effect."""
    print("\n=== Steering Vector Test ===")

    # First check if steering is enabled
    try:
        info = http_get(f"{endpoint}/health")
    except Exception:
        return {"error": "sidecar unreachable"}

    # Compute a steering vector for "accurate/factual" vs "uncertain/vague"
    pairs = [
        {
            "positive": "The capital of France is Paris. This is a verified fact.",
            "negative": "The capital of France is maybe a city. I am not sure.",
            "name": "factuality",
        },
        {
            "positive": "fn validate(s: &str) -> bool { !s.is_empty() && s.len() < 256 }",
            "negative": "fn validate(s: &str) -> bool { todo!() }",
            "name": "code_completeness",
        },
    ]

    steering_results = []
    for pair in pairs:
        try:
            resp = http_post(
                f"{endpoint}/v1/compute_steering",
                {
                    "positive_prompt": pair["positive"],
                    "negative_prompt": pair["negative"],
                    "layer_idx": 9,  # ~n_layers/3 for E2B (35 layers)
                },
                timeout=30,
            )
            vector = resp.get("vector", [])
            magnitude = math.sqrt(sum(v * v for v in vector)) if vector else 0
            steering_results.append({
                "name": pair["name"],
                "magnitude": round(magnitude, 4),
                "vector_dim": len(vector),
                "computed": True,
            })
            print(f"  Computed '{pair['name']}' vector: magnitude={magnitude:.4f}, dim={len(vector)}")
        except Exception as e:
            steering_results.append({
                "name": pair["name"],
                "computed": False,
                "error": str(e)[:200],
            })
            print(f"  Failed to compute '{pair['name']}': {e}")

    return {
        "vectors_computed": steering_results,
        "note": "Steering vectors computed via ActAdd (h+ - h-). Applied via forward hook.",
    }


# ---------------------------------------------------------------------------
# Composition Test: DoLa + TQ + Steering together
# ---------------------------------------------------------------------------

def measure_composition(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Test all three features working together: DoLa + TurboQuant + Steering."""
    print("\n=== Composition Test: DoLa + TQ + Steering ===")

    # Run coding tasks and factuality tasks to see combined effect
    code_results = []
    fact_results = []

    # Coding tasks
    tasks = _load_fixture_tasks()[:20]
    for task in tasks:
        lang = task.get("language", "rust")
        desc = task["description"]
        sig = task.get("signature", "")
        prompt = f"Write the following code. Output ONLY the code, no explanation.\n{desc}\nSignature: {sig}"

        try:
            t0 = time.time()
            text, resp = complete(prompt, max_tokens=256, temperature=0.0, endpoint=endpoint)
            elapsed = time.time() - t0
            code = extract_code(text, lang)

            if lang == "rust":
                passed, _ = validate_rust_compile(code)
            elif lang == "python":
                test_code = _build_python_test(task)
                if test_code:
                    passed, _ = validate_python_code(code, test_code)
                else:
                    try:
                        compile(code, "<t>", "exec")
                        passed = True
                    except SyntaxError:
                        passed = False
            else:
                passed = True

            input_tok = resp.get("usage", {}).get("prompt_tokens", 0)
            output_tok = resp.get("usage", {}).get("completion_tokens", 0)
            meta = resp.get("elai_metadata", {})

            code_results.append({
                "id": task["id"],
                "passed": passed,
                "elapsed_s": round(elapsed, 2),
                "input_tokens": input_tok,
                "output_tokens": output_tok,
                "dola_enabled": meta.get("dola_enabled", "unknown"),
                "tq_enabled": meta.get("turboquant_enabled", "unknown"),
                "steering_enabled": meta.get("steering_enabled", "unknown"),
            })
            print(f"  {task['id']}: {'PASS' if passed else 'FAIL'} ({elapsed:.1f}s, in={input_tok} out={output_tok})")

        except Exception as e:
            code_results.append({"id": task["id"], "error": str(e)[:200]})
            print(f"  {task['id']}: ERROR {e}")

    # Factuality tasks
    for hp in HALLUCINATION_PROMPTS[:8]:
        prompt = f"Answer concisely: {hp['q']}"
        try:
            text, resp = complete(prompt, max_tokens=128, temperature=0.0, endpoint=endpoint)
            answer_lower = text.lower()
            correct = False
            if "correct" in hp:
                correct = hp["correct"].lower() in answer_lower
            elif "correct_contains" in hp:
                correct = hp["correct_contains"].lower() in answer_lower

            fact_results.append({
                "question": hp["q"],
                "correct": correct,
                "answer_preview": text[:100],
            })
        except Exception:
            fact_results.append({"question": hp["q"], "error": "failed"})

    code_pass_rate = sum(1 for r in code_results if r.get("passed")) / len(code_results) if code_results else 0
    fact_acc = sum(1 for r in fact_results if r.get("correct")) / len(fact_results) if fact_results else 0
    avg_elapsed = sum(r.get("elapsed_s", 0) for r in code_results if "elapsed_s" in r)
    avg_elapsed = avg_elapsed / len(code_results) if code_results else 0

    # Get first result's metadata to report config
    first_meta = {}
    for r in code_results:
        if "dola_enabled" in r:
            first_meta = {
                "dola_enabled": r["dola_enabled"],
                "tq_enabled": r["tq_enabled"],
                "steering_enabled": r["steering_enabled"],
            }
            break

    return {
        "config": first_meta,
        "code_pass_rate": round(code_pass_rate, 3),
        "code_total": len(code_results),
        "factuality_accuracy": round(fact_acc, 3),
        "factuality_total": len(fact_results),
        "avg_code_time_s": round(avg_elapsed, 2),
        "code_results": code_results,
        "factuality_results": fact_results,
    }


# ---------------------------------------------------------------------------
# Context Cache Test
# ---------------------------------------------------------------------------

def measure_context_cache(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Measure how fast the model processes repeated prefix (prefix caching effect)."""
    print("\n=== Context Cache Activation/Cleaning Test ===")

    # Use a fixed prefix that would be reused across baby steps
    system_prefix = (
        "You are a Rust code generator. You follow these rules:\n"
        "1. Output ONLY valid Rust code\n"
        "2. No explanations or comments\n"
        "3. Use proper error handling\n"
        "4. Follow Rust naming conventions\n"
    )

    # Measure first request (cold) vs subsequent requests (warm cache)
    prompts = [
        f"{system_prefix}\nWrite fn add(a: i32, b: i32) -> i32",
        f"{system_prefix}\nWrite fn sub(a: i32, b: i32) -> i32",
        f"{system_prefix}\nWrite fn mul(a: i32, b: i32) -> i32",
        f"{system_prefix}\nWrite fn div(a: f64, b: f64) -> Option<f64>",
        f"{system_prefix}\nWrite fn modulo(a: i32, b: i32) -> Option<i32>",
    ]

    timings = []
    for i, prompt in enumerate(prompts):
        try:
            t0 = time.time()
            text, resp = complete(prompt, max_tokens=128, temperature=0.0, endpoint=endpoint)
            elapsed = time.time() - t0
            input_tok = resp.get("usage", {}).get("prompt_tokens", 0)
            output_tok = resp.get("usage", {}).get("completion_tokens", 0)

            timings.append({
                "request_idx": i,
                "elapsed_s": round(elapsed, 3),
                "input_tokens": input_tok,
                "output_tokens": output_tok,
            })
            print(f"  Request {i}: {elapsed:.3f}s (in={input_tok}, out={output_tok})")

        except Exception as e:
            timings.append({"request_idx": i, "error": str(e)[:200]})

    # Now measure with completely different prefix (cache miss)
    different_prefix = "You are a Python data scientist. Output only code.\n"
    diff_prompts = [
        f"{different_prefix}\ndef mean(lst): ...",
        f"{different_prefix}\ndef std(lst): ...",
    ]

    cache_miss_timings = []
    for i, prompt in enumerate(diff_prompts):
        try:
            t0 = time.time()
            text, resp = complete(prompt, max_tokens=128, temperature=0.0, endpoint=endpoint)
            elapsed = time.time() - t0
            cache_miss_timings.append({
                "request_idx": i,
                "elapsed_s": round(elapsed, 3),
            })
            print(f"  Cache-miss request {i}: {elapsed:.3f}s")
        except Exception as e:
            cache_miss_timings.append({"request_idx": i, "error": str(e)[:200]})

    # Analyze
    warm_times = [t["elapsed_s"] for t in timings[1:] if "elapsed_s" in t]
    cold_time = timings[0].get("elapsed_s", 0) if timings else 0
    miss_times = [t["elapsed_s"] for t in cache_miss_timings if "elapsed_s" in t]

    return {
        "cold_first_request_s": cold_time,
        "warm_avg_s": round(sum(warm_times) / len(warm_times), 3) if warm_times else 0,
        "cache_miss_avg_s": round(sum(miss_times) / len(miss_times), 3) if miss_times else 0,
        "speedup_ratio": round(cold_time / (sum(warm_times) / len(warm_times)), 2) if warm_times and cold_time else 0,
        "timings": timings,
        "cache_miss_timings": cache_miss_timings,
    }


# ---------------------------------------------------------------------------
# Hallucination Reduction Measurement
# ---------------------------------------------------------------------------

def measure_hallucination(
    config: dict,
    endpoint: str = SIDECAR_ENDPOINT,
) -> dict:
    """Measure hallucination rate on factual and code questions."""
    print("\n=== Hallucination Reduction Measurement ===")

    # Test with all hallucination prompts
    fact_results = []
    for hp in HALLUCINATION_PROMPTS:
        prompt = f"Answer the following question accurately and concisely in 1-2 sentences:\n{hp['q']}"
        try:
            text, resp = complete(prompt, max_tokens=128, temperature=0.0, endpoint=endpoint)
            elapsed = resp.get("_elapsed_s", 0)

            answer_lower = text.lower()
            correct = False
            has_wrong = False

            if "correct" in hp:
                correct = hp["correct"].lower() in answer_lower
            elif "correct_contains" in hp:
                correct = hp["correct_contains"].lower() in answer_lower

            if hp.get("common_wrong"):
                has_wrong = hp["common_wrong"].lower() in answer_lower
            if hp.get("incorrect_contains"):
                has_wrong = hp["incorrect_contains"].lower() in answer_lower

            fact_results.append({
                "question": hp["q"],
                "answer": text[:300],
                "correct": correct,
                "hallucinated": has_wrong,
                "elapsed_s": round(elapsed, 2),
            })
            status = "CORRECT" if correct else ("HALLUCINATED" if has_wrong else "UNCLEAR")
            print(f"  {status}: {hp['q'][:50]}...")
        except Exception as e:
            fact_results.append({"question": hp["q"], "error": str(e)[:100]})

    # Code hallucination: ask for non-existent APIs
    code_hallucination_prompts = [
        "Write Rust code using the `std::collections::OrderedMap` type.",
        "Write Rust code using `Vec::remove_all()` method.",
        "Write Python code using `os.path.get_extension()` function.",
        "Write Rust code using the `tokio::spawn_blocking_sync()` function.",
        "Write Python code using `json.parse()` method.",
    ]

    code_hall_results = []
    for prompt in code_hallucination_prompts:
        try:
            text, resp = complete(
                f"Write the code as requested. If the API does not exist, say so.\n{prompt}",
                max_tokens=256, temperature=0.0, endpoint=endpoint,
            )
            # Check if model recognized the non-existent API
            text_lower = text.lower()
            recognized = any(w in text_lower for w in [
                "does not exist", "doesn't exist", "no such", "not a real",
                "not available", "there is no", "doesn't have", "does not have",
                "is not a valid", "not a standard", "not found",
            ])
            code_hall_results.append({
                "prompt": prompt,
                "recognized_fake": recognized,
                "response_preview": text[:200],
            })
            print(f"  API hallucination: {'CAUGHT' if recognized else 'MISSED'}: {prompt[:50]}...")
        except Exception as e:
            code_hall_results.append({"prompt": prompt, "error": str(e)[:100]})

    correct_count = sum(1 for r in fact_results if r.get("correct"))
    hallucinated_count = sum(1 for r in fact_results if r.get("hallucinated"))
    caught_count = sum(1 for r in code_hall_results if r.get("recognized_fake"))

    return {
        "factuality_correct": correct_count,
        "factuality_total": len(fact_results),
        "factuality_accuracy": round(correct_count / len(fact_results), 3) if fact_results else 0,
        "hallucination_count": hallucinated_count,
        "hallucination_rate": round(hallucinated_count / len(fact_results), 3) if fact_results else 0,
        "code_api_hallucination_caught": caught_count,
        "code_api_hallucination_total": len(code_hall_results),
        "code_api_hallucination_rate": round(1 - caught_count / len(code_hall_results), 3) if code_hall_results else 0,
        "fact_results": fact_results,
        "code_hall_results": code_hall_results,
    }


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------

def _load_fixture_tasks() -> list:
    tasks_file = FIXTURES_DIR / "tasks.json"
    if tasks_file.exists():
        with open(tasks_file) as f:
            return json.load(f)
    return []


def _build_python_test(task: dict) -> str:
    """Build a simple Python test string from task test_cases."""
    test_cases = task.get("test_cases", [])
    if not test_cases:
        return ""

    lines = []
    for tc in test_cases:
        if "input" in tc and "expected" in tc:
            inp = tc["input"]
            exp = tc["expected"]
            # Simple assertion
            lines.append(f"assert str({task['signature'].split('(')[0].replace('def ', '')}({inp})) == str({exp}), f'Failed for input {inp}'")
        elif "expected_len" in tc:
            inp = tc["input"]
            exp_len = tc["expected_len"]
            func_name = task['signature'].split('(')[0].replace('def ', '')
            lines.append(f"assert len({func_name}({inp})) == {exp_len}")

    return "\n".join(lines) if lines else ""


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def generate_report(
    results: dict,
    config: dict,
    output_path: Path,
    raw_path: Path,
) -> None:
    """Generate model_baseline.toml and raw JSON results."""
    print("\n=== Generating Report ===")

    # Raw JSON
    raw_path.parent.mkdir(parents=True, exist_ok=True)
    with open(raw_path, "w") as f:
        json.dump(results, f, indent=2, default=str)
    print(f"  Raw results: {raw_path}")

    # TOML report
    attn = results.get("attention_curve", {})
    retry = results.get("retry_curve", {})
    few = results.get("few_shot", {})
    struct = results.get("structured_output", {})
    tmpl = results.get("template_fill", {})
    tq = results.get("turboquant", {})
    dola = results.get("dola_comparison", {})
    comp = results.get("composition", {})
    cache = results.get("context_cache", {})
    hall = results.get("hallucination", {})

    attention_ceiling = attn.get("attention_ceiling", 0)

    # Derive model tier
    if attention_ceiling >= 24000:
        tier = "LARGE"
    elif attention_ceiling >= 12000:
        tier = "MEDIUM"
    elif attention_ceiling >= 8000:
        tier = "SMALL"
    else:
        tier = "MICRO"

    hw = config.get("hardware", {})

    lines = [
        f'# ELAI Model Baseline Report',
        f'# Generated: {datetime.now(timezone.utc).isoformat()}',
        f'',
        f'[meta]',
        f'model_name = "{config.get("model", {}).get("model_name", "unknown")}"',
        f'model_size_b = 2.3',
        f'quantization = "NF4 (bitsandbytes)"',
        f'gpu = "{hw.get("gpu_name", "unknown")} {hw.get("total_vram_mb", 0)}MB"',
        f'usable_vram_mb = {hw.get("usable_vram_mb", 0)}',
        f'benchmark_date = "{datetime.now().strftime("%Y-%m-%d")}"',
        f'benchmark_version = "1.0"',
        f'',
    ]

    # Attention curve
    if attn:
        lines.append('[attention_curve]')
        for ctx_size, data in sorted(attn.get("context_sizes", {}).items(), key=lambda x: int(x[0])):
            rate = data["pass_rate"] if isinstance(data, dict) else data
            lines.append(f'{ctx_size} = {rate}')
        lines.append(f'attention_ceiling = {attention_ceiling}')
        lines.append('')

    # Retry curve
    if retry:
        lines.append('[retry_curve]')
        for attempt, rate in sorted(retry.get("retry_curve", {}).items(), key=lambda x: int(x[0])):
            lines.append(f'{attempt} = {rate}')
        lines.append(f'optimal_retries = {retry.get("optimal_retries", 3)}')
        lines.append('')

    # Few-shot
    if few:
        lines.append('[few_shot]')
        lines.append(f'zero_shot_pass_rate = {few.get("zero_shot_pass_rate", 0)}')
        lines.append(f'few_shot_pass_rate = {few.get("few_shot_pass_rate", 0)}')
        lines.append(f'delta = {few.get("delta", 0)}')
        lines.append(f'few_shot_token_cost = {few.get("few_shot_token_cost", 0)}')
        lines.append(f'verdict = "{few.get("verdict", "unknown")}"')
        lines.append('')

    # Structured output
    if struct:
        lines.append('[structured_output]')
        lines.append(f'freeform_valid_rate = {struct.get("freeform_valid_rate", 0)}')
        lines.append(f'freeform_correct_rate = {struct.get("freeform_correct_rate", 0)}')
        lines.append(f'structured_valid_rate = {struct.get("structured_valid_rate", 0)}')
        lines.append(f'structured_correct_rate = {struct.get("structured_correct_rate", 0)}')
        lines.append(f'latency_overhead_pct = {struct.get("latency_overhead_pct", 0)}')
        lines.append(f'verdict = "{struct.get("verdict", "unknown")}"')
        lines.append('')

    # Template fill
    if tmpl:
        lines.append('[template_fill]')
        lines.append(f'open_ended_correct = {tmpl.get("open_ended_correct", 0)}')
        lines.append(f'template_correct = {tmpl.get("template_correct", 0)}')
        lines.append(f'open_ended_avg_tokens = {tmpl.get("open_ended_avg_tokens", 0)}')
        lines.append(f'template_avg_tokens = {tmpl.get("template_avg_tokens", 0)}')
        lines.append(f'token_savings_pct = {tmpl.get("token_savings_pct", 0)}')
        lines.append(f'verdict = "{tmpl.get("verdict", "unknown")}"')
        lines.append('')

    # TurboQuant
    if tq:
        lines.append('[turboquant]')
        lines.append(f'fp16_max_context = {tq.get("fp16_max_context", "unknown")}')
        lines.append(f'tq4_max_context = {tq.get("tq4_max_context", "unknown")}')
        lines.append(f'tq3_max_context = {tq.get("tq3_max_context", "unknown")}')
        lines.append(f'tq2_max_context = {tq.get("tq2_max_context", "unknown")}')
        lines.append('')

    # DoLa
    if dola:
        lines.append('[dola]')
        lines.append(f'factuality_accuracy = {dola.get("factuality_accuracy", 0)}')
        lines.append(f'code_pass_rate = {dola.get("code_pass_rate", 0)}')
        lines.append(f'avg_response_time_s = {dola.get("avg_response_time_s", 0)}')
        lines.append('')

    # Composition
    if comp:
        lines.append('[composition]')
        lines.append(f'code_pass_rate = {comp.get("code_pass_rate", 0)}')
        lines.append(f'factuality_accuracy = {comp.get("factuality_accuracy", 0)}')
        lines.append(f'avg_code_time_s = {comp.get("avg_code_time_s", 0)}')
        lines.append('')

    # Context cache
    if cache:
        lines.append('[context_cache]')
        lines.append(f'cold_first_request_s = {cache.get("cold_first_request_s", 0)}')
        lines.append(f'warm_avg_s = {cache.get("warm_avg_s", 0)}')
        lines.append(f'cache_miss_avg_s = {cache.get("cache_miss_avg_s", 0)}')
        lines.append(f'speedup_ratio = {cache.get("speedup_ratio", 0)}')
        lines.append('')

    # Hallucination
    if hall:
        lines.append('[hallucination]')
        lines.append(f'factuality_accuracy = {hall.get("factuality_accuracy", 0)}')
        lines.append(f'hallucination_rate = {hall.get("hallucination_rate", 0)}')
        lines.append(f'code_api_hallucination_rate = {hall.get("code_api_hallucination_rate", 0)}')
        lines.append('')

    # Derived values
    lines.append('[derived]')
    lines.append(f'model_tier = "{tier}"')
    lines.append(f'attention_ceiling = {attention_ceiling}')

    output_reserve = {"MICRO": 750, "SMALL": 2000, "MEDIUM": 2000, "LARGE": 4000}.get(tier, 1000)
    prompt_budget = max(0, attention_ceiling - output_reserve)
    lines.append(f'recommended_prompt_budget = {prompt_budget}')
    lines.append(f'output_reserve = {output_reserve}')
    lines.append(f'optimal_retries = {retry.get("optimal_retries", 3)}')
    lines.append(f'few_shot_recommended = {str(few.get("verdict", "") == "ADOPT").lower()}')
    lines.append(f'structured_output_recommended = {str(struct.get("verdict", "") == "ADOPT").lower()}')
    lines.append(f'template_recommended = {str(tmpl.get("verdict", "") == "ADOPT").lower()}')
    lines.append(f'swap_strategy = "single_model"')

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  Report: {output_path}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="ELAI Baseline Measurement Harness")
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--measurement", help="Run only a specific measurement (attention, retry, few_shot, structured, template, turboquant, dola, steering, composition, cache, hallucination)")
    parser.add_argument("--endpoint", default=SIDECAR_ENDPOINT)
    args = parser.parse_args()

    config = load_config(args.config)
    endpoint = args.endpoint

    # Check sidecar health
    print(f"Checking sidecar at {endpoint}...")
    for attempt in range(3):
        if check_sidecar_health(endpoint):
            print("Sidecar is healthy.")
            info = get_sidecar_info(endpoint)
            print(f"Models: {info}")
            break
        print(f"  Attempt {attempt + 1}: sidecar not ready, waiting 5s...")
        time.sleep(5)
    else:
        print("ERROR: Sidecar not reachable. Start it first:")
        print("  python -m elai_inference.server --model D:/.ELAI_workflow/.elai_cc/models/gemma-4-E2B-it --port 8100")
        sys.exit(1)

    measurements = config.get("measurements", {})
    results = {}
    run_single = args.measurement

    # Run measurements
    measurement_map = {
        "attention": ("run_attention", measure_attention_curve),
        "retry": ("run_retry", measure_retry_curve),
        "few_shot": ("run_few_shot", measure_few_shot),
        "structured": ("run_structured", measure_structured),
        "template": ("run_template", measure_template),
        "turboquant": ("run_turboquant", measure_turboquant_impact),
        "dola": ("run_dola_comparison", measure_dola_comparison),
        "steering": ("run_steering_test", measure_steering),
        "composition": ("run_composition", measure_composition),
        "cache": ("run_context_cache", measure_context_cache),
        "hallucination": ("run_hallucination", measure_hallucination),
    }

    result_keys = {
        "attention": "attention_curve",
        "retry": "retry_curve",
        "few_shot": "few_shot",
        "structured": "structured_output",
        "template": "template_fill",
        "turboquant": "turboquant",
        "dola": "dola_comparison",
        "steering": "steering",
        "composition": "composition",
        "cache": "context_cache",
        "hallucination": "hallucination",
    }

    for name, (flag, func) in measurement_map.items():
        if run_single and run_single != name:
            continue
        if not run_single and not measurements.get(flag, False):
            continue

        try:
            result = func(config, endpoint)
            results[result_keys[name]] = result
        except Exception as e:
            print(f"\nERROR in {name}: {e}")
            import traceback
            traceback.print_exc()
            results[result_keys[name]] = {"error": str(e)}

    # Generate report
    output_cfg = config.get("output", {})
    output_path = SCRIPT_DIR / output_cfg.get("path", "../../metrics/model_baseline.toml")
    raw_path = SCRIPT_DIR / output_cfg.get("raw_results_path", "../../metrics/baseline_raw_results.json")

    generate_report(results, config, output_path.resolve(), raw_path.resolve())

    print("\n=== BENCHMARK COMPLETE ===")
    print(f"Report: {output_path.resolve()}")
    print(f"Raw: {raw_path.resolve()}")


if __name__ == "__main__":
    main()

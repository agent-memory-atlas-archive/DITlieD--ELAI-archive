# Sandboxed agent code execution for ELAI

## TL;DR
- Tier C should adopt a two-pool model: hot snapshot-restore pool for high-frequency baby steps (~28ms restore, 5 MiB overhead per microVM) and a cold pool for first boots and high-risk profiles. AWS Lambda's Firecracker design and AgentSphere's production deployment confirm 28ms restore is achievable with copy-on-write snapshots [firecracker-microvm.github.io, dev.to/adwitiya, softwareseni.com].
- Keep the planned seccomp BPF profile but make BLOCK-1 argument-checked: socket() must check arg0 == AF_INET (and optionally arg1 == SOCK_STREAM); AF_UNIX/AF_INET6/AF_NETLINK/AF_PACKET must error. Docker's 2026 default profile and Chromium's sandbox both rely on argument filtering — blanket allows on socket() are the canonical Tier-C smell [docs.docker.com, kernel.org seccomp_filter].
- Drop dependency on shared-kernel-only isolation for the researcher profile. CVE-2025-31133/52565/52881 (runc, Nov 2025) demonstrate that bind-mount race conditions defeat maskedPaths even with capabilities dropped and seccomp on; researcher should run inside a Firecracker microVM (or gVisor on hosts without KVM) regardless of NETWORK_EXTRA cost [sysdig.com, cncf.io].
- Bash execution phase 3+ should target Firecracker (Rust, KVM, AWS-grade) over gVisor when KVM is available — Firecracker's ~1-2% steady-state overhead and per-VM 5 MiB beats gVisor's 10-20% syscall-tax for the kind of process/file workload baby-steps generate [northflank.com, gvisor.dev/performance].
- For Windows fallback, the AppContainer + Job-object + LowIL combo is the only path that has ~6 years of Chromium production hardening behind it; OpenAI Codex (May 2026) ships essentially the same trio. Do NOT roll a custom Windows sandbox [microsoft.com SandboxSecurityTools, winbuzzer Codex 2026].

## Key sources
| URL | What | Takeaway |
|---|---|---|
| https://firecracker-microvm.github.io/ | Firecracker official | 125 ms boot, <5 MiB per VM, 150 VMs/sec/host |
| https://dev.to/adwitiya/how-i-built-sandboxes-that-boot-in-28ms-using-firecracker-snapshots-i0k | Snapshot restore deep-dive | Cold ~1s, restore ~28ms via CoW snapshot files |
| https://gvisor.dev/docs/architecture_guide/performance/ | gVisor perf guide | ~10-20% syscall overhead, Systrap default since 2023 |
| https://www.usenix.org/system/files/hotcloud19-paper-young.pdf | True cost of containing | Per-syscall overhead in gVisor is workload-defining |
| https://www.daytona.io/ | Daytona docs | 27ms startup, dedicated kernel per sandbox, agent-native |
| https://e2b.dev/pricing | E2B pricing | $150/mo Pro, $3000/mo enterprise min, ~150-410ms cold |
| https://modal.com/blog/top-code-agent-sandbox-products | Modal sandbox catalog | Modal sandboxes run on gVisor under the hood |
| https://github.com/apple/container | apple/container | Sub-second VM-per-container on Apple silicon, Swift |
| https://blog.stackblitz.com/posts/bringing-webcontainers-to-all-browsers/ | WebContainers | 100% browser security boundary, requires COEP credentialless |
| https://www.cyera.com/research/n8scape-pyodide-sandbox-escape-9-9-critical-post-auth-rce-in-n8n-cve-2025-68668 | CVE-2025-68668 | Pyodide host-bridge = full RCE; "wasm is safe, bindings often not" |
| https://github.com/containers/bubblewrap | bubblewrap | Minimal setuid surface, requires manual hardening, --new-session for TIOCSTI |
| https://github.com/microsoft/SandboxSecurityTools/tree/main/LaunchAppContainer | MS sample | AppContainer + Job + LowIL canonical Windows pattern |
| https://docs.docker.com/engine/security/seccomp/ | Docker seccomp | Default profile blocks ~44 of 300+, allowlist semantics |
| https://www.kernel.org/doc/html/v4.19/userspace-api/seccomp_filter.html | Linux seccomp BPF | BPF cannot deref pointers; arg filtering is constants-only |
| https://www.sysdig.com/blog/runc-container-escape-vulnerabilities | runc CVE bundle Nov 2025 | /dev/null symlink defeats maskedPaths; shared-kernel isolation broken |
| https://www.pillar.security/blog/prompt-injection-leads-to-rce-and-sandbox-escape-in-antigravity | Antigravity escape | Prompt injection + file write = RCE on host |
| https://winbuzzer.com/2026/05/14/building-a-safe-effective-sandbox-to-enable-codex-xcxwbn/ | OpenAI Codex Windows | Online/offline user split, DPAPI, firewall, AppContainer pattern |
| https://www.helpnetsecurity.com/2026/03/25/teampcp-supply-chain-attacks/ | LiteLLM PyPI compromise Mar 2026 | Maintainer creds via Trivy cascade; AI stack blast radius |
| https://northflank.com/blog/kata-containers-vs-firecracker-vs-gvisor | Kata/Firecracker/gVisor | Firecracker 1-2% overhead, gVisor 10-20%, Kata 5-10% |
| https://man7.org/linux/man-pages/man2/prctl.2.html | prctl manpage | NO_NEW_PRIVS prereq for unprivileged seccomp mode 2 |

## Sandbox tech comparison
| Tech | Boot ms | Mem overhead | Linux+Win? | Escape resistance | Per-baby-step latency tax | Citation |
|---|---|---|---|---|---|---|
| Firecracker cold | ~125-1000 | ~5 MiB/VM | Linux only (KVM) | High (HW VMM, no kernel share) | High first boot, near-zero steady | firecracker-microvm.github.io, dev.to/adwitiya |
| Firecracker snapshot-restore | ~28 | same | Linux only | Same as above | Near-zero | dev.to/adwitiya |
| gVisor (runsc, Systrap) | ~100-200 | tens of MiB | Linux only | Medium (still shared HW kernel, but syscall interpose) | 10-20% per syscall — heavy on file/proc | gvisor.dev/performance, usenix.org hotcloud19 |
| Kata Containers | 150-300 | gVisor + VMM | Linux only | High (VM) | 5-10% steady | northflank Kata vs Firecracker |
| Daytona | ~27 | per-sandbox kernel | Linux only (managed) | High (own kernel, isolated FS/net) | Network RTT dominates | daytona.io |
| E2B | 80-410 | managed | Linux only (managed) | High (Firecracker-class) | Network RTT dominates | e2b.dev/pricing, bertomill medium |
| Modal sandbox | sub-second claim | managed | Linux only (managed) | Medium (gVisor under hood) | Network RTT dominates | modal.com blog |
| bubblewrap | <10 | ~0 | Linux only | Low-Medium (user-ns, no kernel share break) | Near-zero | github.com/containers/bubblewrap |
| WebContainers | n/a (in browser) | tab memory | Browser only | Browser sandbox (only) | Renderer-process speed | blog.stackblitz.com |
| Pyodide-in-Deno | ~100-500 | WASM heap | Cross-OS | LOW if host bridge exposed (CVE-2025-68668) | Slow Python, but no IPC | langchain-sandbox repo, cyera CVE-2025-68668 |
| Apple Container | sub-second | per-VM | macOS only | High (VM-per-container, Swift vminitd) | Near-zero | github.com/apple/container, theregister.com |
| Windows AppContainer + Job + LowIL | tens of ms | tiny | Windows only | Medium-High (token + IL + Job) | Near-zero | microsoft SandboxSecurityTools, chromium sandbox |
| runc-only container | <50 | tiny | Linux only | LOW post-Nov-2025 CVEs | Near-zero | sysdig.com runc CVEs |

## Attack surface analysis
- CVE-2025-31133 / 52565 / 52881 (runc, Nov 2025): bind-mount race conditions defeat maskedPaths, allow writing /proc/sys/kernel/core_pattern from inside container = full host root. Patched in 1.2.8 / 1.3.3 / 1.4.0-rc.3. Implication for ELAI Tier C: any plan that relies solely on namespaces + capability drop + seccomp without VM boundary is now demonstrably broken against an attacker who controls the agent's startup args [sysdig.com runc-container-escape-vulnerabilities, cncf.io].
- CVE-2025-68668 ("n8scape", Pyodide in n8n): Python inside Pyodide imported `js` module and called Node `require('child_process')` from the host bridge. Single-host, multi-tenant, post-auth RCE. ELAI lesson: any in-process WASM Python option must aggressively prune the host bridge before exec, and the bridge surface is the trust boundary, not the WASM VM [cyera.com N8Scape, cvereports.com CVE-2025-68668].
- EchoLeak / CVE-2025-32711 (Microsoft 365 Copilot, CVSS 9.3): first known zero-click prompt-injection exfiltration in a production agent. Three chained flaws: template literal bypass, prepareStackTrace exploit, arrow-function oversight in JS sandbox. Implication: prompt injection + a sandbox compatibility hole = exfiltration even when each individually looks "safe" [softwareseni AI Agents in Production Sandboxing].
- Google Antigravity sandbox escape (Pillar Security, late 2025): prompt injection that asks the agent to create a file pivots to RCE via the file-creation pathway. The sandbox controlled syscalls but not the agent's semantic capability surface [pillar.security blog, cyberscoop].
- CVE-2025-53773 (GitHub Copilot): embedded malicious instructions in repo files = RCE via agent. Same pattern: trusted analysis path that mutates host state [softwareseni].
- CVE-2025-23266 NVIDIAScape (Wiz): GPU-accelerated container infra was infrastructure-wide vulnerable. Applies if ELAI ever GPU-accelerates an agent process directly [softwareseni].
- TeamPCP / LiteLLM PyPI 1.82.7+1.82.8 (Mar 2026): maintainer creds harvested via Trivy supply-chain cascade, malicious release bypassed GitHub-based release pipeline. Targeted SSH keys, cloud tokens, K8s secrets, crypto wallets, .env. Implication for ELAI: pip install inside the sandbox needs to NOT see host SSH, AWS, .env. Sandbox must be reset between baby steps or per-step pip layer must be unprivileged + read-only host mounts [helpnetsecurity TeamPCP, bastion.tech LiteLLM, reversinglabs].

## Best techniques to adopt
Ranked highest impact first.

1. Firecracker snapshot-restore for the hot pool (replaces "spawn fresh agent" path)
   - Grounding: dev.to/adwitiya measured 28ms restore vs ~1s cold; AgentSphere production for AI agents; AWS Lambda SnapStart uses same primitive [firecracker-microvm.github.io, dev.to/adwitiya, softwareseni firecracker-gvisor-containers-webassembly].
   - Phase 4A hook: keep clone3 + namespaces path for cold (first-of-kind) boot; add `firecracker --restore-from-snapshot` path for the steady-state baby-step pool. Snapshot per agent_id + role.
   - Caveat: snapshot uniqueness — restoring the same snapshot N times yields N VMs with identical entropy / TCP ISN / etc. Mitigate per arXiv 2102.12892 (Restoring Uniqueness in MicroVM Snapshots).

2. seccomp BPF argument-checked socket() instead of blanket allow (BLOCK-1 of the audit)
   - Grounding: Docker's default profile demonstrates argument filtering on `clone`, `personality`, `setns` is current production practice; kernel.org seccomp_filter.html confirms BPF can read seccomp_data.args[]; Chromium ships this pattern [docs.docker.com seccomp, kernel.org seccomp_filter.html].
   - Phase 4A hook: in IPC_BASELINE, deny socket(). In NETWORK_EXTRA (researcher), allow socket() ONLY where args[0] == AF_INET (2) or AF_INET6 (10) AND args[1] == SOCK_STREAM (1) or SOCK_DGRAM (2). Deny AF_NETLINK/AF_PACKET/AF_UNIX/AF_BLUETOOTH. This is exactly the BLOCK-1 audit ask.
   - Caveat: BPF cannot deref pointers, so anything beyond scalar arg checks (e.g. inspecting struct sockaddr*) must be done via Landlock or a userspace proxy.

3. veth + iptables OUTPUT chain allowlist + DNS-via-trusted-resolver for researcher network egress
   - Grounding: shayon.dev sandbox isolation analysis (2026) — drop CAP_NET_RAW so DNS bypass via raw sockets is closed; force traffic through a Squid/userspace proxy that does domain-level allowlist; deny inside-namespace DNS [shayon.dev 2026/52 lets-discuss-sandbox-isolation].
   - Phase 4A hook: keeps the planned "zero capabilities" researcher; adds iptables OUTPUT rule in the researcher's net-namespace that drops anything not destined for the curated arXiv/HF mirror list + resolver IPs. Reject not drop on the inside so the agent gets fast EHOSTUNREACH rather than 30s hang.

4. Maintain Firecracker (or gVisor) microVM boundary for ALL bash execution from Phase 0 — not just Phase 3+
   - Grounding: runc CVE-2025-31133 etc. demonstrate chroot/namespaces are not a security boundary against an attacker who controls mounts; gVisor 10-20% overhead is acceptable for bash steps that already cost ~30s [sysdig.com runc-container-escape-vulnerabilities, gvisor.dev/performance].
   - Phase 4A hook: revise the "chroot in Phase 0-1, Firecracker/gVisor in Phase 3+" plan to drop chroot entirely. Use Firecracker if KVM is available, gVisor (runsc) as fallback. chroot adds no real boundary that namespaces don't already provide and lulls reviewers.

5. NO_NEW_PRIVS + reduced capability bounding set, set BEFORE seccomp filter
   - Grounding: kernel.org/doc/Documentation/prctl/no_new_privs.txt — prereq for unprivileged seccomp mode 2; mpdesouza.com — sequence matters [kernel.org no_new_privs.txt, man7.org/prctl, mpdesouza no_new_privs].
   - Phase 4A hook: keep the plan; explicitly assert ordering in code: `prctl(PR_SET_NO_NEW_PRIVS, 1)` -> `PR_CAPBSET_DROP` loop over all caps -> `seccomp(SECCOMP_SET_MODE_FILTER, ...)`. Inverted ordering is a frequent bug.

6. Treat per-baby-step state as ephemeral; fork from snapshot instead of reusing a long-lived agent process
   - Grounding: TeamPCP / LiteLLM compromise (Mar 2026) shows that a malicious pip install during a baby step can plant a credential stealer; if the process survives to the next baby step, the stealer survives. Fork-from-snapshot evicts it [helpnetsecurity TeamPCP, upwind.io].
   - Phase 4A hook: align "1+4+postcard IPC" with per-baby-step fork-from-snapshot. The "1" stays long-lived; the "4" workers are snapshot-restored per baby step.

7. Windows: ship the AppContainer + restricted token + Job-object trio exactly; do not innovate
   - Grounding: microsoft/SandboxSecurityTools LaunchAppContainer reference; Chromium sandbox doc; OpenAI Codex (May 2026) reportedly ships the same trio [github.com/microsoft/SandboxSecurityTools, chromium sandbox.md, winbuzzer Codex].
   - Phase 4A hook: keep the Windows fallback plan as-is, but add "AppContainer profile zero capabilities" and "Job object kill-on-close + active-process-limit" as MUST. Add a low-integrity descriptor on the working directory.

## Anti-patterns
- "chroot is part of the boundary." It isn't. Namespaces already give a chroot-equivalent; chroot itself is escapable with CAP_SYS_CHROOT or by having a fd into the parent root [github.com/containers/bubblewrap, sysdig.com runc 2025].
- "We dropped capabilities and set seccomp, we're good without a VM." runc Nov 2025 CVEs disprove this conclusively against an attacker who controls mount config [sysdig.com runc-container-escape-vulnerabilities].
- "Pyodide is a sandbox." Pyodide is a WASM Python; the sandbox is whatever the host bridge exposes. CVE-2025-68668 escapes by importing `js` and reaching Node `require('child_process')` [cyera.com N8Scape, cvereports.com].
- "WebContainers in browser is too sandboxed to bother with isolation review." Reaches the network and DOM; same prompt-injection-to-exfil class as EchoLeak applies [softwareseni AI Agents in Production].
- "Modal/E2B managed sandbox is good enough, don't roll your own." Fine for managed RPC, but ELAI Tier C must work offline / no-egress / per-VRAM-tier. Managed-only loses the ability to run cold-start-sensitive baby steps below ~400ms p50 [e2b.dev/pricing, bertomill medium].
- "Allow socket() unconditionally and rely on iptables." iptables in a hostile network namespace can be tampered with by any process holding CAP_NET_ADMIN; researcher should hold zero caps. seccomp must reject AF_NETLINK and AF_PACKET outright [kernel.org seccomp_filter.html].
- "Cgroups are a security boundary." They are not. They are DoS prevention. Confirmed across multiple 2025-2026 references [kayssel.com docker-security-1, cubepath.com].
- "Reuse the same agent process across baby steps to save the 28ms restore." Loses the ephemerality property that defends against in-step supply-chain compromise [TeamPCP refs].

## Open questions
1. p50 baby-step latency budget with Firecracker snapshot-restore + per-step network namespace setup on the target hardware (3090 box + local Windows): is the end-to-end clone+restore+seccomp+ns-net wiring under 100ms? Need to measure, not assume the dev.to/adwitiya 28ms number.
2. Does Firecracker run usefully on the user's actual Windows host, or is the Windows path strictly AppContainer+Job? (Hyper-V isolation may give a microVM option but adds boot cost.) Measure boot-time on Win11 Hyper-V vs native AppContainer.
3. Snapshot-uniqueness gap (arXiv 2102.12892): are TCP ISNs / /dev/random pools / hostname colliding across restored microVMs a problem for ELAI's IPC contract? Need a test that restores N=10 VMs and asserts orthogonality.
4. Cost of full network-namespace teardown per baby step vs reusing a long-lived netns. Measure netns-create + veth-pair + iptables-restore latency on Linux 6.x.
5. Compatibility tax of gVisor for ELAI's bash workloads specifically (cargo, npm install, python -m pip): measure end-to-end vs Firecracker on the same baby step.

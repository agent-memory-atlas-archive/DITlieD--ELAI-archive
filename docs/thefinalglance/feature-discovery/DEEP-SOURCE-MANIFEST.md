# ELAI deep-source assignment manifest

This is a content-blind, mechanical census and deep-read assignment map. It does not interpret or summarize plan contents.

## Scope and counting rules

- In-scope census: every path returned from the sixteen letter roots B-plan through Q-plan, recursively, plus every path in docs/specs/guide/steps.
- The overview is recorded as the eighteenth globally assigned summary source, but it is external to this census and is not included in any parity equation.
- A source line is one logical line: newline count plus one for a non-empty final unterminated line.
- Byte counts are exact file lengths. SHA-256 values hash the exact source bytes.
- Deep-readable means a .md or .txt file whose bytes decode as UTF-8.
- Inventory ordering and path-set digests use bytewise ascending absolute paths.

Inventory command:

~~~text
rg --files --hidden --no-ignore \
  docs/thefinalglance/B-plan \
  docs/thefinalglance/C-plan \
  docs/thefinalglance/D-plan \
  docs/thefinalglance/E-plan \
  docs/thefinalglance/F-plan \
  docs/thefinalglance/G-plan \
  docs/thefinalglance/H-plan \
  docs/thefinalglance/I-plan \
  docs/thefinalglance/J-plan \
  docs/thefinalglance/K-plan \
  docs/thefinalglance/L-plan \
  docs/thefinalglance/M-plan \
  docs/thefinalglance/N-plan \
  docs/thefinalglance/O-plan \
  docs/thefinalglance/P-plan \
  docs/thefinalglance/Q-plan \
  docs/specs/guide/steps
~~~

## Deterministic chunk construction

Chunks never cross a letter family or the phase family. Already covered summaries and excluded artifacts do not enter deep chunks.

For each family, deep files are ordered by descending logical line count with bytewise path as the tie-breaker. Files over 2,800 lines stand alone. For the remainder, bin count begins at max(ceil(files / 10), ceil(lines / 2800)); deterministic depth-first packing tries bins by ascending current lines, ascending current file count, then bin ordinal, increasing bin count only if no exact packing exists. Finished bins are labeled by the natural-sort key of their smallest path. Within each chunk, the exact paths are naturally sorted.

Dispatch rule: assign one fresh deep-reader sub-agent to each chunk ID below. Its complete source scope is exactly the path table under that chunk.

## Mechanical audit

| Check | Result |
|---|---:|
| In-scope census paths | 653 |
| In-scope summary-covered paths | 17 |
| Deep-assigned readable .md/.txt paths | 635 |
| Explicitly excluded non-text/non-plan paths | 1 |
| Census parity | 653 = 17 + 635 + 1 PASS |
| Deep chunk count | 76 |
| Sum of chunk file counts | 635 |
| Unique deep-assigned paths | 635 |
| Duplicate deep appearances | 0 PASS |
| Missing deep paths | 0 PASS |
| Extra deep paths | 0 PASS |
| Chunks in 1,800–2,800 line target band | 32 |
| Chunks below 1,800 lines | 44 |
| Chunks over 2,800 lines | 0 PASS |
| Chunks over 10 files | 0 PASS |
| Inventory path-list SHA-256 | eaa11f9a1c2bbc5b7d1a5281ef82b0bfc82899fa4d8a0dce0ea7fad7b2a3b8e0 |
| Summary-covered in-scope path-list SHA-256 | b8c93ae6421d344a15e981f513fb7132e0eae1cf91d646c94437e5da321880ef |
| Deep census path-list SHA-256 | 1fcdf80e4b7da868a316273d531a2584234ba837db0d6deb8d2058aad37130dd |
| Deep assignment path-list SHA-256 | 1fcdf80e4b7da868a316273d531a2584234ba837db0d6deb8d2058aad37130dd PASS (equals deep census) |
| Excluded path-list SHA-256 | a170527962f8fae2a61fcb3f4c570aabe5f3da75a29dc7cb39bad8294e61827f |

### Per-family parity

| Family | Census | Summaries | Numbered/index originals | Auxiliary/grounding/fixture | Excluded | Deep total | Chunks | Deep lines | Under-band chunks |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| B | 47 | 1 | 46 | 0 | 0 | 46 | 6 | 15349 | 0 |
| C | 61 | 1 | 60 | 0 | 0 | 60 | 6 | 15415 | 0 |
| D | 32 | 1 | 31 | 0 | 0 | 31 | 4 | 7333 | 1 |
| E | 57 | 1 | 56 | 0 | 0 | 56 | 6 | 13141 | 0 |
| F | 17 | 1 | 15 | 1 | 0 | 16 | 2 | 4285 | 0 |
| G | 12 | 1 | 11 | 0 | 0 | 11 | 2 | 3262 | 2 |
| H | 62 | 1 | 55 | 6 | 0 | 61 | 7 | 10329 | 7 |
| I | 58 | 1 | 55 | 2 | 0 | 57 | 6 | 6981 | 6 |
| J | 123 | 1 | 113 | 9 | 0 | 122 | 13 | 16766 | 12 |
| K | 32 | 1 | 28 | 3 | 0 | 31 | 4 | 5273 | 4 |
| L | 28 | 1 | 25 | 2 | 0 | 27 | 3 | 2990 | 3 |
| M | 8 | 1 | 7 | 0 | 0 | 7 | 1 | 542 | 1 |
| N | 18 | 1 | 17 | 0 | 0 | 17 | 2 | 1429 | 2 |
| O | 6 | 1 | 5 | 0 | 0 | 5 | 1 | 491 | 1 |
| P | 27 | 1 | 26 | 0 | 0 | 26 | 3 | 3023 | 3 |
| Q | 19 | 1 | 17 | 1 | 0 | 18 | 2 | 2229 | 2 |
| PHASE | 46 | 1 | 44 | 0 | 1 | 44 | 8 | 21912 | 0 |
| **TOTAL** | **653** | **17** | **611** | **24** | **1** | **635** | **76** | **130750** | **44** |

### Mechanical anomalies and reconciliations

- The stated 18 summary-layer sources reconcile as 17 in-scope sources plus the out-of-scope overview recorded below: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md.
- One non-plan artifact is present and explicitly excluded: docs/specs/guide/steps/desktop.ini.
- 44 chunks are below the soft 1,800-line target. All still satisfy the preferred 10-file ceiling; the deviations follow mechanically from per-family isolation, discrete file sizes, and the ceiling.
- No chunk exceeds 2,800 lines, no chunk exceeds 10 files, and there are no oversized standalone files.
- 1 source lacks a terminal LF; logical-line counting includes its final unterminated line: docs/thefinalglance/K-plan/K-27_gpt56_sol_isolated_executor_lane.md.

Under-band chunks:

| Chunk ID | Family | Files | Lines |
|---|---|---:|---:|
| DEEP-D-004 | D | 7 | 1755 |
| DEEP-G-001 | G | 6 | 1723 |
| DEEP-G-002 | G | 5 | 1539 |
| DEEP-H-001 | H | 9 | 1499 |
| DEEP-H-002 | H | 9 | 1484 |
| DEEP-H-003 | H | 5 | 1454 |
| DEEP-H-004 | H | 9 | 1452 |
| DEEP-H-005 | H | 9 | 1451 |
| DEEP-H-006 | H | 10 | 1495 |
| DEEP-H-007 | H | 10 | 1494 |
| DEEP-I-001 | I | 10 | 1169 |
| DEEP-I-002 | I | 7 | 1169 |
| DEEP-I-003 | I | 10 | 1157 |
| DEEP-I-004 | I | 10 | 1155 |
| DEEP-I-005 | I | 10 | 1169 |
| DEEP-I-006 | I | 10 | 1162 |
| DEEP-J-001 | J | 10 | 1280 |
| DEEP-J-002 | J | 10 | 1224 |
| DEEP-J-003 | J | 10 | 1229 |
| DEEP-J-004 | J | 10 | 1222 |
| DEEP-J-005 | J | 10 | 1228 |
| DEEP-J-006 | J | 10 | 1223 |
| DEEP-J-008 | J | 10 | 1219 |
| DEEP-J-009 | J | 10 | 1247 |
| DEEP-J-010 | J | 10 | 1220 |
| DEEP-J-011 | J | 10 | 1225 |
| DEEP-J-012 | J | 10 | 1226 |
| DEEP-J-013 | J | 10 | 1220 |
| DEEP-K-001 | K | 7 | 1294 |
| DEEP-K-002 | K | 7 | 1303 |
| DEEP-K-003 | K | 9 | 1307 |
| DEEP-K-004 | K | 8 | 1369 |
| DEEP-L-001 | L | 9 | 985 |
| DEEP-L-002 | L | 9 | 1024 |
| DEEP-L-003 | L | 9 | 981 |
| DEEP-M-001 | M | 7 | 542 |
| DEEP-N-001 | N | 7 | 710 |
| DEEP-N-002 | N | 10 | 719 |
| DEEP-O-001 | O | 5 | 491 |
| DEEP-P-001 | P | 9 | 1070 |
| DEEP-P-002 | P | 10 | 918 |
| DEEP-P-003 | P | 7 | 1035 |
| DEEP-Q-001 | Q | 9 | 1116 |
| DEEP-Q-002 | Q | 9 | 1113 |

## Summary-covered sources

The first row is external context. The remaining 17 rows are part of the in-scope census and are excluded from deep chunks.

| Scope | Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---|---:|---:|---|---|
| external (not in census) | SUMMARY-OVERVIEW | summary-covered | 592 | 50671 | ad1cb330bcec68caa7658c188f057032ca6a6e830d307a41767fed8aa08160f2 | docs/thefinalglance/ELAI-SPEC-OVERVIEW.md |
| in-scope | SUMMARY-PHASE | summary-covered | 1056 | 81373 | b70e5b0becb0bc78208153fac357ef16cfe50489182709a6e1c7d359007fd7dc | docs/specs/guide/steps/phase-index-MASTER.txt |
| in-scope | SUMMARY-B | summary-covered | 1111 | 64058 | 58eea417fc3a032d1a3eba53db9c26b43302a11b6147f71a2d0e0428bf952685 | docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md |
| in-scope | SUMMARY-C | summary-covered | 814 | 67806 | d0384b81bbf6fa849eb1eed0570af817e9bbbafca5b9601c53500ed93b64e3f5 | docs/thefinalglance/C-plan/C-SUMMARY_INDEX.md |
| in-scope | SUMMARY-D | summary-covered | 311 | 27950 | c5dcc459749973102e5ec30874f006abd067e2054cd823bc68d06b85198bffe3 | docs/thefinalglance/D-plan/D-SUMMARY_INDEX.md |
| in-scope | SUMMARY-E | summary-covered | 615 | 45975 | 53ea7b7f8d9c5c2135a1fcdaf47e14743b94005de2f3975ef6030de367f0dac8 | docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md |
| in-scope | SUMMARY-F | summary-covered | 286 | 24492 | bda97d2087adc5a8646e8ed587c3fd765b4a68857082e18b27467709e3e280c6 | docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md |
| in-scope | SUMMARY-G | summary-covered | 225 | 20993 | 6be054812447b7559118ea500440c9d9dbbe4af3f77bcf9631f92028d07b0391 | docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md |
| in-scope | SUMMARY-H | summary-covered | 1038 | 128990 | 01302eea045c4de0ac3651d4dd220b47c4f49630dfcf27b8435864e4c489670d | docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md |
| in-scope | SUMMARY-I | summary-covered | 1048 | 88976 | ee453bc09f053a6c5e783e77c452ea0596f1dcb393832aa376416a47654e0a7d | docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md |
| in-scope | SUMMARY-J | summary-covered | 2386 | 270928 | fdc0c5ad963781b5202b59fa1cc1f10b4e8b2d218da642a5a05e90a6aacc84f8 | docs/thefinalglance/J-plan/J-SUMMARY_INDEX.md |
| in-scope | SUMMARY-K | summary-covered | 600 | 69175 | e35935f2c0036001628ee430ab09769c32d3f8ade2cf49459211138944cece19 | docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md |
| in-scope | SUMMARY-L | summary-covered | 264 | 52094 | d53d116f4fff8bb75bed42664515a69baa2c3987dd1622bed8d72f3816b9e8d4 | docs/thefinalglance/L-plan/L-SUMMARY_INDEX.md |
| in-scope | SUMMARY-M | summary-covered | 146 | 12225 | ef0e882c027405b5296c49667a784aac28ad640153403cc2154971988ebbe25e | docs/thefinalglance/M-plan/M-SUMMARY_INDEX.md |
| in-scope | SUMMARY-N | summary-covered | 363 | 32450 | 27a323a651659e7fc7c1b5e061bd37698882a819ff8415e6847a3c303b84cd03 | docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md |
| in-scope | SUMMARY-O | summary-covered | 84 | 7042 | 4808c7d30650fc6219b1b57d5b6fab9b1d445e185a73c054ae5a207f76df6286 | docs/thefinalglance/O-plan/O-SUMMARY_INDEX.md |
| in-scope | SUMMARY-P | summary-covered | 530 | 51778 | 8a7fb26548bba5a4ceb09bf757871dbf2aa0724843af3904e92f123747bce3b5 | docs/thefinalglance/P-plan/P-SUMMARY_INDEX.md |
| in-scope | SUMMARY-Q | summary-covered | 148 | 41802 | 29b0a63c5e34e116952ada92de09087c632a7a8417fb90930d0541daa2b4d969 | docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md |

## Explicitly excluded artifacts

| Chunk ID | Class | Lines | Bytes | SHA-256 | Reason | Exact absolute path |
|---|---|---:|---:|---|---|---|
| EXCLUDED | non-text/non-plan artifact | 2 | 106 | 217b278d27356e5a25823ccd0240ed949c9a47109e35c53a9a99a58c8f5506b2 | non-plan extension .ini; file(1): Windows desktop.ini | docs/specs/guide/steps/desktop.ini |

## Chunk index

| Chunk ID | Family | Files | Lines | Bytes | Numbered/index | Auxiliary | Target band |
|---|---|---:|---:|---:|---:|---:|---|
| DEEP-B-001 | B | 6 | 2427 | 133280 | 6 | 0 | yes |
| DEEP-B-002 | B | 8 | 2587 | 123236 | 8 | 0 | yes |
| DEEP-B-003 | B | 8 | 2576 | 126263 | 8 | 0 | yes |
| DEEP-B-004 | B | 8 | 2595 | 116448 | 8 | 0 | yes |
| DEEP-B-005 | B | 8 | 2578 | 126541 | 8 | 0 | yes |
| DEEP-B-006 | B | 8 | 2586 | 118996 | 8 | 0 | yes |
| DEEP-C-001 | C | 10 | 2554 | 147739 | 10 | 0 | yes |
| DEEP-C-002 | C | 10 | 2552 | 135174 | 10 | 0 | yes |
| DEEP-C-003 | C | 10 | 2552 | 140376 | 10 | 0 | yes |
| DEEP-C-004 | C | 10 | 2651 | 140924 | 10 | 0 | yes |
| DEEP-C-005 | C | 10 | 2553 | 139392 | 10 | 0 | yes |
| DEEP-C-006 | C | 10 | 2553 | 136705 | 10 | 0 | yes |
| DEEP-D-001 | D | 8 | 1861 | 89099 | 8 | 0 | yes |
| DEEP-D-002 | D | 8 | 1855 | 87723 | 8 | 0 | yes |
| DEEP-D-003 | D | 8 | 1862 | 88049 | 8 | 0 | yes |
| DEEP-D-004 | D | 7 | 1755 | 83367 | 7 | 0 | under |
| DEEP-E-001 | E | 9 | 2134 | 239903 | 9 | 0 | yes |
| DEEP-E-002 | E | 9 | 2131 | 251948 | 9 | 0 | yes |
| DEEP-E-003 | E | 9 | 2141 | 249792 | 9 | 0 | yes |
| DEEP-E-004 | E | 10 | 2302 | 274448 | 10 | 0 | yes |
| DEEP-E-005 | E | 9 | 2137 | 254945 | 9 | 0 | yes |
| DEEP-E-006 | E | 10 | 2296 | 284438 | 10 | 0 | yes |
| DEEP-F-001 | F | 8 | 2188 | 322018 | 8 | 0 | yes |
| DEEP-F-002 | F | 8 | 2097 | 292736 | 7 | 1 | yes |
| DEEP-G-001 | G | 6 | 1723 | 151046 | 6 | 0 | under |
| DEEP-G-002 | G | 5 | 1539 | 137029 | 5 | 0 | under |
| DEEP-H-001 | H | 9 | 1499 | 197472 | 5 | 4 | under |
| DEEP-H-002 | H | 9 | 1484 | 153229 | 7 | 2 | under |
| DEEP-H-003 | H | 5 | 1454 | 166004 | 5 | 0 | under |
| DEEP-H-004 | H | 9 | 1452 | 153591 | 9 | 0 | under |
| DEEP-H-005 | H | 9 | 1451 | 149987 | 9 | 0 | under |
| DEEP-H-006 | H | 10 | 1495 | 159276 | 10 | 0 | under |
| DEEP-H-007 | H | 10 | 1494 | 158951 | 10 | 0 | under |
| DEEP-I-001 | I | 10 | 1169 | 99202 | 9 | 1 | under |
| DEEP-I-002 | I | 7 | 1169 | 99250 | 6 | 1 | under |
| DEEP-I-003 | I | 10 | 1157 | 89623 | 10 | 0 | under |
| DEEP-I-004 | I | 10 | 1155 | 92980 | 10 | 0 | under |
| DEEP-I-005 | I | 10 | 1169 | 95848 | 10 | 0 | under |
| DEEP-I-006 | I | 10 | 1162 | 95462 | 10 | 0 | under |
| DEEP-J-001 | J | 10 | 1280 | 158205 | 9 | 1 | under |
| DEEP-J-002 | J | 10 | 1224 | 152552 | 9 | 1 | under |
| DEEP-J-003 | J | 10 | 1229 | 134050 | 9 | 1 | under |
| DEEP-J-004 | J | 10 | 1222 | 149575 | 8 | 2 | under |
| DEEP-J-005 | J | 10 | 1228 | 149610 | 8 | 2 | under |
| DEEP-J-006 | J | 10 | 1223 | 149753 | 9 | 1 | under |
| DEEP-J-007 | J | 2 | 2003 | 222633 | 1 | 1 | yes |
| DEEP-J-008 | J | 10 | 1219 | 151045 | 10 | 0 | under |
| DEEP-J-009 | J | 10 | 1247 | 127107 | 10 | 0 | under |
| DEEP-J-010 | J | 10 | 1220 | 146633 | 10 | 0 | under |
| DEEP-J-011 | J | 10 | 1225 | 147166 | 10 | 0 | under |
| DEEP-J-012 | J | 10 | 1226 | 151391 | 10 | 0 | under |
| DEEP-J-013 | J | 10 | 1220 | 133280 | 10 | 0 | under |
| DEEP-K-001 | K | 7 | 1294 | 142506 | 6 | 1 | under |
| DEEP-K-002 | K | 7 | 1303 | 143184 | 5 | 2 | under |
| DEEP-K-003 | K | 9 | 1307 | 190784 | 9 | 0 | under |
| DEEP-K-004 | K | 8 | 1369 | 142681 | 8 | 0 | under |
| DEEP-L-001 | L | 9 | 985 | 126089 | 7 | 2 | under |
| DEEP-L-002 | L | 9 | 1024 | 133375 | 9 | 0 | under |
| DEEP-L-003 | L | 9 | 981 | 101509 | 9 | 0 | under |
| DEEP-M-001 | M | 7 | 542 | 38550 | 7 | 0 | under |
| DEEP-N-001 | N | 7 | 710 | 64575 | 7 | 0 | under |
| DEEP-N-002 | N | 10 | 719 | 82251 | 10 | 0 | under |
| DEEP-O-001 | O | 5 | 491 | 48702 | 5 | 0 | under |
| DEEP-P-001 | P | 9 | 1070 | 122799 | 9 | 0 | under |
| DEEP-P-002 | P | 10 | 918 | 122607 | 10 | 0 | under |
| DEEP-P-003 | P | 7 | 1035 | 153893 | 7 | 0 | under |
| DEEP-Q-001 | Q | 9 | 1116 | 162025 | 8 | 1 | under |
| DEEP-Q-002 | Q | 9 | 1113 | 162098 | 9 | 0 | under |
| DEEP-PHASE-001 | PHASE | 6 | 2692 | 165923 | 6 | 0 | yes |
| DEEP-PHASE-002 | PHASE | 6 | 2750 | 178707 | 6 | 0 | yes |
| DEEP-PHASE-003 | PHASE | 6 | 2751 | 182824 | 6 | 0 | yes |
| DEEP-PHASE-004 | PHASE | 6 | 2773 | 176724 | 6 | 0 | yes |
| DEEP-PHASE-005 | PHASE | 4 | 2735 | 169614 | 4 | 0 | yes |
| DEEP-PHASE-006 | PHASE | 5 | 2791 | 174259 | 5 | 0 | yes |
| DEEP-PHASE-007 | PHASE | 5 | 2723 | 166521 | 5 | 0 | yes |
| DEEP-PHASE-008 | PHASE | 6 | 2697 | 162325 | 6 | 0 | yes |

## Exact deep-read chunk assignments

### DEEP-B-001

Family **B**; totals: **6 files**, **2427 logical lines**, **133280 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-B-001 | numbered/index original | 1176 | 75207 | b36a6d8c6fdcc4a5c4287906170197da416c85a88dd6f4b2dd1207567c774b5a | docs/thefinalglance/B-plan/B-0_INDEX.md |
| DEEP-B-001 | numbered/index original | 181 | 8212 | 3e76deff6f7ee27f56b273d4d457bcee3e884717261a6a3cc14c7f870af8e4cb | docs/thefinalglance/B-plan/B-10_contextbudget_profile_activation.md |
| DEEP-B-001 | numbered/index original | 203 | 8316 | 921e91af0e1fb9027aa76506f9568f924d5fe72b6e880fe722b438f24051d1db | docs/thefinalglance/B-plan/B-11_design_system_restoration.md |
| DEEP-B-001 | numbered/index original | 268 | 11880 | 513001e8dbd83b6bdb8bcadbc5f029ccf3de63d12b9187557be2f1cfda6ea7b1 | docs/thefinalglance/B-plan/B-20_tool_registry_unification.md |
| DEEP-B-001 | numbered/index original | 237 | 10417 | 5f657918370fe990a4bed466104f7b9b3ab9696f36adde96e9b99c68288ff2a6 | docs/thefinalglance/B-plan/B-38_ambiguity_score_gate.md |
| DEEP-B-001 | numbered/index original | 362 | 19248 | 1d7849556b111b377cf0b2c8385bb596d66ae4669210365b18610b2862636673 | docs/thefinalglance/B-plan/B-41_per_level_auditor_modes.md |

### DEEP-B-002

Family **B**; totals: **8 files**, **2587 logical lines**, **123236 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-B-002 | numbered/index original | 218 | 9817 | 44e91be8da1ea255ff353df2cada61ac79c23723e83315906fb0822e1827286c | docs/thefinalglance/B-plan/B-1_path_unification.md |
| DEEP-B-002 | numbered/index original | 245 | 11543 | 8c7cfdac376353b926a43f521ca9f3568ebefc285e8318e0ff37f0f767728c5b | docs/thefinalglance/B-plan/B-2_fsm_phasetransition_wiring.md |
| DEEP-B-002 | numbered/index original | 338 | 16931 | a7c5f0acde93adce4e4fba95e958a83ef0ae4e99fb4636781b815afe323b38b2 | docs/thefinalglance/B-plan/B-4_knowledge_producer_wiring.md |
| DEEP-B-002 | numbered/index original | 194 | 8689 | bdc5721c92765943a58047c47943aa5d193c8ff0637a67a977a573591ad74ec1 | docs/thefinalglance/B-plan/B-12_shell_vs_workspace_decision.md |
| DEEP-B-002 | numbered/index original | 165 | 6493 | 061c55e89a7018b322450a0b65646e1c02497f45baa8d70dbbe3cc09a1a2a61c | docs/thefinalglance/B-plan/B-24_phase_4I_media_validator.md |
| DEEP-B-002 | numbered/index original | 465 | 22365 | 16e2210616db9b83d8b6b95cbfbf98b5b6f89f03b05923eb099e9671d403a174 | docs/thefinalglance/B-plan/B-31_resumable_indexing_lock_lifecycle.md |
| DEEP-B-002 | numbered/index original | 402 | 19213 | f42601ae4f2501b2b2f309ba2141002d821b1b3fb2bc871072867ee369651289 | docs/thefinalglance/B-plan/B-34_entry_point_detection.md |
| DEEP-B-002 | numbered/index original | 560 | 28185 | 712dcaa6bb4bd4242b87f9ebbc526d173875cda07a68a7d3f6beb4face1dc612 | docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md |

### DEEP-B-003

Family **B**; totals: **8 files**, **2576 logical lines**, **126263 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-B-003 | numbered/index original | 514 | 27022 | 05ba155a7fd48bd6c0fcff334bc4ab8336f0669da41bf4ea0b5b4422b5ef9211 | docs/thefinalglance/B-plan/B-3_stub_command_eradication.md |
| DEEP-B-003 | numbered/index original | 153 | 6690 | 0aeec6b5d12127cfd46c95e580f19b18332bbf2195e1c93afa019a5898ce80ec | docs/thefinalglance/B-plan/B-8_pipelinegrc_activation.md |
| DEEP-B-003 | numbered/index original | 204 | 9948 | 6273a43aaafcee2a38af2c5a2aa97053fe2b928be042af69251fbf69386fac32 | docs/thefinalglance/B-plan/B-9_sessiondag_activation.md |
| DEEP-B-003 | numbered/index original | 201 | 8836 | f1c085dcbd1cef6d9258053dc011b3a89e3bdc36169fe903be78ed366b1202cf | docs/thefinalglance/B-plan/B-13_activity_feed_restoration.md |
| DEEP-B-003 | numbered/index original | 469 | 22540 | 56181ccf70c24ed8bad6002622d9ffac446d533e2df6ab018fddfe2fc9586b89 | docs/thefinalglance/B-plan/B-15_phase0_debt_burndown.md |
| DEEP-B-003 | numbered/index original | 443 | 21321 | a37a115862feee953a107401924a4a1d096bee21ef455d75cb5a903d7896cd34 | docs/thefinalglance/B-plan/B-30_multilang_dkg_hardening.md |
| DEEP-B-003 | numbered/index original | 254 | 11781 | 7f2c225817e7ec5ed73e3df260cdd6ec4c71342e958d51b6240e44e64d9b28f7 | docs/thefinalglance/B-plan/B-37_multi_runtime_backend.md |
| DEEP-B-003 | numbered/index original | 338 | 18125 | 9a48126f92bc959da36a8d1fe617a908296a01641a2c2f020baf055f99bec3df | docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md |

### DEEP-B-004

Family **B**; totals: **8 files**, **2595 logical lines**, **116448 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-B-004 | numbered/index original | 176 | 8723 | 2dc125ed704490e7dcc384ffef2b1f2d62fcf658f1bcd070306d84db1e3fac1b | docs/thefinalglance/B-plan/B-5_reasoning_producer_wiring.md |
| DEEP-B-004 | numbered/index original | 191 | 8379 | 2b318fb705458d8f9807cb589f24c0c786f434949c77fd2ea1b9d7f4b96ed34a | docs/thefinalglance/B-plan/B-14_live_token_streaming_chat.md |
| DEEP-B-004 | numbered/index original | 214 | 10501 | 91b5af2c9cc0e340940f4d631dd0dbf389e0114aef9fc2976ea971510e86d78a | docs/thefinalglance/B-plan/B-25_phase_4J_empire_integration.md |
| DEEP-B-004 | numbered/index original | 250 | 9698 | 79853608c3a8598b8c5ec0a609a05b2204664db2da04606610af623ed29f498a | docs/thefinalglance/B-plan/B-27_cloud_mode.md |
| DEEP-B-004 | numbered/index original | 499 | 19866 | 9ba29b7b5fc57d3d70a33fa170a98b101cc683581d0ae9cf845d66ee8455ad29 | docs/thefinalglance/B-plan/B-31.5_resumable_indexing_scale_benchmark.md |
| DEEP-B-004 | numbered/index original | 501 | 22608 | e0216562c870cf2c3e98e303f5996b9fa44c7f81eb2b51c40bf8c64cca194b3d | docs/thefinalglance/B-plan/B-33_interactive_graph_viewer.md |
| DEEP-B-004 | numbered/index original | 356 | 16455 | b6671f0e0e40bbd6250f4296c0703469bed8762862ecf61815558ba8c978a5b3 | docs/thefinalglance/B-plan/B-36_dashboard_v2_src_tauri_wiring.md |
| DEEP-B-004 | numbered/index original | 408 | 20218 | 45c46401f9b448a3be4e1470f6b60aa6c087d604a0da08ae9a03e767398a2cf9 | docs/thefinalglance/B-plan/B-42_cosmos_artifact_tier.md |

### DEEP-B-005

Family **B**; totals: **8 files**, **2578 logical lines**, **126541 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-B-005 | numbered/index original | 197 | 8949 | e23091d6040c90d2be6aea5a07b33ab73e258570b39d1d4742f70cd06932cdb0 | docs/thefinalglance/B-plan/B-6_orchestration_producer_wiring.md |
| DEEP-B-005 | numbered/index original | 155 | 7682 | 51950a9b3c63a8dca4532c3e2b0d0836dc32d929131762fb61842e451386e0e0 | docs/thefinalglance/B-plan/B-17_ground_vacuous_gates.md |
| DEEP-B-005 | numbered/index original | 277 | 12204 | c838bcbc1f868ad9b0dd3692b830bc421817e5562733d2a12987de758bb181db | docs/thefinalglance/B-plan/B-22_phase_4G_autonomous_mode.md |
| DEEP-B-005 | numbered/index original | 241 | 11181 | 57c232b1a11c42ee88c4c05c09b2161f8d614993e4080a348e860db7121b1adc | docs/thefinalglance/B-plan/B-26_phase_4K_6_to_13_dashboard_completion.md |
| DEEP-B-005 | numbered/index original | 211 | 8007 | 7c63d9abb7018a7100c9d742f898c5e9702ec9732e605ab7e089f71d20e42c26 | docs/thefinalglance/B-plan/B-28_focus_tunnel.md |
| DEEP-B-005 | numbered/index original | 448 | 23173 | 0b2fe1a0e0a0e9d72c29b2322db266b0e6bd377f5b35456b7dd96a316dcc607b | docs/thefinalglance/B-plan/B-29_symbol_level_impact_api.md |
| DEEP-B-005 | numbered/index original | 379 | 16783 | 94d9048b0f8cc6f918a94d44eebe7fb15ef81d75670db2508bc8beceb3280e3c | docs/thefinalglance/B-plan/B-32_minified_file_chunking.md |
| DEEP-B-005 | numbered/index original | 670 | 38562 | ca5a8fcbc90071443721c3c25957435b61fa79dee82fdf86783dd02e99fc95e9 | docs/thefinalglance/B-plan/B-43_one_line_install.md |

### DEEP-B-006

Family **B**; totals: **8 files**, **2586 logical lines**, **118996 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-B-006 | numbered/index original | 399 | 20912 | e31ede6a8206ac925d2f3789a856d88f8934555326d772c593a3327f2f5c8cf6 | docs/thefinalglance/B-plan/B-7_pipeline_factory_producer_wiring.md |
| DEEP-B-006 | numbered/index original | 280 | 15101 | f66edf4ddfc48738ed96c0f92ef962e078365461953c577154161bd6a2190330 | docs/thefinalglance/B-plan/B-16_anti_stale_enforcement.md |
| DEEP-B-006 | numbered/index original | 203 | 8241 | bd27c1eacb49038ddcb18020892a1bcb6d98a0a74d0b041160cbaa54788e27fc | docs/thefinalglance/B-plan/B-18_tier_c_sandbox_activation.md |
| DEEP-B-006 | numbered/index original | 254 | 10282 | f22a1c5e0d8bc689a6cbc53e199e9c768e75318ae44e074112e1a6461e4e03e8 | docs/thefinalglance/B-plan/B-19_hook_system_unification.md |
| DEEP-B-006 | numbered/index original | 166 | 7290 | a4299a9271636b523daaf981f97b4a7b85a17ee8a34eca209510693e3adc3344 | docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md |
| DEEP-B-006 | numbered/index original | 227 | 10070 | 387d2c7969892307a913f3ebdc90baf34d75bd2c5874bf394fb10f105d110c8a | docs/thefinalglance/B-plan/B-23_phase_4H_self_modification.md |
| DEEP-B-006 | numbered/index original | 461 | 19054 | 4e0f9850ed73e607d64693d0a574b4b0f5538882a100170ffb407e2db0bbbad9 | docs/thefinalglance/B-plan/B-29.5_symbol_graph_scale_benchmark.md |
| DEEP-B-006 | numbered/index original | 596 | 28046 | 71ebbb6cfba20afbe817dc9187fe083025de5a2d456a545e40073b70630e9fe7 | docs/thefinalglance/B-plan/B-35_context_artifacts_pipeline.md |

### DEEP-C-001

Family **C**; totals: **10 files**, **2554 logical lines**, **147739 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-C-001 | numbered/index original | 426 | 37226 | 44972d7f8ff76bba317e838b160783113e53994bd83a797a0a8a4263902d3c66 | docs/thefinalglance/C-plan/C-0_INDEX.md |
| DEEP-C-001 | numbered/index original | 294 | 15979 | 2993e4663af2dd1235efd2e8854a864ddc095de9e3c09c3d4c21bbf38b14b7a7 | docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md |
| DEEP-C-001 | numbered/index original | 267 | 12969 | f062f78a0ec61099175f398eca1bea03db15e2cd7b7c613c4e00b4482cdf39b4 | docs/thefinalglance/C-plan/C-6_dashboard_design_tokens.md |
| DEEP-C-001 | numbered/index original | 248 | 14945 | 6121c760754f1e740b19f27b2657f41d5641ba1829562cbd59d0e3c6199aa835 | docs/thefinalglance/C-plan/C-18_test_infra_backfill_contracts.md |
| DEEP-C-001 | numbered/index original | 313 | 18242 | 395835b1e4e130ee1930ed3691d2a054ca8b98b070beeb4b0e421c088fd26127 | docs/thefinalglance/C-plan/C-22_feasibility_probe_completeness.md |
| DEEP-C-001 | numbered/index original | 321 | 17765 | d903d80738215782a580bc531e5bbc762f4d126f4889378a9646a68ad3b0c490 | docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md |
| DEEP-C-001 | numbered/index original | 213 | 10107 | 7939ad3cdbc8b53fab7c3c8f60dcbc6dd663a4371233aff3278b6edb5bd6a782 | docs/thefinalglance/C-plan/C-41_skills_driven_workflow_formalization.md |
| DEEP-C-001 | numbered/index original | 149 | 6658 | 76e6d4d6eede8079280606a287cc7e4c59f856b441aabbe766732364a70194eb | docs/thefinalglance/C-plan/C-51_aider_polyglot_harness_adoption.md |
| DEEP-C-001 | numbered/index original | 145 | 6380 | 61c5c8fdc74f007e85835ea4cde05e3f5fde1b919af00cd2dc7792efa19a0134 | docs/thefinalglance/C-plan/C-56_codebase_memory_dkg_31_repo_eval.md |
| DEEP-C-001 | numbered/index original | 178 | 7468 | 46c9cbab8161f7011797580a0a9efdcc1cf8803d95d87887b74d363a7769837e | docs/thefinalglance/C-plan/C-58_agentforge_rrk_reranker_head_to_head.md |

### DEEP-C-002

Family **C**; totals: **10 files**, **2552 logical lines**, **135174 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-C-002 | numbered/index original | 288 | 15390 | 61485f3c2bb18d40ff70effd99c6e7ce2b26b389c4cfc5e084f9922808579efd | docs/thefinalglance/C-plan/C-1_ssb_emission_knowledge_reasoning.md |
| DEEP-C-002 | numbered/index original | 298 | 16413 | 1efaf96308cd77b7e490167a6a0b87dde71cba2c98edea0ee4b11ba26368f6b1 | docs/thefinalglance/C-plan/C-4_tauri_commands_orchestration_cloud.md |
| DEEP-C-002 | numbered/index original | 345 | 18521 | 611982c5119fa992ebe5b757b0eb088cde483a6a69b9f3dd825b4d43d4f3d893 | docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md |
| DEEP-C-002 | numbered/index original | 256 | 13443 | fb080a4d27fd46d49ef888411df890ef3d066773d1249a0e52008acc07efdc1c | docs/thefinalglance/C-plan/C-12_per_level_auditor_navigator_wiring.md |
| DEEP-C-002 | numbered/index original | 328 | 19927 | 020c243e16a9aa80fbe67f1a293376ee6d6ab4851767df416adbd5e020a7ee6d | docs/thefinalglance/C-plan/C-19_test_infra_backfill_adversarial.md |
| DEEP-C-002 | numbered/index original | 316 | 17598 | 4e5fb5d72a96926c4ed261632d01dbc0a179c467fd30ab37aef911cf39b73d31 | docs/thefinalglance/C-plan/C-27_testing_trust_t0_wiring_contract.md |
| DEEP-C-002 | numbered/index original | 219 | 9793 | a59524220b15f09e0213a359f79ccbe7c64550261cf240281ee8debcb14b1ea8 | docs/thefinalglance/C-plan/C-42_semantic_gateway_zero_trust_fuzzing.md |
| DEEP-C-002 | numbered/index original | 203 | 9957 | e07ece1e15c5dece851eee4cdb95575e6052fa80ebb5bc0bdbf0814db546cb12 | docs/thefinalglance/C-plan/C-43_phase_4h_scope_freeze_ratification.md |
| DEEP-C-002 | numbered/index original | 142 | 6626 | 605bb7df86670c939852dd2fb4626632c91065b360f05c9d839061813b3f8763 | docs/thefinalglance/C-plan/C-50_contract_coding_test_scaffolding.md |
| DEEP-C-002 | numbered/index original | 157 | 7506 | 275a37cc2eb5cd37a919e43163c26ec7717d9a6806523fa02c044f9a73ef230d | docs/thefinalglance/C-plan/C-57_openhands_index_swe_cycle_swe_atlas.md |

### DEEP-C-003

Family **C**; totals: **10 files**, **2552 logical lines**, **140376 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-C-003 | numbered/index original | 270 | 14092 | 86d8423136fbb574f47360c399e940ab2e1ea6213d90b1638515e41e05ab3989 | docs/thefinalglance/C-plan/C-3_tauri_commands_knowledge_surface.md |
| DEEP-C-003 | numbered/index original | 238 | 15464 | 1c360b38a1360fae66c7b6e0e852d2c7bc2c3dce8ba4ed15334cacecb879dc9a | docs/thefinalglance/C-plan/C-7_dashboard_ux_completeness.md |
| DEEP-C-003 | numbered/index original | 323 | 18055 | 3b771f327a2b494502a2bb4b78a29c4aff8edeeb6534b981bb090513e3e18023 | docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md |
| DEEP-C-003 | numbered/index original | 257 | 14067 | 21de47602b05c8e2d7f7f5b7a656b056aecd6d055bdcc78cb0799fae92513445 | docs/thefinalglance/C-plan/C-11_sandbox_completeness.md |
| DEEP-C-003 | numbered/index original | 364 | 20302 | 8924390ecd517fe1cc5af0d4f042bc73f10b98e2c74e23665f3da600aec0414a | docs/thefinalglance/C-plan/C-14_phase0_debt_dead_code_strip.md |
| DEEP-C-003 | numbered/index original | 314 | 18888 | 616c65fec4fc18a753580277a1be34802d9904d384e4e3a683142450143122d5 | docs/thefinalglance/C-plan/C-20_test_infra_backfill_metrics.md |
| DEEP-C-003 | numbered/index original | 297 | 17417 | a48ec359ca59b19112f892e3543111fd9432d7965b1eb266b213ff46e2c03cf2 | docs/thefinalglance/C-plan/C-29_testing_trust_t2_t3_intent_metamorphic.md |
| DEEP-C-003 | numbered/index original | 149 | 6874 | bb840830ec7ea3ce082bee5a1dc04965965ad127c01f40e50244a2dd025c8566 | docs/thefinalglance/C-plan/C-45_cantante_contrastive_credit_attribution.md |
| DEEP-C-003 | numbered/index original | 146 | 6363 | 16ecc5b9415144f0d872075eb7c28c85a4b26600f4c7c1b165628c12e538275f | docs/thefinalglance/C-plan/C-48_safeharbor_memory_guardrail.md |
| DEEP-C-003 | numbered/index original | 194 | 8854 | db01ce1db99412d565e2477c1231de7e2777ef5f02c879cd3fc4c68f6c6909b2 | docs/thefinalglance/C-plan/C-59_cross_config_benchmark_master_report.md |

### DEEP-C-004

Family **C**; totals: **10 files**, **2651 logical lines**, **140924 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-C-004 | numbered/index original | 572 | 33447 | 7d83f7fc29b7f0ab7176a8d414e9528b10800082870c064b784316bd7182cdd4 | docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md |
| DEEP-C-004 | numbered/index original | 291 | 16011 | a3682aef7400c0ea72f0e869ffc14c4cdae08044ce75cd9dad222d46e0ad1f93 | docs/thefinalglance/C-plan/C-17_knowledge_indexing_resilience.md |
| DEEP-C-004 | numbered/index original | 263 | 14337 | 574bdaa442252088850291dea69a4a207ed91f7407f73fa1b65ec3b2a755d75c | docs/thefinalglance/C-plan/C-21_self_mod_anti_stale_automation.md |
| DEEP-C-004 | numbered/index original | 302 | 17106 | 1dbf30473c2c81d68ea1654fdb857619519dc7884def6bd44531ac9e123189fb | docs/thefinalglance/C-plan/C-25_compiled_pipeline_template_crystallizer.md |
| DEEP-C-004 | numbered/index original | 317 | 17743 | e91e5293d81da1d29c6363d88d0b18c802feba879c83661089d035e51576fcac | docs/thefinalglance/C-plan/C-26_constrained_decoding_retry_strategy.md |
| DEEP-C-004 | numbered/index original | 209 | 11120 | 838141d7c9c0b6de8c2e9856aac30b6efb9967b9edc781e4d51c045d64b81daf | docs/thefinalglance/C-plan/C-38_pagerank_production_contract_aider_spec_fix.md |
| DEEP-C-004 | numbered/index original | 147 | 6545 | 57d6d3add2451979a2f69e3f2526f256872c52ed53dcf948b42cc6c65b9b7120 | docs/thefinalglance/C-plan/C-46_uno_orchestra_parsimonious_routing.md |
| DEEP-C-004 | numbered/index original | 133 | 5675 | d1f5ae104bb65eb05ee11ed7d9e96e25c8ea460980d5ed0f1da962f318320ba6 | docs/thefinalglance/C-plan/C-47_case_based_calibration_tool_use.md |
| DEEP-C-004 | numbered/index original | 169 | 7150 | b722a8dc3544fd53ec0475ce8795215c757dd76b9f1c86b67e855c1737d8818a | docs/thefinalglance/C-plan/C-52_dafny_formal_verification_spike.md |
| DEEP-C-004 | numbered/index original | 248 | 11790 | d5c1efcb7bff65299413816e9a1f15fbb9bf9f4b45b58e2b01a63fd9a8bc8acc | docs/thefinalglance/C-plan/C-54_swe_bench_verified_run_three_configs.md |

### DEEP-C-005

Family **C**; totals: **10 files**, **2553 logical lines**, **139392 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-C-005 | numbered/index original | 335 | 17924 | 49325f3ae8452c9e2d3a8ac22641e32410087b73eff0caf18d586c84f0ab0c2d | docs/thefinalglance/C-plan/C-9_autonomous_infrastructure.md |
| DEEP-C-005 | numbered/index original | 262 | 16202 | 5f49d3bbdd19d0cb15738b3aea26f479fcabd66d4fe6da5588f9baef69b594fe | docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md |
| DEEP-C-005 | numbered/index original | 316 | 18649 | f322c804a752911069e837d7dbf84edfee0b0b9c2363d23a89230c9e2ad7b8a7 | docs/thefinalglance/C-plan/C-24_symbol_impact_dashboard_surface.md |
| DEEP-C-005 | numbered/index original | 302 | 17454 | 973bf9b1d3f6f1d91dcc04261fc04b96bfbb41c8cc6d47bf2153c08c730dbe3d | docs/thefinalglance/C-plan/C-28_testing_trust_t1_mutation.md |
| DEEP-C-005 | numbered/index original | 335 | 19191 | d43769eaae135e5a0ccbd1e5e59e786dd7ba6e77354080fbbc459734ab8eb1b2 | docs/thefinalglance/C-plan/C-32_code_completeness_skeleton_first.md |
| DEEP-C-005 | numbered/index original | 188 | 9628 | 1c8d63fea0654ea21a247fb899cb39fa4bc6af16dcbc89b4327080660a8fee28 | docs/thefinalglance/C-plan/C-34_empire_integration_deferred.md |
| DEEP-C-005 | numbered/index original | 274 | 14425 | 6526502744d6a24ecc8f19ef89feaa3835fd0207c1614f2d67c050a72f558851 | docs/thefinalglance/C-plan/C-35_tool_attention_dynamic_gating.md |
| DEEP-C-005 | numbered/index original | 239 | 12294 | f56e13556ce81e8725188277d640c47f77ba00854aa30e702c262eef419377ae | docs/thefinalglance/C-plan/C-36_mucoco_mutation_operator_catalog.md |
| DEEP-C-005 | numbered/index original | 166 | 7535 | e7540ed1c1ce1a0e237b43caf6a441fb2a8e2e9f7912c8562c555a51cb8b181e | docs/thefinalglance/C-plan/C-44_composer2_runtime_backend_tier.md |
| DEEP-C-005 | numbered/index original | 136 | 6090 | 5957e84a819e79366a1eee19b06fa84742ea2391b199ac30d943ced3c03ff4d4 | docs/thefinalglance/C-plan/C-49_iterative_distillation_prompt_policy.md |

### DEEP-C-006

Family **C**; totals: **10 files**, **2553 logical lines**, **136705 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-C-006 | numbered/index original | 295 | 17145 | 5d717b6ef26424d585fd9c4324e68d40c0e5852b586ca359a507b8198e94c2ba | docs/thefinalglance/C-plan/C-13_ipc_dispatcher_production_swap.md |
| DEEP-C-006 | numbered/index original | 322 | 18230 | 493315b967fc286b7a7dcaadf36e03eb7796777a7c67972940c116b3b2ce0aed | docs/thefinalglance/C-plan/C-15_phase0_debt_clock_entropy.md |
| DEEP-C-006 | numbered/index original | 249 | 14681 | 529b7fab69c5eb5ee27c2544a660874d7985ea83f5950dd610433a0dcc40aec4 | docs/thefinalglance/C-plan/C-23_sessiondag_branch_ui.md |
| DEEP-C-006 | numbered/index original | 314 | 17772 | a3e0aa3f1eb719f1de4e27671044ad6f59eb1dc2148166c769c28455d1e44ae0 | docs/thefinalglance/C-plan/C-31_code_completeness_file_plan.md |
| DEEP-C-006 | numbered/index original | 392 | 22068 | 36f9b52ee5652f5b6b4d1cab9be1eaad0a074078d2fcbcff0b73dd7ada018627 | docs/thefinalglance/C-plan/C-33_docs_phase_map_updates.md |
| DEEP-C-006 | numbered/index original | 202 | 10275 | 8847c1fc4babe514140a170f777b1a0deefea73b1dc6482de60257f5aef9854d | docs/thefinalglance/C-plan/C-37_per_role_decoding_defaults_reeval.md |
| DEEP-C-006 | numbered/index original | 215 | 10297 | f0a4a122795fa7261252284f200bb92fc3b7dc56fe87f2a21f94bb338da3dcc7 | docs/thefinalglance/C-plan/C-39_mcp_bridge_cli_surface.md |
| DEEP-C-006 | numbered/index original | 267 | 13250 | c452840361dc473d320f4d587499e83091c057c5e18be12e7114df5267b74d7f | docs/thefinalglance/C-plan/C-40_github_pr_adapter.md |
| DEEP-C-006 | numbered/index original | 152 | 6387 | 396ad0ef90bedd79873a8bc1e84dffc8dd54822ce52375d2fe06a6678c5e92fb | docs/thefinalglance/C-plan/C-53_nf4_kernel_watch_in_context_counter_test.md |
| DEEP-C-006 | numbered/index original | 145 | 6600 | 4827f148f2d72b3d7652254721ecfd35cf6caf28a44e1bd7d50a9a318f1144c8 | docs/thefinalglance/C-plan/C-55_aider_polyglot_leaderboard_submission.md |

### DEEP-D-001

Family **D**; totals: **8 files**, **1861 logical lines**, **89099 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-D-001 | numbered/index original | 370 | 18851 | dbd623d56e2c4323dc74478e919e6a4d5f590e323a3b32cbae405077a3d79db5 | docs/thefinalglance/D-plan/D-0_INDEX.md |
| DEEP-D-001 | numbered/index original | 292 | 14108 | 9db078102d56d8f0b8853a6e462c73f11400bc8770795ff3d68f32fb17e13a16 | docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md |
| DEEP-D-001 | numbered/index original | 337 | 17697 | 81ae25b3c2f47e8701fa437655da7ab9a143bff0e71dddb15bcf678ddd1ac4cb | docs/thefinalglance/D-plan/D-6_held_out_task_store.md |
| DEEP-D-001 | numbered/index original | 255 | 11790 | d41df45d74ab950e688f5c18ee7563d719f1f724bf7be510d3462c7625f000e5 | docs/thefinalglance/D-plan/D-12_operationalization_check_predicate.md |
| DEEP-D-001 | numbered/index original | 108 | 3991 | 02a583b93872475b9beca844488a064fe627d5996f5e381247f0537d722569cf | docs/thefinalglance/D-plan/D-23_sycophancy_detection_benchmark.md |
| DEEP-D-001 | numbered/index original | 188 | 8588 | 2326b8557579ba697a6ca74c77e35fb165d9609b9a4f2ecbb0dcd4ba77689f0d | docs/thefinalglance/D-plan/D-26_reasoning_template_dashboard.md |
| DEEP-D-001 | numbered/index original | 179 | 8078 | f1d7e75035b7d259f321e86030b0e897357159a60e385900f8a823deea7206bc | docs/thefinalglance/D-plan/D-27_mode_routing_refusal_dashboard.md |
| DEEP-D-001 | numbered/index original | 132 | 5996 | 279fc0b00f752321d019a73ffc718888a778d55019fba8e745b92ee5e406a5ba | docs/thefinalglance/D-plan/D-29_hierarchical_bai_research.md |

### DEEP-D-002

Family **D**; totals: **8 files**, **1855 logical lines**, **87723 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-D-002 | numbered/index original | 395 | 22049 | 9ce34c7a98f3fdeaa7231dd3038d6e01799341e89c426aa94279e0dc8e01770e | docs/thefinalglance/D-plan/D-1_reasoning_template_schema.md |
| DEEP-D-002 | numbered/index original | 241 | 11140 | 3b0df57f1daec68276f23c5450adb8d43aa30b63247cd831aa6adc6a5ee0d8bc | docs/thefinalglance/D-plan/D-8_crypto_archive_verify.md |
| DEEP-D-002 | numbered/index original | 331 | 15751 | b65d3f1e968c4165308395790b3f763e5d2b02719a8d0e32f7b5dcf4eaf02c4b | docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md |
| DEEP-D-002 | numbered/index original | 284 | 13147 | 228e9fccb6d8603ecdc686094f9e6957eeecbcd301771014ab37d35375f95154 | docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md |
| DEEP-D-002 | numbered/index original | 225 | 10499 | d0a300d465da3ef6d0da2774b092a275645b736eb50bbf2671012d9460cf5390 | docs/thefinalglance/D-plan/D-14_sycophancy_probe.md |
| DEEP-D-002 | numbered/index original | 122 | 4783 | 231181682761531f25ec4c93cd4b551363885da8e6695ed58c01ee8a7eee4294 | docs/thefinalglance/D-plan/D-21_mode_routing_coverage_benchmark.md |
| DEEP-D-002 | numbered/index original | 101 | 3359 | 3353c13e72b2f67a79d81bef25975e12cb915215b4fbde5a69c4a875e740df2e | docs/thefinalglance/D-plan/D-24_cross_domain_canary_regression_suite.md |
| DEEP-D-002 | numbered/index original | 156 | 6995 | 059a098165c07c89e49080ad1284197b81fec0f889298da4b2170be96c5c8ab5 | docs/thefinalglance/D-plan/D-28_adversarial_example_farming.md |

### DEEP-D-003

Family **D**; totals: **8 files**, **1862 logical lines**, **88049 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-D-003 | numbered/index original | 343 | 17256 | 3641a99de8b4709ff8a7e574830f6797be9f26ecfe17e4642a737375531dd04c | docs/thefinalglance/D-plan/D-2_ppm_sidecar_wiring.md |
| DEEP-D-003 | numbered/index original | 341 | 17622 | f9bacb3334a1441ed4fda1fef1149c678f574e9bdf42d7334d6457c328f507fa | docs/thefinalglance/D-plan/D-3_verification_result_aggregator.md |
| DEEP-D-003 | numbered/index original | 310 | 14877 | f36727d2f018e3f05cdae593bf27760e56531c4b9c9d4399172d10d8398980c7 | docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md |
| DEEP-D-003 | numbered/index original | 264 | 12347 | 97731ae36aa27f687453d09827818d5ee0390f46d7ab7ca92b3159fd3cdc73c2 | docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md |
| DEEP-D-003 | numbered/index original | 170 | 7108 | 7d0e7eaae7ba12c412fd4f2a737372aec827fdb0a488775cf196419e7d4e3bc5 | docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md |
| DEEP-D-003 | numbered/index original | 185 | 8309 | 194e83b8aaf365041958d4a8612ba548920d41237468c10b395015da0e450835 | docs/thefinalglance/D-plan/D-19_prmbench_adversarial_gaming.md |
| DEEP-D-003 | numbered/index original | 111 | 4082 | 20f72758b1a8165ae85d04058b167b562c29746faaad227b1414c135fcc68f11 | docs/thefinalglance/D-plan/D-25_determinism_archive_tamper_suite.md |
| DEEP-D-003 | numbered/index original | 138 | 6448 | f84076544ac127454260151e50f98013575af682e3f1bb40437953912d0ba923 | docs/thefinalglance/D-plan/D-30_calibration_bootstrap_corpus.md |

### DEEP-D-004

Family **D**; totals: **7 files**, **1755 logical lines**, **83367 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-D-004 | numbered/index original | 327 | 16650 | 0be0c4b31e3ef02c34d9f90da2fc873799a0e5baf77ce6c65b1b417a1b4f37eb | docs/thefinalglance/D-plan/D-4_tripwire_stack.md |
| DEEP-D-004 | numbered/index original | 269 | 12118 | 51818038b085919609f4736e7c4bca3145a0a5bc308c8ea87062d2f92d1a6291 | docs/thefinalglance/D-plan/D-7_cross_domain_canary.md |
| DEEP-D-004 | numbered/index original | 404 | 20681 | 9af38bb296549864faf9742f140ba5e7535d87a0a9084408e9ddb6e2c8371ae2 | docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md |
| DEEP-D-004 | numbered/index original | 229 | 10638 | c0301438b1372f9c13a098ce52b936742fc94f39d19e805cab519dea8e0e2871 | docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md |
| DEEP-D-004 | numbered/index original | 246 | 11835 | aa717c6562a29091d06e595274b8115fdd2a67f0f926c29871d6021f5021a3fe | docs/thefinalglance/D-plan/D-18_observable_effect_typed.md |
| DEEP-D-004 | numbered/index original | 167 | 7162 | 5f86d79ffda38e2bae156b4cc885e3637455d4888fd9b5dfeac2f5652d3a894b | docs/thefinalglance/D-plan/D-20_arc_agi2_ood_probe.md |
| DEEP-D-004 | numbered/index original | 113 | 4283 | b4c67ffe790e5d2985df587f220e79cff6e9820cde53d0fb788ec8a4b8f08a48 | docs/thefinalglance/D-plan/D-22_conformal_calibration_validation.md |

### DEEP-E-001

Family **E**; totals: **9 files**, **2134 logical lines**, **239903 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-001 | numbered/index original | 270 | 18289 | 8fad1c0a31a6ff9c8ffa828c6e2805fd836cc4f416e0c0fc458d432401f525f1 | docs/thefinalglance/E-plan/E-0_INDEX.md |
| DEEP-E-001 | numbered/index original | 184 | 22340 | f6a35d5f9efd8d8ed61751cd54e65d32b2f2e9f1661cda21f4a7cb7e41d07bfe | docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md |
| DEEP-E-001 | numbered/index original | 198 | 19422 | 3f46328b0820ccafb999c4ae74df2f2cf1f5eb489f50376ae8831b567b590b22 | docs/thefinalglance/E-plan/E-6_multi_agent_failure_modes.md |
| DEEP-E-001 | numbered/index original | 229 | 23553 | 3dce448358fc23accc53938f8ec1b8e8091220d6cc230bb9399a965192983db9 | docs/thefinalglance/E-plan/E-7_darwinian_self_improvement.md |
| DEEP-E-001 | numbered/index original | 256 | 29214 | 6a7c915beeb5a5e17de77256a6f71f8b9aad65d1ede8673b0c386a35e7d5ee59 | docs/thefinalglance/E-plan/E-19_reasoning_quality.md |
| DEEP-E-001 | numbered/index original | 233 | 29080 | 1c34c64c706133e751d8cea6ba917b5ffa844e5689c1b52c4a8a9724cca5e2f5 | docs/thefinalglance/E-plan/E-36_document_vision_rag.md |
| DEEP-E-001 | numbered/index original | 237 | 29480 | 054f001ed639b0b9ad76d6347139a7ec241b2d3c62c2e0eba3585f61b6e6d970 | docs/thefinalglance/E-plan/E-38_ai_roadmap_2026_signals.md |
| DEEP-E-001 | numbered/index original | 202 | 23056 | 94bff276a560a86888f0f9e89aa65803162d8ffd8ec34fdcb31161e6e0f30ecf | docs/thefinalglance/E-plan/E-42_tree_sitter_grammars_2026.md |
| DEEP-E-001 | numbered/index original | 325 | 45469 | 254ac17c31ab7de92783328a0ae3381be96f170a512906a51f1005e6560c5489 | docs/thefinalglance/E-plan/E-55_agentic_rl_and_reward.md |

### DEEP-E-002

Family **E**; totals: **9 files**, **2131 logical lines**, **251948 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-002 | numbered/index original | 209 | 28213 | 3f4e0b3c189eeddef277c83c2277db24d1481f6eb2231e7df8e00a62c01f6d25 | docs/thefinalglance/E-plan/E-1_infinite_context_chat_illusion.md |
| DEEP-E-002 | numbered/index original | 200 | 24403 | 502b84c4ee3bfc91ee1c94f8b8b5d1e16c7560869015a9de3c3e8856427ddd35 | docs/thefinalglance/E-plan/E-3_model_handoff_ux.md |
| DEEP-E-002 | numbered/index original | 217 | 24950 | 4627c7cff910c274e50b71e9087d8c48958f1924d64173754df9ff08e3e0756b | docs/thefinalglance/E-plan/E-14_browser_e2e_agents.md |
| DEEP-E-002 | numbered/index original | 181 | 17170 | 562b0f096a64febd1dc82af1bedd4a63f75e54fe6e16c2bfef19a3ee59570e73 | docs/thefinalglance/E-plan/E-16_vlm_image_validation.md |
| DEEP-E-002 | numbered/index original | 251 | 22214 | b87ac81c7b9421ad6aab068e45a14e6b896a6c1ecbedea214c16697bc9adddfc | docs/thefinalglance/E-plan/E-17_rag_retrieval_quality.md |
| DEEP-E-002 | numbered/index original | 231 | 24911 | 480d16062314089e229723832c96a9f2f2b6afe58358ef585e9544b3d781ec6e | docs/thefinalglance/E-plan/E-34_rust_ai_ecosystem_2026.md |
| DEEP-E-002 | numbered/index original | 258 | 33705 | 54543d18aecca52d7331db3cb1ea2bf7f68fd0a4e2030d545f937993e6e2c4db | docs/thefinalglance/E-plan/E-39_cost_token_economics.md |
| DEEP-E-002 | numbered/index original | 280 | 35454 | 54868188d615640e830e53237928e84fc2c7cf666c3d5ccf4f893f2b380a94f5 | docs/thefinalglance/E-plan/E-49_coding_agent_metrics.md |
| DEEP-E-002 | numbered/index original | 304 | 40928 | 01f93a3d63fdd7b76a420a658f79433e3e3b7dae0896d65004169eec6df45f07 | docs/thefinalglance/E-plan/E-53_formal_verification_codegen.md |

### DEEP-E-003

Family **E**; totals: **9 files**, **2141 logical lines**, **249792 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-003 | numbered/index original | 230 | 27509 | 35c3e70126542558e72c52d6de18980d95073c01194f99a20c7b110b479f7d59 | docs/thefinalglance/E-plan/E-4_chat_ux_critique.md |
| DEEP-E-003 | numbered/index original | 254 | 23436 | f6131b6454c00aa21a4a29fbb9d57fddbeba62b2aad0dfa993c3c457f07fd5dc | docs/thefinalglance/E-plan/E-9_agent_eval_benchmarks.md |
| DEEP-E-003 | numbered/index original | 193 | 22550 | f7d9b36475c890d2e9f36c15d3d0645f707bf7d8a5badf6c396784a96e5afa5f | docs/thefinalglance/E-plan/E-12_production_fullstack_codegen.md |
| DEEP-E-003 | numbered/index original | 207 | 20485 | 383ee956b5f6a53422c550ce709f5bd28fa487ab7250cdd9a8e97df0229b92f3 | docs/thefinalglance/E-plan/E-18_tool_use_reliability.md |
| DEEP-E-003 | numbered/index original | 265 | 38777 | 09f90367279bf56ab56f617e3572e2b0f8e25f9f88dd21f34dac3a448cce0627 | docs/thefinalglance/E-plan/E-26_inference_engines_2026.md |
| DEEP-E-003 | numbered/index original | 195 | 20236 | 06fcc374e1a364489477ec826f5411ac2023939851cc52a75a79840cd1e08079 | docs/thefinalglance/E-plan/E-27_quantization_2026.md |
| DEEP-E-003 | numbered/index original | 214 | 23462 | b9421eab36e28a8343ea1d052165451255d7ef2b33e3e61639fcb671bc12c58c | docs/thefinalglance/E-plan/E-32_context_window_extension.md |
| DEEP-E-003 | numbered/index original | 237 | 29056 | e98765d11070553619d9951d1e0eabe4e5247b54a1c4f5974b7902e6f680c6b7 | docs/thefinalglance/E-plan/E-44_checkpointing_durability.md |
| DEEP-E-003 | numbered/index original | 346 | 44281 | 7af1c532f18884142e4a5fd9572666b428bbec5078a2665481c94e8c657d9557 | docs/thefinalglance/E-plan/E-54_eval_corpus_design.md |

### DEEP-E-004

Family **E**; totals: **10 files**, **2302 logical lines**, **274448 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-004 | numbered/index original | 306 | 38449 | 62f9363c6a8b85504f7e99f3160262d55330a406f443fc27b867a35d64cea348 | docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md |
| DEEP-E-004 | numbered/index original | 215 | 22065 | a659b75ba79e6acd6316c922a29ba9738f5362fac4c7ca168e085184fd4b8e3d | docs/thefinalglance/E-plan/E-8_self_evolving_prompts.md |
| DEEP-E-004 | numbered/index original | 251 | 25555 | c2c7addd1c5e3fb9a0d6cc7efcdd1f43683a46448b853e3b46a12ba24eca9995 | docs/thefinalglance/E-plan/E-20_local_cloud_routing.md |
| DEEP-E-004 | numbered/index original | 174 | 20362 | 49402af32aa8c6d6954ab87b56c5a78000db67389376f5df8ac1c7c7f3a87d20 | docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md |
| DEEP-E-004 | numbered/index original | 177 | 20635 | c328f0391feb65e083c113def69e7e29a5482c41c938d5a7d3ea300a5c1ff87a | docs/thefinalglance/E-plan/E-22_github_trending_may2026.md |
| DEEP-E-004 | numbered/index original | 230 | 29091 | 1e47656298584ac4fb47b8e01dbf8d2ec9ff3d94fb773ba11b138f1c0a23e359 | docs/thefinalglance/E-plan/E-25_claude_code_architecture.md |
| DEEP-E-004 | numbered/index original | 209 | 25312 | c41b7e7b7a5397535bbe9e61d778b9a2b86d5121d3ec0b6485ef6810654b0e56 | docs/thefinalglance/E-plan/E-29_agentic_observability.md |
| DEEP-E-004 | numbered/index original | 277 | 33592 | ebbe04ab238e9e44d3824d2a6de8f7448046005cdcdfbae2379294d55ab92c7d | docs/thefinalglance/E-plan/E-37_prompt_injection_defense.md |
| DEEP-E-004 | numbered/index original | 200 | 23983 | 7c4ef885c8503e51d06730ef6f0136026d5915272abddb3f11588bd897856c20 | docs/thefinalglance/E-plan/E-41_ci_cd_pr_review.md |
| DEEP-E-004 | numbered/index original | 263 | 35404 | b20c1e805a3719f6661417e8826dab788be1452aa3676caed4a94179601d3b00 | docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md |

### DEEP-E-005

Family **E**; totals: **9 files**, **2137 logical lines**, **254945 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-005 | numbered/index original | 237 | 24994 | 01364088b471d5daf3d96b8a2590dff395c0cd02e921040925ffee775f96101f | docs/thefinalglance/E-plan/E-10_critic_of_elai_design.md |
| DEEP-E-005 | numbered/index original | 205 | 22178 | d2455876c798ee5eaff2439e875b06fb6b30f9b99b719d15f1e97c3594d17598 | docs/thefinalglance/E-plan/E-13_tdd_verification_agents.md |
| DEEP-E-005 | numbered/index original | 235 | 28137 | bb10d0dcec97785b8e56a213577e9d203d6797803b60c2e69d60f616e96a5da3 | docs/thefinalglance/E-plan/E-15_sandboxed_exec.md |
| DEEP-E-005 | numbered/index original | 195 | 19981 | ddcb9ec59a5a0ceaf6d9f4c81626fca69159f0e091f1b1bd7abc3b1a15fdd79c | docs/thefinalglance/E-plan/E-28_attention_kernels_2026.md |
| DEEP-E-005 | numbered/index original | 189 | 22018 | f1c88b75edb3266fca90919dda7f59eee8adadb8619f41251be3ebbd09d1992c | docs/thefinalglance/E-plan/E-31_coding_model_leaderboard.md |
| DEEP-E-005 | numbered/index original | 257 | 33053 | 5d8a4c238a6fd5573ba29e836ac03a03dbba4ac8fb32a25677c2b7bc0a782b26 | docs/thefinalglance/E-plan/E-33_dashboard_ux_agentic.md |
| DEEP-E-005 | numbered/index original | 226 | 24164 | 59b840f934ad713035e7e25e03c88605378cb6be0ea2efe523d72d2a13a7dd9c | docs/thefinalglance/E-plan/E-43_agent_skill_systems.md |
| DEEP-E-005 | numbered/index original | 316 | 44365 | 71f79781566132e202189d48b9af9882a30aadd7dad5a8e4d38355fc57a91d46 | docs/thefinalglance/E-plan/E-50_saas_architecture_patterns.md |
| DEEP-E-005 | numbered/index original | 277 | 36055 | 5d6dff6f8867458dc8db3f9333c0b9a8eebaa44c14c902337d364268033af45f | docs/thefinalglance/E-plan/E-52_lsp_integration.md |

### DEEP-E-006

Family **E**; totals: **10 files**, **2296 logical lines**, **284438 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-006 | numbered/index original | 167 | 20506 | 413367c052e5999ceb1a48aa2005ac6a164fe3fcdee10d6b82918d82fc3020a6 | docs/thefinalglance/E-plan/E-11_idea_to_saas_prior_art.md |
| DEEP-E-006 | numbered/index original | 179 | 20160 | 5aa3fc49ff6f8b88724e24cd4c6af3f16cb4c2ed3bff5453d9fd87de2ff3f560 | docs/thefinalglance/E-plan/E-23_developer_discourse_2026.md |
| DEEP-E-006 | numbered/index original | 199 | 23263 | c223a7adddab58256c04e64c060d3c191d5e1d4940d6a99380331e96ba6ee2c5 | docs/thefinalglance/E-plan/E-24_agent_protocols_mcp.md |
| DEEP-E-006 | numbered/index original | 292 | 34960 | 1c5fd429280daf40759b134e53cb3607441dd56b3598a249a55d433945f76560 | docs/thefinalglance/E-plan/E-30_public_agent_incidents.md |
| DEEP-E-006 | numbered/index original | 252 | 31527 | a7402ff411d695c1bfa3f78f47901fa78444d6a97cd77e901316813375b9c8f8 | docs/thefinalglance/E-plan/E-35_structured_output_2026.md |
| DEEP-E-006 | numbered/index original | 283 | 36559 | 9a4a7b4ec0e3dbd8e444cb44db82ee0807c11e33132b90e158f85b62e5aa4c02 | docs/thefinalglance/E-plan/E-40_symbol_graph_tools.md |
| DEEP-E-006 | numbered/index original | 232 | 26150 | 553d3c62662d2defb2928e1337b7231f0b6228f6195b21ef960903b7f57e99ab | docs/thefinalglance/E-plan/E-45_cross_language_drift.md |
| DEEP-E-006 | numbered/index original | 219 | 30223 | 85283ecc757de0c916f97caf90803bfd492765959e43b176bb60bac449b47f8a | docs/thefinalglance/E-plan/E-46_planning_decomposition.md |
| DEEP-E-006 | numbered/index original | 209 | 26039 | e268eb470450820aace82d5833ef90275b2a3918e2a9c49f08aa003097ecc238 | docs/thefinalglance/E-plan/E-47_lora_finetuning.md |
| DEEP-E-006 | numbered/index original | 264 | 35051 | bbf4aadbc8d599e11bc30c2e5b1419b9e1862e714e3e5bb50e06188eedd1bf9d | docs/thefinalglance/E-plan/E-51_edge_inference_and_mlx.md |

### DEEP-F-001

Family **F**; totals: **8 files**, **2188 logical lines**, **322018 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-F-001 | numbered/index original | 233 | 17485 | e3ee054392549bdfe5d1b2a98d1c6655fa69315af91690fdcc34e0970eb0d868 | docs/thefinalglance/F-plan/F-0_INDEX.md |
| DEEP-F-001 | numbered/index original | 217 | 36324 | ae65098ff39db55e08a04dc05ac63a536ec198e2eb0589e8d378659628f72dc5 | docs/thefinalglance/F-plan/F-4_test_integrity_cant_lie.md |
| DEEP-F-001 | numbered/index original | 242 | 42472 | 79b45385f3006784902c4f74f680ff8dbff3b8658cba86ec7dd74836b50c0e02 | docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md |
| DEEP-F-001 | numbered/index original | 238 | 41399 | f6ec6d435203788582eabe709ac8fa32771d435839f5b92a402b971818c0d29f | docs/thefinalglance/F-plan/F-9_hallucinated_dep_writetime_gate.md |
| DEEP-F-001 | numbered/index original | 220 | 39238 | 25c9f983cbf1d39ec8e497dbdde633755f02de9130f6d1d962062065bba4d95f | docs/thefinalglance/F-plan/F-10_security_monotonicity_iter_guard.md |
| DEEP-F-001 | numbered/index original | 258 | 50021 | 5605f2402d271d79445f0dfab3d0f88a5ece101a715100b0f7f18cdbdd1f0c3c | docs/thefinalglance/F-plan/F-11_close_one_config_generation.md |
| DEEP-F-001 | numbered/index original | 267 | 55558 | c4ea77cbd0e5f0d0b4650b35ea2c32e2c5746cb678ee206795415d2e1e0cc922 | docs/thefinalglance/F-plan/F-12_semantic_contradiction_local_nli.md |
| DEEP-F-001 | numbered/index original | 513 | 39521 | 2ec94c5367fa7955b34fb38c36583e06dff6836a3b8a8a37e41f729199c67ad6 | docs/thefinalglance/F-plan/F-13_session_archive_schema_bump.md |

### DEEP-F-002

Family **F**; totals: **8 files**, **2097 logical lines**, **292736 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-F-002 | numbered/index original | 220 | 36413 | c5663fe857fcfdeabe30d23bd1015a6db790a2e6a9ee4b03fadda4590ed249ce | docs/thefinalglance/F-plan/F-1_vibegate_unified_write_gate.md |
| DEEP-F-002 | numbered/index original | 276 | 42687 | ff1e10d7270dbd77a1486344857b49c1387cf1364e685e6c184e1b945398bac3 | docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md |
| DEEP-F-002 | numbered/index original | 253 | 48443 | e6e072e9ab7be5534964428aafa2d9df0e1e90cd5b6d150c88713007ea171ab0 | docs/thefinalglance/F-plan/F-2c_low_fp_cwe_rules_seccodeplt_baseline.md |
| DEEP-F-002 | numbered/index original | 232 | 37550 | 50cdf05501a571650f3f08785256a7e2f3f4288644ce8ba4fcf736773aa69b88 | docs/thefinalglance/F-plan/F-3_camel_toolfilter_dataplane.md |
| DEEP-F-002 | numbered/index original | 235 | 34502 | a0c9721572136ec6a6112d5a54644c8afb9881d1639048f7a1239b293c3c1a73 | docs/thefinalglance/F-plan/F-5_staleness_quarantine_cascade.md |
| DEEP-F-002 | numbered/index original | 498 | 41255 | 4ddb67d8bf9b34535f523c6769bc6fe64e67d836e51fee1b4d17b3ab1ddf70a5 | docs/thefinalglance/F-plan/F-6_learned_skill_library.md |
| DEEP-F-002 | numbered/index original | 263 | 42837 | e4ad7c84da7317d6b28d26057d5fa3c11f7d89f79aa931d2a3ac9214baf93443 | docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md |
| DEEP-F-002 | auxiliary/grounding/fixture | 120 | 9049 | 15742a1e69d2e45d76140543f58e44241852a766530cfb3bd3ad391409eec7bb | docs/thefinalglance/F-plan/F-WIRING-DOCTRINE-WHY.md |

### DEEP-G-001

Family **G**; totals: **6 files**, **1723 logical lines**, **151046 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-G-001 | numbered/index original | 203 | 16661 | 9f4d4b682e5cb23e9f9de7693aa5983610f36b30dd82f8ef89c756c4224f7223 | docs/thefinalglance/G-plan/G-0_INDEX.md |
| DEEP-G-001 | numbered/index original | 308 | 26015 | 673d3ff40364f44dd1859de50a337f23b00190344740c835315878a72ebcc52d | docs/thefinalglance/G-plan/G-1_taint_propagation_dataplane.md |
| DEEP-G-001 | numbered/index original | 307 | 26829 | d2f67733390d909e731e64cd44da36a90f6be3d6d868b5740de4cf3275279453 | docs/thefinalglance/G-plan/G-3_blast_radius_gates.md |
| DEEP-G-001 | numbered/index original | 252 | 20236 | 65e86605f188ae78ce4971068727df0d6e40d4aefd59807e134f0dea046e9ef3 | docs/thefinalglance/G-plan/G-4_mcp_rugpull_repin.md |
| DEEP-G-001 | numbered/index original | 293 | 26579 | b2bfd089120de0a6dcda991722f1e3953b772aaddf331056d4271185a4ede052 | docs/thefinalglance/G-plan/G-5_reviewer_cockpit_dsr.md |
| DEEP-G-001 | numbered/index original | 360 | 34726 | 9244c46c562faec04aa7f619b5fceb13c736e9fa07ff0ab450d29b0003a7139e | docs/thefinalglance/G-plan/G-7_confidence_producer_dual_llm.md |

### DEEP-G-002

Family **G**; totals: **5 files**, **1539 logical lines**, **137029 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-G-002 | numbered/index original | 279 | 22953 | b70c014def1b95932dc4d7a5c0d0a795bd249aab130b05140d7be82d7b5b28d8 | docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md |
| DEEP-G-002 | numbered/index original | 301 | 27117 | d3ad5003f857dd354915857497bf1d215d5154a585763a32a2736df340342d07 | docs/thefinalglance/G-plan/G-6_safedelta_benchmark.md |
| DEEP-G-002 | numbered/index original | 310 | 29442 | 0e8b82f6d21e5e715debb93e7f8a7e4378e87a468c8039b3e42e52cfa311fce4 | docs/thefinalglance/G-plan/G-8_intoto_attestation_article50.md |
| DEEP-G-002 | numbered/index original | 308 | 28308 | 709b3f6ab79dee2354470dda33a9078c7d701dd56daf5ce2196027894a126ae3 | docs/thefinalglance/G-plan/G-9_journal_redaction_ttl.md |
| DEEP-G-002 | numbered/index original | 341 | 29209 | b04b65f60259d84979c537bf310366b83b0a55bfb65d3fe8e1823d87c9ea2f56 | docs/thefinalglance/G-plan/G-10_model_drift_regression_canary.md |

### DEEP-H-001

Family **H**; totals: **9 files**, **1499 logical lines**, **197472 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-001 | auxiliary/grounding/fixture | 374 | 81705 | 8eec614056da0e92512037a4cc5cdff519b758adb6537c8ecd2a2546ac679321 | docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md |
| DEEP-H-001 | auxiliary/grounding/fixture | 48 | 8685 | 40ea6fe11bafaa2de2e24a8dd2971dc964f36430099a072d4f4bf05953d4e21b | docs/thefinalglance/H-plan/_grounding/H-UX-MEMORY-GROUNDING-MAP.md |
| DEEP-H-001 | auxiliary/grounding/fixture | 163 | 16337 | 30d032f5e560771c1ebb0920566e772ffb70210198108a4ef77c9f00a69ffebc | docs/thefinalglance/H-plan/_grounding/H-VERIFICATION-GROUNDING-MAP.md |
| DEEP-H-001 | numbered/index original | 192 | 16711 | 3e354b8538656e9d06dc777d75cf08e4d47714014754c6e0c0838738b9d0f95e | docs/thefinalglance/H-plan/H-9_cost_cockpit_dashboard.md |
| DEEP-H-001 | numbered/index original | 185 | 16503 | dae3993560602d022db730a739f91d606fbb40afa7ed303292d2ee4c2f8f730d | docs/thefinalglance/H-plan/H-11_live_layer_file_explorer_fs.md |
| DEEP-H-001 | numbered/index original | 181 | 15186 | c9f710a0e83b3f9d3a102d45efded686fbe0200b7ee04b49ea551488e372c052 | docs/thefinalglance/H-plan/H-20_leakage_taint_composite.md |
| DEEP-H-001 | auxiliary/grounding/fixture | 124 | 11060 | 79c5b79dd9e804f7b6081a0bf692a6cb267ee4a9cfc7a7a567528a41c8b565c0 | docs/thefinalglance/H-plan/H-28-WIRE-1_reward_gate_darwin_admit.md |
| DEEP-H-001 | numbered/index original | 58 | 16304 | 9837fb42e7f53899e34c4fa76b9c4d7685c5183ff1624ab2a0773c783ff171e3 | docs/thefinalglance/H-plan/H-35_rewind_over_ssb_journal.md |
| DEEP-H-001 | numbered/index original | 174 | 14981 | eaa8357ef13cec49031341d85fbd95b95fdaf97c5d3b5812984ab4ccd4514251 | docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md |

### DEEP-H-002

Family **H**; totals: **9 files**, **1484 logical lines**, **153229 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-002 | auxiliary/grounding/fixture | 153 | 14781 | 5383df80092bda52557751d59d5cc776cc4c80fa3fdd5d13099e102c119f826d | docs/thefinalglance/H-plan/_grounding/H-REASONING-GROUNDING-MAP.md |
| DEEP-H-002 | auxiliary/grounding/fixture | 293 | 29136 | 21efdffcf035fcb726f7fbf02e01acd52c5f8a1c00a5684ea31a568d77b00f55 | docs/thefinalglance/H-plan/_grounding/H-TRUST-GROUNDING-MAP.md |
| DEEP-H-002 | numbered/index original | 196 | 16488 | e774afc9d58af08ab3054b2d65856bfeeeca4e7f76d02fc1406c65ab2267d83e | docs/thefinalglance/H-plan/H-3_output_token_diff_fim_streaming.md |
| DEEP-H-002 | numbered/index original | 176 | 14255 | 4cec8e1da48bbeb639c1c6083e97e715ca5e036ac5ea9eccc2deae65d035a5c7 | docs/thefinalglance/H-plan/H-5_batch_api_noninteractive_lanes.md |
| DEEP-H-002 | numbered/index original | 78 | 15246 | 51762414d3124c598f66b41f84fd00ad8557029442b36c5c11bb300f6bfd225c | docs/thefinalglance/H-plan/H-38_cost_confirm_taskstop_podstop.md |
| DEEP-H-002 | numbered/index original | 164 | 13197 | 1dfc508ba2d62734382c18e7d17ccd5afddbcea866a80704a6ec5d47dd7af21a | docs/thefinalglance/H-plan/H-40_decorrelation_keyed_verifier_retention.md |
| DEEP-H-002 | numbered/index original | 182 | 15299 | d9efb3fd8ab447ca27ffac1f7cca46eddd9a9d34a20f84a2c3b7be9b9be92c56 | docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md |
| DEEP-H-002 | numbered/index original | 186 | 16397 | 45a66e3e2dc45b9ae92162fa5072733b265852edf52ea3ad6d1a6468e3f9f976 | docs/thefinalglance/H-plan/H-45_worstcase_lineage_promotion_scoring.md |
| DEEP-H-002 | numbered/index original | 56 | 18430 | 1ac48ae75e5c5cea09742e9897c5dface12ce5b060507a332c11a2e0ac87ab93 | docs/thefinalglance/H-plan/H-54_obligation_closure_escape_hatch_gate.md |

### DEEP-H-003

Family **H**; totals: **5 files**, **1454 logical lines**, **166004 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-003 | numbered/index original | 987 | 100252 | 176e2c125b4a61daaba4cf6a5032bbc11c86bfb9592840a62d04023fe15af67c | docs/thefinalglance/H-plan/H-0_INDEX.md |
| DEEP-H-003 | numbered/index original | 167 | 13758 | 971c58db1967760106d7975a6230a1cdaa55d2426f513b3ce4b10feaecf852fe | docs/thefinalglance/H-plan/H-12_plan_research_modes_chrome.md |
| DEEP-H-003 | numbered/index original | 154 | 11685 | c2221c0ef3f31b876be8a4db0e47e3c24c1cd4ff36a95865bda70f0039678de7 | docs/thefinalglance/H-plan/H-19_reusable_holdout_budget.md |
| DEEP-H-003 | numbered/index original | 81 | 19202 | a267395a04bcecff474209a8ddcf9b03e2ea39ed0636dc516fd19e802ae378d6 | docs/thefinalglance/H-plan/H-32_code_grounded_validate_before_use.md |
| DEEP-H-003 | numbered/index original | 65 | 21107 | 9269045e1384aeabac0a7dafac6544c621dfb02c04924e4a0afc8616dcc3d3a4 | docs/thefinalglance/H-plan/H-33_memory_hub_governance_provenance.md |

### DEEP-H-004

Family **H**; totals: **9 files**, **1452 logical lines**, **153591 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-004 | numbered/index original | 199 | 17231 | ba8ec7494f463d6dd520bfbc27fe7c15ac20e5a4b7cc5ed79958bd0b7cdb2f0f | docs/thefinalglance/H-plan/H-1_prompt_caching_production_path.md |
| DEEP-H-004 | numbered/index original | 183 | 14603 | 8462ba3d59bb28f837e8ed6a547132a715b4cd30451a9d8fee0cd9e1ae0b6664 | docs/thefinalglance/H-plan/H-2_cache_stable_layout_breakpoints.md |
| DEEP-H-004 | numbered/index original | 190 | 17358 | 50a7fc0ff067efcbabd6063912f5d83b4a4a97316362805f6afe63e8027cc9ec | docs/thefinalglance/H-plan/H-14_design_mode_web_first.md |
| DEEP-H-004 | numbered/index original | 170 | 14804 | 912007044c0325a81e065b37ebb8e98a43e4942a7fffc0be5fd11417ba7bdf06 | docs/thefinalglance/H-plan/H-22_trading_metamorphic_null_composite.md |
| DEEP-H-004 | numbered/index original | 159 | 13040 | 0df2211e3277109d497a10529b46a71348d5a50fd88c0a1654d5f262be440df5 | docs/thefinalglance/H-plan/H-24_sim_to_real_parity.md |
| DEEP-H-004 | numbered/index original | 178 | 15141 | e2b48cf7acc1274d901c7b7ee814142ec8f64b84b0e56b22ab6b25d1be4bdb17 | docs/thefinalglance/H-plan/H-27_gate_self_validation.md |
| DEEP-H-004 | numbered/index original | 211 | 19143 | bd45662c99682012e88dbd4ea44c1e3ad784154dd371a4ed544c8e029035b158 | docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md |
| DEEP-H-004 | numbered/index original | 101 | 23826 | ee7d9e6f6bede4c6fc051dbc40b54c0e64932cfae6777f7d6b56167d7501a5e3 | docs/thefinalglance/H-plan/H-49_undo_redo_branching_timetravel.md |
| DEEP-H-004 | numbered/index original | 61 | 18445 | e9b67c78d692e8492cbc886503f730935b18bc337c6c349671654d257d2df85d | docs/thefinalglance/H-plan/H-53_citation_prebind_constrained_synthesis.md |

### DEEP-H-005

Family **H**; totals: **9 files**, **1451 logical lines**, **149987 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-005 | numbered/index original | 183 | 15181 | 6631f0ce3c130430c00ce6a22433f1ec6ceccdde97a42843d8e84f51f4a4bc45 | docs/thefinalglance/H-plan/H-4_tool_search_beta_header.md |
| DEEP-H-005 | numbered/index original | 196 | 18067 | fae2d1681c50081ec150f7e0725eb3f94750dcbd797f52909e8f83583bfb2e56 | docs/thefinalglance/H-plan/H-10_canvas_shell_code_graph.md |
| DEEP-H-005 | numbered/index original | 157 | 12529 | c68d739a8f6915a3cd3c8c6cb1fa03deb63fc2c525f03ee09d9f5f26a8a03624 | docs/thefinalglance/H-plan/H-21_runtime_lookahead_differential.md |
| DEEP-H-005 | numbered/index original | 170 | 13988 | 91839bc23fac6a22efe287e9a74e69d414dd1af30c389dcf71e48be246ba7e5d | docs/thefinalglance/H-plan/H-25_result_manifest_rederivation.md |
| DEEP-H-005 | numbered/index original | 100 | 21375 | 567916ee25ae528058dd394f4a0896d5a56022385d9e6d936fc0a3e54ce1c300 | docs/thefinalglance/H-plan/H-31_user_decision_bitemporal_ledger.md |
| DEEP-H-005 | numbered/index original | 187 | 16346 | 6a11647451537b53e832889dd9b30ad0616d36e0bac449029c4d7f3670c0ded7 | docs/thefinalglance/H-plan/H-44_behavioral_trap_detection_recall.md |
| DEEP-H-005 | numbered/index original | 222 | 19667 | dd78d5f76058527483d81c9d574bced024b374d3a2d1c985fcc1fab86c259b2c | docs/thefinalglance/H-plan/H-47_dgm_eval_firewall_dategate_templateregen.md |
| DEEP-H-005 | numbered/index original | 177 | 15522 | 55976febf487ec58ebe6ff6c959fed04702e68780689c9861f1381eec57eebaa | docs/thefinalglance/H-plan/H-48_wire_bandit_on_real_reward.md |
| DEEP-H-005 | numbered/index original | 59 | 17312 | f71167566b0e3ea92643000783d6f06c7fb54e17f7e00d216ba7b5849cbbd6f2 | docs/thefinalglance/H-plan/H-52_deterministic_finding_anchoring_cockpit.md |

### DEEP-H-006

Family **H**; totals: **10 files**, **1495 logical lines**, **159276 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-006 | numbered/index original | 187 | 15448 | e9769453ae7948564d79fd4fa19b77125cddf53f42998bfc1b521d60adf8ddd1 | docs/thefinalglance/H-plan/H-6_observation_masking_memory_readback.md |
| DEEP-H-006 | numbered/index original | 229 | 18991 | 15db2f5950891147501f3554b3bb4765ccf8b4d065de97557a56337c6a8b32c9 | docs/thefinalglance/H-plan/H-7_calibrated_routing_architect_editor_verifier.md |
| DEEP-H-006 | numbered/index original | 156 | 12103 | 0fbaed9c8aa1dbc2567d23a51d17393fd98fe3ccf2385bf5d1bf482cb24e5da7 | docs/thefinalglance/H-plan/H-8_persist_oracle_answer_cache.md |
| DEEP-H-006 | numbered/index original | 196 | 16323 | 5db7b0a2cab23ca6ce9e1d39bd42167c8c395fb9e62329f782a5e4e146ba715f | docs/thefinalglance/H-plan/H-17_trial_ledger_keystone.md |
| DEEP-H-006 | numbered/index original | 169 | 13906 | b4fe6353a73853013e6b33e73dab7ac538aaff47544d603fbf1fdcd81b66a705 | docs/thefinalglance/H-plan/H-23_same_seed_rerun_hash_equality.md |
| DEEP-H-006 | numbered/index original | 182 | 16052 | 0e56d5c00322a65b4824767c75f862226b9e99503cf4a207b460783b3c38e340 | docs/thefinalglance/H-plan/H-28_reward_tampering_harness_frozen.md |
| DEEP-H-006 | numbered/index original | 176 | 15510 | ba6415c663e578116a81b2185e67de524e89231bfdbb0e794ec0db75d601af39 | docs/thefinalglance/H-plan/H-29_drift_state_machine.md |
| DEEP-H-006 | numbered/index original | 81 | 20040 | d19c6343e8a5691c828bee62302d29cf54eb4c18fd8f48d18f8b2e748ba955be | docs/thefinalglance/H-plan/H-30_staleness_intent_drift_metric.md |
| DEEP-H-006 | numbered/index original | 71 | 19556 | 0be01fb08340a04e15d98a15dc5bfbe4d5eda64484731ae079d91657516bd296 | docs/thefinalglance/H-plan/H-36_completion_needs_input_notifications.md |
| DEEP-H-006 | numbered/index original | 48 | 11347 | 5bb488b582e5f87bdcaab6594fb45d22515e28f51988e0ddce219efb5c4fbee9 | docs/thefinalglance/H-plan/H-51_sleeptime_handoff_digest_autodetect.md |

### DEEP-H-007

Family **H**; totals: **10 files**, **1494 logical lines**, **158951 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-H-007 | numbered/index original | 201 | 18517 | 834cc3b5cc584b29a9e747d489da3336494a8f7c13ce0c326816041af5d2e5dc | docs/thefinalglance/H-plan/H-13_model_drives_canvas_ops.md |
| DEEP-H-007 | numbered/index original | 203 | 18956 | 6c6e8e22970541cac52140347d6c6035b2374ca0f16487841f18a90a7f83047e | docs/thefinalglance/H-plan/H-15_spawn_view_reply_channel.md |
| DEEP-H-007 | numbered/index original | 162 | 14017 | f7f2d976fa2f23b060b932e0100f85257ccf50b2a9b5584b42c18d6bfd4b6078 | docs/thefinalglance/H-plan/H-16_retire_orbit.md |
| DEEP-H-007 | numbered/index original | 180 | 14750 | 3bd3c29c9ba41a696e95f2e80e40fdf516a17f19972b48971b290241d5f4b436 | docs/thefinalglance/H-plan/H-18_deflated_sharpe_pbo_gate.md |
| DEEP-H-007 | numbered/index original | 171 | 14403 | 107e3787bf1c01150473ead4f93249ea4d5e0c198acee2d66d8c3297a07f0591 | docs/thefinalglance/H-plan/H-26_shadow_exec_financial_invariant_mr.md |
| DEEP-H-007 | numbered/index original | 50 | 15317 | cfd96e0d3321debd17d8d1d07646021194c85b74f52012105567399b81bc367c | docs/thefinalglance/H-plan/H-34_mcp_defer_loading_tool_search.md |
| DEEP-H-007 | numbered/index original | 67 | 16629 | f5e2d5afe69f7d5536e7593805ebc347e40996f070a9724f8ec767509dc601a3 | docs/thefinalglance/H-plan/H-37_plan_approval_gate.md |
| DEEP-H-007 | numbered/index original | 191 | 16195 | 40141e881577893e18fe79f218fd68eb889d6bc6d434dfc0a33bf63724644871 | docs/thefinalglance/H-plan/H-39_abstention_aware_promotion_eval.md |
| DEEP-H-007 | numbered/index original | 184 | 16112 | 5d9202e4bee8d6526e8bf4129cff1f3b6d57009f728555ff993fad491a5fc5dd | docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md |
| DEEP-H-007 | numbered/index original | 85 | 14055 | 7a4aca7123e61b4aef96ee0f6455b21a653f6920bcccc3914c0bd9b6bef23d96 | docs/thefinalglance/H-plan/H-50_regime_aware_context_management.md |

### DEEP-I-001

Family **I**; totals: **10 files**, **1169 logical lines**, **99202 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-I-001 | auxiliary/grounding/fixture | 207 | 19331 | ce1f436fb27ffce089a287819303292ebdebaface6ade9a19c1eb3a84c7fc4bc | docs/thefinalglance/I-plan/_grounding/I-GROUNDING-MAP.md |
| DEEP-I-001 | numbered/index original | 132 | 9754 | ccab90fbbb09287db8a48034d0456cec1db3772a4ead607ed1a5dddb84662039 | docs/thefinalglance/I-plan/I-8_aletheia_voluntary_abort.md |
| DEEP-I-001 | numbered/index original | 96 | 8373 | ea8f8892b102b2d63a660258f6cdf7c34be9bd487bc7febe9eaebfb9e4123f7f | docs/thefinalglance/I-plan/I-13_memory_guard_dual_serialization.md |
| DEEP-I-001 | numbered/index original | 91 | 7905 | 37ef02f48c9516f096a6be9d9c7fe02be2ea6430e709ec8f1bc908446119bcee | docs/thefinalglance/I-plan/I-15_semantic_cache_blake3_fastpath.md |
| DEEP-I-001 | numbered/index original | 107 | 9384 | 2860acc45deb41191eaaea364e1660426133a1300e406bab0dae9ce794866a9d | docs/thefinalglance/I-plan/I-18_upip_chain_sigstore_aibom.md |
| DEEP-I-001 | numbered/index original | 109 | 10168 | db9dd2b95a380588951ce8770f808906ae5ebb35e2858a7c215a211d7dc94ebf | docs/thefinalglance/I-plan/I-23_unified_conformal_calibration.md |
| DEEP-I-001 | numbered/index original | 126 | 11150 | 1002977a47734e94adf737f305eb47547dfb08e22e7b1efa6de5e3dccb02a7b6 | docs/thefinalglance/I-plan/I-28_dag_credit_assignment.md |
| DEEP-I-001 | numbered/index original | 115 | 9143 | 1da497b94fd36a98295b97884af93011f71c69ac3e9a25b0ee0bb2480e6b698e | docs/thefinalglance/I-plan/I-36_cross_model_prompt_compilation.md |
| DEEP-I-001 | numbered/index original | 118 | 9493 | 32a8ee7339c40fbad52c46beb8bcf56a0dc8aa4a8b8882cd160e5b8b4dbd4dd8 | docs/thefinalglance/I-plan/I-37_ears_requirement_objects.md |
| DEEP-I-001 | numbered/index original | 68 | 4501 | ee8dd28f1b2e37953a369a71300f2314be868da7a05938a6928d29ec38711f95 | docs/thefinalglance/I-plan/I-48_prompts_as_artifacts.md |

### DEEP-I-002

Family **I**; totals: **7 files**, **1169 logical lines**, **99250 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-I-002 | auxiliary/grounding/fixture | 96 | 7098 | 852ff3b2dddf6faa1417d85fb4ca32c807ad9864455ac44d09fd4fe8c6ac1048 | docs/thefinalglance/I-plan/_grounding/I-STEAL-GROUNDING-MAP.md |
| DEEP-I-002 | numbered/index original | 603 | 55315 | 43b35d0a46695824b1ace99b26d123ee341f6e3e2482f2ce788455b234e25093 | docs/thefinalglance/I-plan/I-0_INDEX.md |
| DEEP-I-002 | numbered/index original | 115 | 8303 | 6c1f799dc789166ae317b010c90bde3b638790da3d7226377db51e973551f669 | docs/thefinalglance/I-plan/I-5_pristine_context_injector.md |
| DEEP-I-002 | numbered/index original | 101 | 9012 | ea92c63822a8f99dddd92229ee9ce6e0740bdf38c9d412ea80c42cbbdb10d3b4 | docs/thefinalglance/I-plan/I-10_prefix_invariance_ttl_tiering.md |
| DEEP-I-002 | numbered/index original | 109 | 9693 | ccd141807a2f4a2e9e06a14cca0f1b9a98fc354ad45607d557a14184eb81906e | docs/thefinalglance/I-plan/I-29_diffusion_draft_lane.md |
| DEEP-I-002 | numbered/index original | 70 | 4699 | 76f28dd16ae1a1afd57adb4b4a510dd2340c2bc1ff0ded8024743781ee75c892 | docs/thefinalglance/I-plan/I-46_bash_interceptor_redirect.md |
| DEEP-I-002 | numbered/index original | 75 | 5130 | 2b0994903bad66b9191a261e10b043ca44ab5f10894e16f969fd757894f8562d | docs/thefinalglance/I-plan/I-54_structural_ast_edit_tool.md |

### DEEP-I-003

Family **I**; totals: **10 files**, **1157 logical lines**, **89623 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-I-003 | numbered/index original | 117 | 8213 | c1dcb05555ab190ce87f7daad419eea07a79cf80199d2abb25c50b060d7b44ef | docs/thefinalglance/I-plan/I-1_directive_supervisor_enforcement_plane.md |
| DEEP-I-003 | numbered/index original | 193 | 15323 | 9967be0ae8db28d7564b96b4910b02a65889f5e651524f613c21744a36f4ea2d | docs/thefinalglance/I-plan/I-4_evidence_ranked_search_ladder.md |
| DEEP-I-003 | numbered/index original | 136 | 9856 | 33f08e0127711d345b45fcda45b32dd5d0c98cc1480fcf9c8ec446a066325d2f | docs/thefinalglance/I-plan/I-9_error_amplification_telemetry.md |
| DEEP-I-003 | numbered/index original | 120 | 11650 | c5eeb6d26e1822656af736488960c7e02fb1fac6728bc557539832aaca17c24e | docs/thefinalglance/I-plan/I-20_budgeted_mutation_scheduler.md |
| DEEP-I-003 | numbered/index original | 97 | 7468 | c97b04a168be835f81175ea9972796dd86205e5a11b51ae40900dec4ed70e6e4 | docs/thefinalglance/I-plan/I-32_structural_role_scoping.md |
| DEEP-I-003 | numbered/index original | 106 | 7960 | 9284444b3e35cc869a8a183b20547eccfe4c7be42c35ea18724249db627d1bb0 | docs/thefinalglance/I-plan/I-33_mechanical_decision_governance.md |
| DEEP-I-003 | numbered/index original | 112 | 8484 | 078dee380d1da622cc80fbf155fa2894c31e493641d2ef5c4d5053a30d05eed7 | docs/thefinalglance/I-plan/I-38_qdelta_statistical_floor.md |
| DEEP-I-003 | numbered/index original | 129 | 10568 | 2679407d264ef6dc8976d422826a856f1ab0a05f4f6cada461944cef6e22e212 | docs/thefinalglance/I-plan/I-41_spec_time_critique_lenses.md |
| DEEP-I-003 | numbered/index original | 75 | 5127 | c60669807533228199827b1e5a9edc1dc9133ae18fa161f8e2ac63f5146c9a4b | docs/thefinalglance/I-plan/I-50_subscription_auth_broker.md |
| DEEP-I-003 | numbered/index original | 72 | 4974 | 0f62ced3fcb20fea3fddba8d4f75fe2235c52345bb076584fe77ca8079def6ec | docs/thefinalglance/I-plan/I-52_checkpoint_rewind_tool.md |

### DEEP-I-004

Family **I**; totals: **10 files**, **1155 logical lines**, **92980 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-I-004 | numbered/index original | 149 | 11710 | fdaa9fe71d80559f57d01ef7d3ede36e00b0588d3fa301c129a51410db261474 | docs/thefinalglance/I-plan/I-2_cow_workspace_isolation.md |
| DEEP-I-004 | numbered/index original | 129 | 9829 | 8bad3f027824c9a46efcae6b28d188dc648fb929b682afdcb399b957611abc5f | docs/thefinalglance/I-plan/I-3_ssb_throughput_atomic_saves.md |
| DEEP-I-004 | numbered/index original | 99 | 8318 | 1e1cc7c674fe4856276377b4c963285d753a8e05d26f23c5b2b6474a3b59fc0e | docs/thefinalglance/I-plan/I-12_evidence_gated_playbook_curator.md |
| DEEP-I-004 | numbered/index original | 113 | 11881 | 9fd839ddafb7566288f9f399d4d73264d4e0fc9e43fccac23f67c6141e5bdfc9 | docs/thefinalglance/I-plan/I-21_sprint_contracts_ralph_loop.md |
| DEEP-I-004 | numbered/index original | 93 | 7288 | df9b3e9b1bf8c7c447a68b034b5e7aa9b516e59c02def240b4f9d9c175fa5f48 | docs/thefinalglance/I-plan/I-27_budgeted_convergence_instrumentation.md |
| DEEP-I-004 | numbered/index original | 151 | 11580 | 67b08387af37ab07ec0b5072c2a1fc84ddd8c381f4977b98cb91d5a14b6e9702 | docs/thefinalglance/I-plan/I-34_crane_hybrid_grammar_pinning.md |
| DEEP-I-004 | numbered/index original | 108 | 8542 | 9af1b88d39bba20f0ac5dfcc989d9e330284495e03ef9a83b2a6790876843a3a | docs/thefinalglance/I-plan/I-35_temporal_tool_sequence_enforcement.md |
| DEEP-I-004 | numbered/index original | 123 | 9379 | 8924710544fb9f2891a1677ed079ba9e09d93c647375ec536ec6d421a6a107d3 | docs/thefinalglance/I-plan/I-40_trust_layer_wrapper_rra_spec.md |
| DEEP-I-004 | numbered/index original | 118 | 9511 | 3474fa466b27fd10239e0336b81b3c123927c3ad4a32e38ca35d9cdef4f914f1 | docs/thefinalglance/I-plan/I-42_critique_only_profile.md |
| DEEP-I-004 | numbered/index original | 72 | 4942 | 8259c203eb17be3e60f4ceef392a3ed9d598ce4d996c47883a06e06d89afc295 | docs/thefinalglance/I-plan/I-45_objective_hash_sealed_receipts.md |

### DEEP-I-005

Family **I**; totals: **10 files**, **1169 logical lines**, **95848 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-I-005 | numbered/index original | 112 | 8103 | 16e098e23c85b71ef887c3067f6c7a5ccdf31d366c81444b8a9495296c961e67 | docs/thefinalglance/I-plan/I-6_anchor_pane_slot.md |
| DEEP-I-005 | numbered/index original | 123 | 8922 | 0571a2d979e4559362d05d5a2988acc59b4720673bdddc4e838d03c4381f6d69 | docs/thefinalglance/I-plan/I-7_cascading_escalation_offline_batch.md |
| DEEP-I-005 | numbered/index original | 91 | 7863 | cf20fe5c2f23381769b192e8f8f16692ef494889ea812eab4f6037e46f8c165c | docs/thefinalglance/I-plan/I-11_pabu_belief_state_handoff.md |
| DEEP-I-005 | numbered/index original | 102 | 9064 | 1b1861d46cfc8790c793f4eb767c15ba5d4122be88f65c93eef6b72c66090f2c | docs/thefinalglance/I-plan/I-14_deterministic_context_compression.md |
| DEEP-I-005 | numbered/index original | 117 | 11279 | 6c94548b4d7912ca8df9f97d19e583cf17e8fec1f821f5c93a0f95811ba4c070 | docs/thefinalglance/I-plan/I-22_progress_shaped_rewards.md |
| DEEP-I-005 | numbered/index original | 182 | 13541 | 4d528329f9c628955e994ad3235bcd956e2c700a9170b0e70c48e31e14dbc8c6 | docs/thefinalglance/I-plan/I-25_ledger_trained_router.md |
| DEEP-I-005 | numbered/index original | 147 | 14227 | fff87039bfe586e07bea2e8af938c75cebf02b5d54c643c6373165d9b149503d | docs/thefinalglance/I-plan/I-31_grpo_flywheel.md |
| DEEP-I-005 | numbered/index original | 99 | 7834 | 911f9ea47ed576d28ada35919a9628905682e29527ff8e3649de9b77f1006a34 | docs/thefinalglance/I-plan/I-39_plan_state_from_repo.md |
| DEEP-I-005 | numbered/index original | 127 | 10501 | 384eb267ae4495ebd1dd9e3a181b9fb9745cfc6b17346c5562ccf21d40c2256b | docs/thefinalglance/I-plan/I-44_zero_retention_export.md |
| DEEP-I-005 | numbered/index original | 69 | 4514 | 63e7cddae7af1c5f6f48f20bcebae3803956ae9b1f96f4235f6746e84587a449 | docs/thefinalglance/I-plan/I-51_model_capability_catalog.md |

### DEEP-I-006

Family **I**; totals: **10 files**, **1162 logical lines**, **95462 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-I-006 | numbered/index original | 126 | 7458 | c8b6f7a608d4197f045c23b94805e9913f4dcfdf8a3739a03be948354026509e | docs/thefinalglance/I-plan/I-16_kv_cache_pruning_quant.md |
| DEEP-I-006 | numbered/index original | 116 | 10706 | 078783e757b8246068f36405735f72640d971b3dd6d2c7f4d0a86119d594cb2c | docs/thefinalglance/I-plan/I-17_tri_layer_evaluation_stack.md |
| DEEP-I-006 | numbered/index original | 104 | 9294 | 341f1b798a54b24f2e684ea1894d7bec2aba36c8857a245d9d9a25469404494c | docs/thefinalglance/I-plan/I-19_tdd_enforcer_preflight.md |
| DEEP-I-006 | numbered/index original | 122 | 12191 | 56514d820a02e8d10d50c7e3af84b97802ed3ec4e90188153eb1835e63e8c4b9 | docs/thefinalglance/I-plan/I-24_isomorphic_canary_antigaming.md |
| DEEP-I-006 | numbered/index original | 98 | 8284 | 04e955fcd4e741584a8903d8f37b947d3a23a3c675df3280f2667d96c7dad929 | docs/thefinalglance/I-plan/I-26_dynamic_thinking_budgets.md |
| DEEP-I-006 | numbered/index original | 200 | 18184 | 5f295045bec8cf4303304f9d059735b5439481201b246b0fd22d799c7ee8e2a2 | docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md |
| DEEP-I-006 | numbered/index original | 132 | 11085 | 15c3ca9aa6bb71561208b8cf9fa22f0f205d5d64447ca6dc9b8d0c3726579048 | docs/thefinalglance/I-plan/I-43_claim_scoper_kill_gate.md |
| DEEP-I-006 | numbered/index original | 80 | 5381 | 2039b01baa9e649e99857b52136bf03f3f7258a2adedbec1bd3309af65137cfa | docs/thefinalglance/I-plan/I-47_verifier_input_hardening.md |
| DEEP-I-006 | numbered/index original | 112 | 8179 | dfbfcecb6b7ecfe29e19288c5d397664effd62b1fe8560e113a2f59425e23647 | docs/thefinalglance/I-plan/I-49_bedrock_azure_direct_lanes.md |
| DEEP-I-006 | numbered/index original | 72 | 4700 | d7206d6e212dc811e543e3faea3a53b415ee2683ebfea17ce675df6e4f78be8f | docs/thefinalglance/I-plan/I-53_recipe_runner_tool_pack.md |

### DEEP-J-001

Family **J**; totals: **10 files**, **1280 logical lines**, **158205 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-001 | auxiliary/grounding/fixture | 116 | 11547 | 50e4be89e620d49b183aa1c4eb7328f9cebba96bcc7f4b11907f05a45458eddc | docs/thefinalglance/J-plan/_grounding/J-DOCTRINE-PROPAGATION-GROUNDING-MAP.md |
| DEEP-J-001 | numbered/index original | 411 | 32213 | 05dfe76a6cec744d7997a950c66e1a8de5989cddb66961054a6fd90507350f68 | docs/thefinalglance/J-plan/J-28_token_ledger_usage_unification.md |
| DEEP-J-001 | numbered/index original | 52 | 8901 | 48ec1348240d581bf9c8eec04435737db868414056d5eb06a8abced4f1e22d4a | docs/thefinalglance/J-plan/J-40_research_modes_extractive_compression.md |
| DEEP-J-001 | numbered/index original | 64 | 13503 | 7acf286a7333674f3fd8aef4c09263595df2b74c6e525d6f3f98389009f5073b | docs/thefinalglance/J-plan/J-43_doctrine_propagation_keystone.md |
| DEEP-J-001 | numbered/index original | 74 | 10089 | dea371a8d907eaf51862bbb4abe4a5793450c1f63f36c132fba1641495db04a3 | docs/thefinalglance/J-plan/J-66_ssb_temporal_monitor_engine.md |
| DEEP-J-001 | numbered/index original | 71 | 11022 | 5589a2ff109efac990569f6d21a4448ea1c2d199dc4650543a4c9ee12352e782 | docs/thefinalglance/J-plan/J-76_context_rot_calibration_riders.md |
| DEEP-J-001 | numbered/index original | 85 | 11883 | 65a33ecf1283cfea85f1cdc71c0d5d829eca3ee273b55d77b709f58a763ee925 | docs/thefinalglance/J-plan/J-90_crosslang_dynamic_blast_cone_composition_gate.md |
| DEEP-J-001 | numbered/index original | 124 | 19571 | b635caee013ece299a9104eaa7f909e198d62cd3db7810f269b422ecbf8df138 | docs/thefinalglance/J-plan/J-99_time_as_context_plane.md |
| DEEP-J-001 | numbered/index original | 130 | 22538 | 1ba7068df85ce878ba99579ef7b0b876621ee8d747d9a5b1f4b320ad2f3a40b1 | docs/thefinalglance/J-plan/J-105_memory_roi_attribution_curation.md |
| DEEP-J-001 | numbered/index original | 153 | 16938 | d89c578e2c4c32b291d47a40d1e63f015039036474db16b411499de02ae4b749 | docs/thefinalglance/J-plan/J-112_spoken_render_narration_dispatcher.md |

### DEEP-J-002

Family **J**; totals: **10 files**, **1224 logical lines**, **152552 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-002 | auxiliary/grounding/fixture | 167 | 15553 | 93be94ae51e434ad135f4350b34d5150b8eadd4232764607fda238fa031a629e | docs/thefinalglance/J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md |
| DEEP-J-002 | numbered/index original | 68 | 9572 | 6e81b60bdb1d0538d530ed16ea0a753fca1b17fdd56aa2020c3acdd6159e1dae | docs/thefinalglance/J-plan/J-22_journal_payload_schema_lock.md |
| DEEP-J-002 | numbered/index original | 128 | 8968 | 4234c551973f8bd86560ed6f39ec2869ac9916da6e05d49c7c4dddf8912b257c | docs/thefinalglance/J-plan/J-27_harness_protected_blocks_webcache_dispatch.md |
| DEEP-J-002 | numbered/index original | 266 | 20212 | 15743fc53d887d83602458878467f32199b0284095129d5559bfe4ca69a55717 | docs/thefinalglance/J-plan/J-36_freeze_head_local_prefix_kv_reuse.md |
| DEEP-J-002 | numbered/index original | 59 | 13429 | 2c04f6012404e6c05bec7bd4685d3d3530d22b837dc7cc60b664cc3d632c20b8 | docs/thefinalglance/J-plan/J-44_reasoning_method_artifact_shape.md |
| DEEP-J-002 | numbered/index original | 136 | 25682 | 4e5abfb374146890e421759970c98afb23580301aeb792972c219e1cc38f75be | docs/thefinalglance/J-plan/J-52_returntime_perEdit_enforcement_hooks.md |
| DEEP-J-002 | numbered/index original | 91 | 13486 | 02d7cfafeb2bfc6c3dc5cf9fe780ab524692b319824dac1c3e4b953c9ac259cc | docs/thefinalglance/J-plan/J-93_lacuna_tier2_composition_fills.md |
| DEEP-J-002 | numbered/index original | 73 | 10880 | 2f8dac276d70931c1f3b6a1d985db1c9588578a6a9314beddded75b219153c50 | docs/thefinalglance/J-plan/J-94_lacuna_tier3_honesty_thin_measure_first.md |
| DEEP-J-002 | numbered/index original | 122 | 18369 | ecc9f455e5d8e1780cfdd7149fcd85b9537584bb9b301e8fb8af96319a878196 | docs/thefinalglance/J-plan/J-106_output_metacommentary_enforcement.md |
| DEEP-J-002 | numbered/index original | 114 | 16401 | 2fe0a4e11d65d8e6c36ebf4a14e82234c6ac5cde5238d9889b1c2aaac8055ffc | docs/thefinalglance/J-plan/J-109_learned_artifact_admission_hardening.md |

### DEEP-J-003

Family **J**; totals: **10 files**, **1229 logical lines**, **134050 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-003 | auxiliary/grounding/fixture | 100 | 7977 | 98910181c58542183e5619761e63ba9f7a4c56fffc365f6e2782434969d308f7 | docs/thefinalglance/J-plan/_grounding/J-HARNESS-GROUNDING-MAP.md |
| DEEP-J-003 | numbered/index original | 230 | 16685 | a4c241e9a9946b185a218220d450d257dbac8d35638d04382883353dd12f89cd | docs/thefinalglance/J-plan/J-8_vlm_perception_research_upgrade.md |
| DEEP-J-003 | numbered/index original | 229 | 17490 | 1b516ccc21c0181f06337677ffef148b99f2c3d02ac2ec6f49daebe519f46e38 | docs/thefinalglance/J-plan/J-10_insight_backprop_tree_memory.md |
| DEEP-J-003 | numbered/index original | 125 | 9341 | 76408a6e4ec22c7fab76dbaf6b4df545a6c5b8e9b9ca8b0229d1c87fbb6f139a | docs/thefinalglance/J-plan/J-33_mineru_document_parsing_lane.md |
| DEEP-J-003 | numbered/index original | 71 | 16351 | 1c3073dbb831d8e809844d07c349801ebbd5875daa7f0d630e14eab78e1e4fdd | docs/thefinalglance/J-plan/J-35_intent_control_layer.md |
| DEEP-J-003 | numbered/index original | 66 | 7764 | 6b8cd27ea659dc10f932751cc88e77833be45655b373154be32ec65e09edca0c | docs/thefinalglance/J-plan/J-75_rrf_ablation_harness.md |
| DEEP-J-003 | numbered/index original | 79 | 12417 | 3ec78ac4c80c068d60d18e01153185a18251cd44eb8fa27da81df535fc7c6907 | docs/thefinalglance/J-plan/J-79_grill_signal_interlace.md |
| DEEP-J-003 | numbered/index original | 76 | 11669 | 20377cde67d5ebaf3facefc3effc439c9686679ed926c69efd6c6d5613eade62 | docs/thefinalglance/J-plan/J-87_journal_completeness_substrate.md |
| DEEP-J-003 | numbered/index original | 135 | 17381 | 8369ee3a2091f0c723af3c005a963bc587a5c77035a784441796d8be8818af04 | docs/thefinalglance/J-plan/J-96_verified_case_graph_core.md |
| DEEP-J-003 | numbered/index original | 118 | 16975 | f8a354e89efa1bcf777d5c1dc82c096209ec106488af9e2bb2ba321f64ad4c4c | docs/thefinalglance/J-plan/J-107_longcot_recursive_scaffold_bench.md |

### DEEP-J-004

Family **J**; totals: **10 files**, **1222 logical lines**, **149575 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-004 | auxiliary/grounding/fixture | 244 | 19451 | 95a20e1459aae3814c6c8d7c5b015b2383750c90411daa8c6569e4f47a540dc7 | docs/thefinalglance/J-plan/_grounding/J-LACUNA-FILL-GROUNDING-MAP.md |
| DEEP-J-004 | auxiliary/grounding/fixture | 113 | 10081 | df1ed8253fad499498bb0b46798bb583d4f611a05fcaf68e708f1719de193926 | docs/thefinalglance/J-plan/_grounding/J-SDL-CONTEXT-MEMORY-GROUNDING-MAP.md |
| DEEP-J-004 | numbered/index original | 175 | 11609 | 2a1d82273c652ce413adc530ee4be6d1c9c0bc316f4669f126df031b9d21d6eb | docs/thefinalglance/J-plan/J-1_perception_gateway.md |
| DEEP-J-004 | numbered/index original | 72 | 9170 | 019bdb5ac8c1bbc951f7865bf910334ed560d831cae5a06936f771e7ada843d3 | docs/thefinalglance/J-plan/J-12_runtime_intervention_harness.md |
| DEEP-J-004 | numbered/index original | 57 | 11695 | 07a85bef7fbd919bf14fa7b19b0312c327a6105abc48879cd633111acd905dc5 | docs/thefinalglance/J-plan/J-47_program_falsifier_per_axis_discipline.md |
| DEEP-J-004 | numbered/index original | 153 | 28604 | 8d863f7dc8996166061ab048a7b4ef0edf1c6283aa3fda77d500f42d2b0b71c3 | docs/thefinalglance/J-plan/J-64_acp_transport_for_agent_bridges.md |
| DEEP-J-004 | numbered/index original | 92 | 14163 | fc930619d3c290dd7cc03752814a80949e51e7c1dff74568bb86726b7dd00102 | docs/thefinalglance/J-plan/J-65_power_of_ten_generated_code_gate.md |
| DEEP-J-004 | numbered/index original | 69 | 9052 | da0cbdbfc55a033d0ea5919777be07abbfc177f66fa8c5d0f75edf9f601d9752 | docs/thefinalglance/J-plan/J-74_graph_as_retrieval_channel.md |
| DEEP-J-004 | numbered/index original | 127 | 18492 | 2d53f1467843b0f20a86dab4a31e9f7ec9132fdb29fe152e91cdee24255a0709 | docs/thefinalglance/J-plan/J-97_case_reuse_gated_promotion_ladder.md |
| DEEP-J-004 | numbered/index original | 120 | 17258 | 6517d64f8d98692586671250ed0f310f5d47bd6d2430e2dc03de8d82275e2a4c | docs/thefinalglance/J-plan/J-104_live_index_overlay_idle_checkpoint.md |

### DEEP-J-005

Family **J**; totals: **10 files**, **1228 logical lines**, **149610 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-005 | auxiliary/grounding/fixture | 82 | 7001 | b95f0969ee320db99c0d890ed9a15db9184d207d52b9262918778ce2b3f9c4a1 | docs/thefinalglance/J-plan/_grounding/J-NASA-GATES-GROUNDING-MAP.md |
| DEEP-J-005 | auxiliary/grounding/fixture | 123 | 11356 | c582305db03a7a219f3326b7f7e5e516d44a65f3cbd9f1bace681a795b229335 | docs/thefinalglance/J-plan/_grounding/J-RLM-GROUNDING-MAP.md |
| DEEP-J-005 | numbered/index original | 76 | 11507 | 9fce1c93f78b935be1b54fc865ae75f61fde8b78ed87c36a0b9fcb7d2c7aa873 | docs/thefinalglance/J-plan/J-21_memory_semantic_validation_poisoning.md |
| DEEP-J-005 | numbered/index original | 72 | 11224 | 32e1b64cf55987d2709626648561af3eb0b0b3e3ac155482f776ac509dafe446 | docs/thefinalglance/J-plan/J-24_nonprompt_steering_context_frugal_tools.md |
| DEEP-J-005 | numbered/index original | 242 | 17760 | 588642b09e1d311aa9938a08dc2898443c7bfbce7557377791efdad86d051806 | docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md |
| DEEP-J-005 | numbered/index original | 184 | 15967 | 08641ab134f7fdbd706f6a3d7fe9516c227d198d10f026a64f21485158ec9f8a | docs/thefinalglance/J-plan/J-61_canvas_redirect_loop.md |
| DEEP-J-005 | numbered/index original | 129 | 24172 | 499e523b2427ccc1df9aff1c006d134d1780e28e499e472c78501d48e7d715e8 | docs/thefinalglance/J-plan/J-62_env_constraint_skill_cards_prewrite_gate.md |
| DEEP-J-005 | numbered/index original | 64 | 8087 | 4d74609faccba0c2b46d119af96c1857fec3fb9da372c33ef971bed0fc1dba67 | docs/thefinalglance/J-plan/J-73_retrieval_abstention_gate.md |
| DEEP-J-005 | numbered/index original | 117 | 18276 | 610d965ec0aa4f3a9f14fe7187b89236c3985269bf1694a7d9dd06fdefcf9c5a | docs/thefinalglance/J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md |
| DEEP-J-005 | numbered/index original | 139 | 24260 | 95e4140231e2fc54c8969ed6d1010192c09b6792a5d0452395f9b07f10b0ea95 | docs/thefinalglance/J-plan/J-101_transformation_surviving_taint_binding.md |

### DEEP-J-006

Family **J**; totals: **10 files**, **1223 logical lines**, **149753 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-006 | auxiliary/grounding/fixture | 170 | 13623 | ffeab5ab6d32a1b22f820d822b27928e2c32b258b43fb61cfcb3c76c0c2d2e15 | docs/thefinalglance/J-plan/_grounding/J-PERCEPTION-GROUNDING-MAP.md |
| DEEP-J-006 | numbered/index original | 252 | 19833 | a92b5a3aa97e0927aaa41e6b6f116cb53eb4471cc2398e5bf39894aae4b4d899 | docs/thefinalglance/J-plan/J-11_arbor_steal_intake_closeout.md |
| DEEP-J-006 | numbered/index original | 69 | 8181 | b06877268151b696e3991341488089503919973cb300d17229c18ec148028f2c | docs/thefinalglance/J-plan/J-13_local_lane_token_determinism.md |
| DEEP-J-006 | numbered/index original | 58 | 7173 | f764dbb3f13b7c4137f3c451aa5a53f41274f904294c76286fe9018a99fce4e3 | docs/thefinalglance/J-plan/J-15_determinism_steal_amendments.md |
| DEEP-J-006 | numbered/index original | 73 | 10771 | 8bc686eaa2a1a829b97a8fad03d30e2268b644c37b14468cf1c4794a8c961b3c | docs/thefinalglance/J-plan/J-17_plan_level_compositional_safety.md |
| DEEP-J-006 | numbered/index original | 90 | 16227 | d64fc105d2f13d4661258375d150a47e683d53dc626e623c8894f5524b857287 | docs/thefinalglance/J-plan/J-77_perception_faithfulness_trust_spine.md |
| DEEP-J-006 | numbered/index original | 121 | 19114 | 523bc77737116f97b33c7d9c9bcc4977b88106a44d86024be430bc0373f42dd9 | docs/thefinalglance/J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md |
| DEEP-J-006 | numbered/index original | 114 | 16451 | 0069c60ec5bab56eeca9887b39ae43ba4066af67390c8cbd7a73d2a9006117c5 | docs/thefinalglance/J-plan/J-98_canvas_case_memory_altitude.md |
| DEEP-J-006 | numbered/index original | 127 | 22540 | 37b04877d6e92c0afc41543fe046249a24e35ac54b47b679e96a2f752d9de940 | docs/thefinalglance/J-plan/J-102_config_ingress_admission_gate.md |
| DEEP-J-006 | numbered/index original | 149 | 15840 | 900bfa925f6c7711cada1160227e74d0975effedd4bc90f4c5b1d0fbaf056743 | docs/thefinalglance/J-plan/J-111_voice_ingress_spoken_confirm.md |

### DEEP-J-007

Family **J**; totals: **2 files**, **2003 logical lines**, **222633 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-007 | auxiliary/grounding/fixture | 49 | 4654 | 134e2db9ab1f112280afcdadf076a650c922473ef1eb21716cc48564eb2b4bec | docs/thefinalglance/J-plan/_grounding/J-VOICE-GROUNDING-MAP.md |
| DEEP-J-007 | numbered/index original | 1954 | 217979 | 6ddf9e63c98e65a0bae520b32d99197fa32457366373951d2aea28115d35f0c4 | docs/thefinalglance/J-plan/J-0_INDEX.md |

### DEEP-J-008

Family **J**; totals: **10 files**, **1219 logical lines**, **151045 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-008 | numbered/index original | 215 | 14684 | 7d166b95f215267fe093e58a6d117668abcce96b4028b020bc0bb0383991265d | docs/thefinalglance/J-plan/J-2_elai_mcp_control_plane.md |
| DEEP-J-008 | numbered/index original | 78 | 11213 | 4a849f9126a91b0b7f0cc3e9b6ebb18cc50710efcb7eab117e9b596b6c09f846 | docs/thefinalglance/J-plan/J-16_blast_radius_regression_gate.md |
| DEEP-J-008 | numbered/index original | 54 | 9714 | 1824daf87a060c94d5723e68348ea0d5d01a61115672aea38c371091c696e035 | docs/thefinalglance/J-plan/J-41_cheap_grounding_stack.md |
| DEEP-J-008 | numbered/index original | 237 | 31996 | 9857f71108b82ecbcf85bb81d2d5d66b60c7ecd0965d6e946ef82424d68ea9f8 | docs/thefinalglance/J-plan/J-54_free_api_rotation_pool.md |
| DEEP-J-008 | numbered/index original | 135 | 18023 | 90b33af25158f2a1f3b57f909b1f3328b0229a71d1c4f63e3bd7d4d26823cb2c | docs/thefinalglance/J-plan/J-55_model_strength_modality_router.md |
| DEEP-J-008 | numbered/index original | 126 | 15539 | bdb1041b29320632cddac7f989d5ef71c3cf9924cf55cd8c74ed332bafb0f54a | docs/thefinalglance/J-plan/J-57_eval_gated_prompt_promotion.md |
| DEEP-J-008 | numbered/index original | 120 | 16920 | 00421fa3c1cc90b6657ef5f0eaa29599e62af3f10b19c5e136c6d90deef06327 | docs/thefinalglance/J-plan/J-58_sprint_contract_negotiation_loop.md |
| DEEP-J-008 | numbered/index original | 70 | 9167 | fbb21f6a4b0665843097a4a30fbd6807f19084163850094c5eb4c11e5a9fb0ba | docs/thefinalglance/J-plan/J-67_multimode_gate_actions.md |
| DEEP-J-008 | numbered/index original | 110 | 15029 | 41edb223794a6645d503c2966c8fb4977d9184acd105f6fae4ed9ab6e3fc8b21 | docs/thefinalglance/J-plan/J-82_deterministic_token_efficient_emission.md |
| DEEP-J-008 | numbered/index original | 74 | 8760 | c093406bfe92affa457cf9472a0eecb1def7ec8c76b488f4a7dbd25d3009f8d0 | docs/thefinalglance/J-plan/J-91_reexecutable_rra_track.md |

### DEEP-J-009

Family **J**; totals: **10 files**, **1247 logical lines**, **127107 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-009 | numbered/index original | 154 | 9997 | 71e62fb4da98fa8c66118da884b6b6c9c5065e5b97824e00b01807fd07089eb1 | docs/thefinalglance/J-plan/J-3_harness_undo_file_backups.md |
| DEEP-J-009 | numbered/index original | 125 | 7217 | fbc98bfad4df3d98c4d0618814af4230ed56db4e1b56d25e11dd56c0ac9f2c84 | docs/thefinalglance/J-plan/J-4_handoff_file_ledger.md |
| DEEP-J-009 | numbered/index original | 117 | 6695 | b6a2291192996a86d7cf391f1e57e64a8c7f2c9bc88407f72e076ffb2af6b926 | docs/thefinalglance/J-plan/J-6_incremental_eval_skill.md |
| DEEP-J-009 | numbered/index original | 78 | 10608 | ef778aff3e6f0ded62693bca69799ab456d3b447ff67a7270b83cd482952a524 | docs/thefinalglance/J-plan/J-19_vllm_alternative_backend.md |
| DEEP-J-009 | numbered/index original | 72 | 10973 | ab51246807f8d828dccf383d85942f57a8e08dd805a2df739634c07ed48247fd | docs/thefinalglance/J-plan/J-23_multilang_review_gate_catalog.md |
| DEEP-J-009 | numbered/index original | 353 | 25864 | 0e1dcbbaa73473b740b59ecb500991fc89033f8907ea5738392af5210fe53ec9 | docs/thefinalglance/J-plan/J-26_sia_evaluator_abi_task_packs.md |
| DEEP-J-009 | numbered/index original | 53 | 11088 | 74dab10afb9dcba59df40fb67019eeede78f9d5a0f84f32ad6a2431639203985 | docs/thefinalglance/J-plan/J-45_cross_family_advisory_calibration.md |
| DEEP-J-009 | numbered/index original | 97 | 13691 | 6aa66a2919824f1b119c35328beaae9d39f4af12f2484d2c0b10627055102735 | docs/thefinalglance/J-plan/J-51_toolcall_adherence_floor_role_gate.md |
| DEEP-J-009 | numbered/index original | 68 | 8274 | ee8205c19373335d8845d29e88dbf0de57346cb9d35742bbda95c7d893be016e | docs/thefinalglance/J-plan/J-88_runtime_dkg_edges_machine_audit_lane.md |
| DEEP-J-009 | numbered/index original | 130 | 22700 | 14a6319373b62e72d9d8ba3a11841722b505ee258461c708068a03635f98f1ec | docs/thefinalglance/J-plan/J-100_time_as_control_plane.md |

### DEEP-J-010

Family **J**; totals: **10 files**, **1220 logical lines**, **146633 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-010 | numbered/index original | 97 | 5852 | c5e374d6f3fe5b697bcad0388ea1cdd5da5b4006f08065cd549ddbbc09f15e98 | docs/thefinalglance/J-plan/J-5_litegame_harness_port.md |
| DEEP-J-010 | numbered/index original | 72 | 11202 | fe0bff13e4338dd8f38e3d7697a2cc1c19cc33041d2e7882731e04354971939a | docs/thefinalglance/J-plan/J-20_selfimprove_bandit_security_gate.md |
| DEEP-J-010 | numbered/index original | 137 | 10131 | 08ac818ce9d155070ff038018e772c52f2adf0ef6641746b2e3bee1a9ce7f3b0 | docs/thefinalglance/J-plan/J-32_elai_frontdoor_router.md |
| DEEP-J-010 | numbered/index original | 273 | 20689 | dbe28936f6efaded6feccf6f5d27f4683fa00aab46d26c0eeda4dda6b7d5558e | docs/thefinalglance/J-plan/J-37_stale_staleness_adjudication.md |
| DEEP-J-010 | numbered/index original | 53 | 8766 | 77876a76e84c246dacd346f4c60fcb5808f3550e602871a1fb20cf98cd617291 | docs/thefinalglance/J-plan/J-39_decomposition_node_admission_gate.md |
| DEEP-J-010 | numbered/index original | 122 | 14768 | 43fd45d2b59e1478585777e8eca16feaebf8d2ea0c6c95ed531c8340b4d31ab6 | docs/thefinalglance/J-plan/J-56_living_eval_corpus_governance.md |
| DEEP-J-010 | numbered/index original | 129 | 23315 | ce91e8037a1c60ca50b57e4e8c436bb5c33f2b147d5f3cc9c469ecae32ffe0b9 | docs/thefinalglance/J-plan/J-63_local_lane_claudeception.md |
| DEEP-J-010 | numbered/index original | 68 | 10117 | cde74ee318a97b2dc6574bbf76d47a8cc5ab4c99611f6b190af630361c074aa0 | docs/thefinalglance/J-plan/J-70_gate_manifest_traceability_ivv_independence.md |
| DEEP-J-010 | numbered/index original | 159 | 24677 | 419d1ef2f9fab9a9f7a3ad5680313e64781594bcc0bf8ec7550162866e533002 | docs/thefinalglance/J-plan/J-80_worktree_pool_lifecycle.md |
| DEEP-J-010 | numbered/index original | 110 | 17116 | 1d1d0ea128087e57414db6cfcac58da016d8106e3132906f0314a693bf0e17e0 | docs/thefinalglance/J-plan/J-110_abstention_timeliness_eval_supply.md |

### DEEP-J-011

Family **J**; totals: **10 files**, **1225 logical lines**, **147166 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-011 | numbered/index original | 135 | 8540 | 798be092b4525c7b4fc4cc36569350085f9890302c8c8820023fe48239b97919 | docs/thefinalglance/J-plan/J-7_ext_steal_intake_closeout.md |
| DEEP-J-011 | numbered/index original | 222 | 16207 | 6e8da5646f39b204daa54f1571b073cb36f0bc80aea8835ef22ae2039aeb8004 | docs/thefinalglance/J-plan/J-30_provider_byok_custom_endpoints.md |
| DEEP-J-011 | numbered/index original | 61 | 13618 | 5f313dcee6cdbc1a5e2bb347ebb8483f434a59ba7a2601e15b48a9e28869e33c | docs/thefinalglance/J-plan/J-42_within_step_distill_reset_reasoning.md |
| DEEP-J-011 | numbered/index original | 232 | 19923 | e4c748baeb0f130f289125f6ab5a0876eb30a1675156354fa17bbf227de0c3e9 | docs/thefinalglance/J-plan/J-48_local_serving_tier_router.md |
| DEEP-J-011 | numbered/index original | 75 | 11342 | a416badabaf6e5b2695c69a96acf17d8f5896c517c623837bc96011ad4f91780 | docs/thefinalglance/J-plan/J-72_indomain_retriever_adaptation.md |
| DEEP-J-011 | numbered/index original | 72 | 11663 | 47d53f8c1dc6895fc598476c7ea89fec082d3483d3264c6f0ff7c14d3997af77 | docs/thefinalglance/J-plan/J-78_context_access_spec.md |
| DEEP-J-011 | numbered/index original | 79 | 11197 | 56abf6becb4268ad2430304a900e38b5877d3ebdecf514cc245b948e4f8f2dfc | docs/thefinalglance/J-plan/J-89_objective_and_plan_dag_prevention_gate.md |
| DEEP-J-011 | numbered/index original | 104 | 12817 | 64401c209e23a7236590486fbc70cb91733f23d6df20109cd0e0110d6ddea4be | docs/thefinalglance/J-plan/J-92_orchestrator_fsm_dispatch_ordering.md |
| DEEP-J-011 | numbered/index original | 120 | 18203 | 7490b518892c6855a34e6315e02f7eddc398365751caac58a5b5f1aade2a7db3 | docs/thefinalglance/J-plan/J-95_context_activation_wires.md |
| DEEP-J-011 | numbered/index original | 125 | 23656 | 133d235652d0b88f393956ac3f5b264c16715970bec5de5c50a7321a26cdf084 | docs/thefinalglance/J-plan/J-108_retrieval_navigator_depth_operator.md |

### DEEP-J-012

Family **J**; totals: **10 files**, **1226 logical lines**, **151391 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-012 | numbered/index original | 240 | 16822 | a7eb946cef30e208f2c3a039d9b5126910c87d75de1fb7e710d8d6fea465bf8c | docs/thefinalglance/J-plan/J-9_hardware_detection_model_catalog.md |
| DEEP-J-012 | numbered/index original | 70 | 12583 | edc4539355c4d97c9e2b2ff56b707976c063ad06c3c3467c405cd9479a1b149e | docs/thefinalglance/J-plan/J-18_p1_retrieval_hardening.md |
| DEEP-J-012 | numbered/index original | 186 | 13600 | a124503bf301609fe3a97f57e3f41c39677453a554a7f4885273fc798940c780 | docs/thefinalglance/J-plan/J-49_multi_seat_admission_quota.md |
| DEEP-J-012 | numbered/index original | 87 | 18043 | c3b036aee24a606d6d7debf427fa1596e6a54b38569d338493945a3c6df1b8a2 | docs/thefinalglance/J-plan/J-53_openai_compat_tool_input_repair.md |
| DEEP-J-012 | numbered/index original | 129 | 17832 | da150da29827859d0eb72871ce79575060e22cf882f4fb0b54ee08e09dc3696d | docs/thefinalglance/J-plan/J-59_per_seat_memory_trust_isolation.md |
| DEEP-J-012 | numbered/index original | 74 | 10068 | 325e2e553ed4388a27f0d913c4f4b8fc695fb71e705c6602f11c46858ff9f962 | docs/thefinalglance/J-plan/J-68_self_code_formal_verification.md |
| DEEP-J-012 | numbered/index original | 62 | 8416 | b27555b4a4902d7f9d804fca60f941add6a7e0d5cf01dcf6aeb5470d55615443 | docs/thefinalglance/J-plan/J-71_cedar_policy_as_code_spike.md |
| DEEP-J-012 | numbered/index original | 116 | 14700 | bdaa479f33571f8c1ca52a4bc469ff61652565b6404a8886c9b847a7c0b59e5b | docs/thefinalglance/J-plan/J-84_grcv3_reasoning_effort_classifier.md |
| DEEP-J-012 | numbered/index original | 124 | 17211 | 72afcf551b143da4bc4fbad8dc69360863db92a5332d7b257199983d5a894b07 | docs/thefinalglance/J-plan/J-85_grcv3_overthinking_guard.md |
| DEEP-J-012 | numbered/index original | 138 | 22116 | 35c515d2fdde9be20d8191529f373618ac784359de6c3c6a7a74fe17c3fe364a | docs/thefinalglance/J-plan/J-103_read_access_ladder_interceptor.md |

### DEEP-J-013

Family **J**; totals: **10 files**, **1220 logical lines**, **133280 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-J-013 | numbered/index original | 72 | 9367 | 5a5296af7a190c51b39d966fe05291768dc8f0d8712d383d76319b2e22b397ae | docs/thefinalglance/J-plan/J-14_repro_pack_rra_verify_upgrade.md |
| DEEP-J-013 | numbered/index original | 186 | 12890 | 169788cb27abb10d3e2b2134eeef0b6f636d23ccff44df6ea27f02f9392a5a19 | docs/thefinalglance/J-plan/J-29_usage_window_realtime_surfaces.md |
| DEEP-J-013 | numbered/index original | 120 | 8890 | 93a0c6a0cc59107cb1787986b4ed0fcb4df3215c2bfb04a665957c9e35a91d24 | docs/thefinalglance/J-plan/J-31_canvas_system_mode.md |
| DEEP-J-013 | numbered/index original | 126 | 9311 | 25b1eacdba09a9d68b388aa5341c402a2e0eab0a26170ad8774ba3f29b5c4685 | docs/thefinalglance/J-plan/J-34_empirical_context_calibration.md |
| DEEP-J-013 | numbered/index original | 148 | 23858 | c1795cb2d38b19db819e66709665eb6ecffd377511a9911c154f8b75cbe479da | docs/thefinalglance/J-plan/J-38_agentic_retrieval_loop.md |
| DEEP-J-013 | numbered/index original | 55 | 10818 | 9f9f34cae23380bb0e13233c69edb766905bfd727072f50e9e02f0272f1fa6c5 | docs/thefinalglance/J-plan/J-46_doctrine_debt_rate_measurement.md |
| DEEP-J-013 | numbered/index original | 95 | 11970 | 503e4ba063566a49689b4bc559af36633900f0efebc895b9cb476eeff615ddc7 | docs/thefinalglance/J-plan/J-50_recovery_trajectory_sft_exhaust.md |
| DEEP-J-013 | numbered/index original | 239 | 22580 | 0cb44a43e6a56a7c7a376e483112613d004ce4af1524f3d44e4ab64bdcb3efa8 | docs/thefinalglance/J-plan/J-60_layered_observability_canvas.md |
| DEEP-J-013 | numbered/index original | 68 | 9166 | 807d89feaea3cfbeab235142159e818cee7986af7788c2b55fe4fd4ac38f5483 | docs/thefinalglance/J-plan/J-69_ssb_log_integrity_hash_chain.md |
| DEEP-J-013 | numbered/index original | 111 | 14430 | d378502224735b714c18b57ee521a90478c8f09080c1b5c1985696b83ab1a5af | docs/thefinalglance/J-plan/J-81_autonomous_supervision_hardening.md |

### DEEP-K-001

Family **K**; totals: **7 files**, **1294 logical lines**, **142506 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-K-001 | auxiliary/grounding/fixture | 14 | 613 | ebcc9e1f0f44b3e9063c8b302c169ac19e70dc19f7404275ca9b6df17caac1fd | docs/thefinalglance/K-plan/_fixtures/claims_negative.md |
| DEEP-K-001 | numbered/index original | 589 | 62363 | 287aac9c469ed4cf2ecd557677b548c6f1d57415efb48f4cb03717d57b571092 | docs/thefinalglance/K-plan/K-0_INDEX.md |
| DEEP-K-001 | numbered/index original | 162 | 11522 | e2a15f341705b4a8c34b321594f77c53349d3c5880fa7c58fd565260e9100370 | docs/thefinalglance/K-plan/K-1_problem_quantification_dossier_claims_policy.md |
| DEEP-K-001 | numbered/index original | 143 | 9588 | cb3b3d264988c7de2aa2e547f40fd4b294b2b5c52a3f9e839469288c3d3bd580 | docs/thefinalglance/K-plan/K-8_absorbable_unit_audit_provider_pregate.md |
| DEEP-K-001 | numbered/index original | 152 | 10730 | d68e2d4f78b0b9050f652ad44928835399253a3ce6fb0e87875e379fede24fd5 | docs/thefinalglance/K-plan/K-9_quarterly_model_refresh_cadence.md |
| DEEP-K-001 | numbered/index original | 121 | 20722 | dd3bfff7d98b2a252693700f6bbbee21693625d48324f95426749c0d8fd7686f | docs/thefinalglance/K-plan/K-16_reasoning_receipts_doctrine.md |
| DEEP-K-001 | numbered/index original | 113 | 26968 | 5e1a605f83d748c9bf6ad6c2e1245da57150b4b3ab384a236ad890131944f0c7 | docs/thefinalglance/K-plan/K-20_reexecutable_rra_differentiator.md |

### DEEP-K-002

Family **K**; totals: **7 files**, **1303 logical lines**, **143184 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-K-002 | auxiliary/grounding/fixture | 147 | 12809 | 751afd17f314fa78bab4611187e8a3fe472e6cf32040f04f82273333bc1d003d | docs/thefinalglance/K-plan/_grounding/K-GROUNDING-MAP.md |
| DEEP-K-002 | numbered/index original | 164 | 11834 | f9c013b1d201c54d02f23f45ebbc38f74952133d063e50c91ae0f04171e58230 | docs/thefinalglance/K-plan/K-3_pr_ci_evidence_emitter.md |
| DEEP-K-002 | numbered/index original | 156 | 11039 | a4edbae783504430afc577a5e265e39dae30dcf8af1bc63424241c693dc7adbc | docs/thefinalglance/K-plan/K-6_compliance_export.md |
| DEEP-K-002 | numbered/index original | 110 | 22090 | d2061d2b11b4664e48f73583c5273553b1d171fa3a29b06a5586dd440b0e32fe | docs/thefinalglance/K-plan/K-17_context_management_measured_regime_aware_differentiator.md |
| DEEP-K-002 | numbered/index original | 119 | 25061 | ac4e1c618cbb6f9d8d2adcc8ddba67cfd4ca4b2611159bfd70b3af4544a2ca4d | docs/thefinalglance/K-plan/K-18_case_reuse_economics_compounding_state_differentiator.md |
| DEEP-K-002 | numbered/index original | 124 | 25398 | 3e74bc6363632ce1e9bef2372a5cdd8d46fe1765c35ce69e5e1cc8995b43942b | docs/thefinalglance/K-plan/K-22_doctrine_enforcement_coverage_ledger.md |
| DEEP-K-002 | auxiliary/grounding/fixture | 483 | 34953 | ff6d5bbc0eda55245d57441f2d734c64750ff49ef79298918a3d70ac14231f24 | docs/thefinalglance/K-plan/K-MASTER-BRIEF-2026-06-12.md |

### DEEP-K-003

Family **K**; totals: **9 files**, **1307 logical lines**, **190784 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-K-003 | numbered/index original | 175 | 25427 | 9f6fc2f1dcfd3bc1a3677fc8fd49efb555e2999f3126e0144df67965b7fdd378 | docs/thefinalglance/K-plan/K-2_routing_economics_bench.md |
| DEEP-K-003 | numbered/index original | 113 | 14573 | 8c04dfced393dde37459bd74dd1f88d9b7f43cafdb1f122e3bdfd28816275524 | docs/thefinalglance/K-plan/K-12_losslessness_enforcement_handoff_faithfulness_gate.md |
| DEEP-K-003 | numbered/index original | 220 | 18953 | 160b79b51509af4acaf00e73e0c78ba5ab930271bda917fe53208a5e6b3fd626 | docs/thefinalglance/K-plan/K-13_owasp_control_mapping_injection_posture.md |
| DEEP-K-003 | numbered/index original | 213 | 18830 | 465bdc8774cfa480b4f5fca643c46b70a7026aee516d3fca9cfadadacc3411d5 | docs/thefinalglance/K-plan/K-14_time_awareness_determinism_doctrine.md |
| DEEP-K-003 | numbered/index original | 121 | 20304 | 6a449433d79f468cba0b376aa681f2f54c0ac2c989f9e22c895fc34319aa5dc0 | docs/thefinalglance/K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md |
| DEEP-K-003 | numbered/index original | 128 | 30207 | df926ae2037798a5f8662ddd1a6e284b6c45555835dcb247f0df20e4c1763c1c | docs/thefinalglance/K-plan/K-21_journal_completeness_doctrine.md |
| DEEP-K-003 | numbered/index original | 147 | 28654 | 252072e29c1080986cd380e64d6d5075ad2ef207b030c2b0079ab33be35a72a4 | docs/thefinalglance/K-plan/K-24_credit_billing_monetization.md |
| DEEP-K-003 | numbered/index original | 157 | 27235 | 8cb32b29094eb721cb6bab2accff62db25536060a436adf7a3822d49b3a5a723 | docs/thefinalglance/K-plan/K-26_codex_burn_proof_spine_run.md |
| DEEP-K-003 | numbered/index original | 33 | 6601 | c6502e431e098c9290ed62624290bbd63cb152e707c5307d1084e9890cf185df | docs/thefinalglance/K-plan/K-27_gpt56_sol_isolated_executor_lane.md |

### DEEP-K-004

Family **K**; totals: **8 files**, **1369 logical lines**, **142681 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-K-004 | numbered/index original | 379 | 34183 | 38eb3cbcb76ceed64c8f3c7e289f29c21c96f11c43243aa559919146015bd8fe | docs/thefinalglance/K-plan/K-4_composed_gate_tax_budget.md |
| DEEP-K-004 | numbered/index original | 154 | 10591 | 89b0c6d5ef5f1ee9f31330c69cd6be519af0c2a27c640c609e9149de50128687 | docs/thefinalglance/K-plan/K-5_risk_tiered_gating_policy.md |
| DEEP-K-004 | numbered/index original | 119 | 7364 | 091743d7d303556fe404bac36ec024c5ff4002803e2885c4ff444215d87b5beb | docs/thefinalglance/K-plan/K-7_appsec_insurance_positioning_brief.md |
| DEEP-K-004 | numbered/index original | 108 | 12289 | bcf810f6e3c659299812823a60dbc924265cfabab19d6ec3ab7e7c8504f415db | docs/thefinalglance/K-plan/K-11_human_viewable_evidence_render.md |
| DEEP-K-004 | numbered/index original | 178 | 12943 | 8704bd2573ae333837a98771d8eed8b7943f5ed36e428c288d7a5283a2ea11c8 | docs/thefinalglance/K-plan/K-18a_ps5_observe_arm_testrun.md |
| DEEP-K-004 | numbered/index original | 123 | 25925 | 59987ee41081686af01ba8eb4d0dc160069e31188fca8e6b2122efa8ce429648 | docs/thefinalglance/K-plan/K-19_self_improvement_fitness_doctrine.md |
| DEEP-K-004 | numbered/index original | 145 | 28204 | 7559d984074d2452f762cdd8e0cf164e122b1eea1f9186aa5e9a4ce4855bc1e2 | docs/thefinalglance/K-plan/K-23_multi_repo_workspace_memory.md |
| DEEP-K-004 | numbered/index original | 163 | 11182 | 020c7dc82006573cfee6c24c11543c7dd97fe72a22b8bcbebac7dc74f1d68a94 | docs/thefinalglance/K-plan/K-25_continuous_absorption_watchdog.md |

### DEEP-L-001

Family **L**; totals: **9 files**, **985 logical lines**, **126089 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-L-001 | auxiliary/grounding/fixture | 140 | 11063 | 39f9666a5233ccd29768589c4d6a72fc76e580f2760e4c38c3c4404cff10c477 | docs/thefinalglance/L-plan/_grounding/L-CGG-GROUNDING-MAP.md |
| DEEP-L-001 | auxiliary/grounding/fixture | 63 | 11564 | 39b312d9d09ce658e96d288f838fd531ab0910a0268567a9dc37879ea62e573a | docs/thefinalglance/L-plan/_grounding/L-OQF-GROUNDING-MAP.md |
| DEEP-L-001 | numbered/index original | 105 | 11247 | 4522454c06464bdc1e85f4b6623da4529d042831853bcf592993f901076188b1 | docs/thefinalglance/L-plan/L-4_semantic_entropy_inter_seed_predictor.md |
| DEEP-L-001 | numbered/index original | 103 | 9823 | 37bdeb64f66c1b7298192c97bbf35a1d2d94169f519a5f7c47699e148852cd33 | docs/thefinalglance/L-plan/L-6_risk_as_curiosity_targeted_effort_router.md |
| DEEP-L-001 | numbered/index original | 99 | 10636 | 6abbb9190f4dff67f9be047890428a9fb6b743e0cae4aa19d4952167d443cce5 | docs/thefinalglance/L-plan/L-9_research_math_deepthink_grounding_profile.md |
| DEEP-L-001 | numbered/index original | 107 | 12396 | 4d2c5ca95cec82579c0dc9b40627dae5be4aa8ac33d827a02a435914f36aa3e6 | docs/thefinalglance/L-plan/L-12_skill_front_door_self_improving_skills.md |
| DEEP-L-001 | numbered/index original | 48 | 3573 | b6fd0e72f8f204a4b14789c036fd2f9f6a95e8764d6ee573447f4b1679d50789 | docs/thefinalglance/L-plan/L-18_projection_corollaries_lossless_timetravel.md |
| DEEP-L-001 | numbered/index original | 121 | 18382 | 4a9696923fea804362ccf6d768b687ba87d764d660bd44710df00f8da2a2c4df | docs/thefinalglance/L-plan/L-22_claimant_authored_falsifiers_demarcation.md |
| DEEP-L-001 | numbered/index original | 199 | 37405 | 38b51473fd103b44d6fb3fcb5695684365ec7d59c4a99c1f3365e47db53efa14 | docs/thefinalglance/L-plan/L-24_conjecture_memory_middle_tier.md |

### DEEP-L-002

Family **L**; totals: **9 files**, **1024 logical lines**, **133375 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-L-002 | numbered/index original | 273 | 47939 | e165c39b5218e9e2d259844cf1f59b9186151c3371e813c5833d342cff8699de | docs/thefinalglance/L-plan/L-0_INDEX.md |
| DEEP-L-002 | numbered/index original | 102 | 9639 | 9775facb9f9b4d0e7b55c220546d3d0984540ab2b4346aa0ad7beee5aef8e09c | docs/thefinalglance/L-plan/L-1_typed_claim_extraction_substrate.md |
| DEEP-L-002 | numbered/index original | 106 | 10963 | a5be632d50e13873b2bb8b5a06ee01e07fbef0d0033c1aa42aee1a64cd6608ce | docs/thefinalglance/L-plan/L-2_claimrisk_score_convergence_fuse.md |
| DEEP-L-002 | numbered/index original | 104 | 10341 | d87f38926588acfcebd16daf22557182eb1f3a1183eb5c72cd2d52d4b54a29e8 | docs/thefinalglance/L-plan/L-7_hallucination_policy_gate_abstention.md |
| DEEP-L-002 | numbered/index original | 98 | 10108 | bfd9e7d2e4cd379fc63827d2f93250f32ca4e36bf7b8217592dac58585c564a8 | docs/thefinalglance/L-plan/L-8_working_memory_hygiene_prune_spurious.md |
| DEEP-L-002 | numbered/index original | 110 | 12820 | fe136677f450bae306a65be16f3536caff068861803bedc2af2f4374115524d8 | docs/thefinalglance/L-plan/L-14_one_shot_verified_builder_front_door.md |
| DEEP-L-002 | numbered/index original | 48 | 3296 | 1db66b809c7dd791ad73e3a5bd18ce989e5baf56898e96912bf79df96611450d | docs/thefinalglance/L-plan/L-19_ambient_pagein_budget_ladder.md |
| DEEP-L-002 | numbered/index original | 55 | 7596 | b64df00b3ef72a4ee5093912bc694c321c652520056e13de9637fed70a611371 | docs/thefinalglance/L-plan/L-21_weak_lane_verification_intensity.md |
| DEEP-L-002 | numbered/index original | 128 | 20673 | 2cb97e97e92939949cccc40a2c80ce69403daa7e1fea86060b41a026058db51f | docs/thefinalglance/L-plan/L-23_reason_explore_profile_conjecture_criticism.md |

### DEEP-L-003

Family **L**; totals: **9 files**, **981 logical lines**, **101509 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-L-003 | numbered/index original | 100 | 9164 | 28ebb08e8a43609440e155b85bcfd395eb0ee842469d93493406c0a6dd65eb98 | docs/thefinalglance/L-plan/L-3_evidence_round_trip_predictor.md |
| DEEP-L-003 | numbered/index original | 103 | 11568 | b254e847133d4d7cbb095f2f49dfa07d8cd73a8d0b3aff23238c125c595ea14f | docs/thefinalglance/L-plan/L-5_counterfactual_do_test_on_model_claims.md |
| DEEP-L-003 | numbered/index original | 105 | 11367 | abbfeceafb19d8e5990965bc9ef25146d4c094a63ed7ced77dd2ac3df4a2a47b | docs/thefinalglance/L-plan/L-10_cgg_hallucination_delta_bench.md |
| DEEP-L-003 | numbered/index original | 121 | 15291 | 23829927375fc942d71e8e98da5b2398c87dc8e2b9331be82744251c420b216a | docs/thefinalglance/L-plan/L-11_output_quality_enforcement_system.md |
| DEEP-L-003 | numbered/index original | 108 | 12609 | f3a811ffb63919e3914bf7d9517b3d2d01bfbf3385f300e901a6ae2a3d2ded9e | docs/thefinalglance/L-plan/L-13_user_authorable_rules_hooks_surface.md |
| DEEP-L-003 | numbered/index original | 201 | 17531 | c2b41ec4e57e9666348a96933f6a7f318310945ab43dbaf65bec58ecfb346121 | docs/thefinalglance/L-plan/L-15_repair_controller_and_evidence_first_reorder.md |
| DEEP-L-003 | numbered/index original | 137 | 11437 | c9b2613f249cc21b0dcebbb1fb82d05f876d65778371a86394874d601ff72e51 | docs/thefinalglance/L-plan/L-16_immutable_transcript_pure_fold_view_core.md |
| DEEP-L-003 | numbered/index original | 49 | 3738 | 9432f36e365840e51762961d7b5dc72c218a168af9970523c798fc8ed9b0b774 | docs/thefinalglance/L-plan/L-17_freeze_epoch_controller_prefix_invariance.md |
| DEEP-L-003 | numbered/index original | 57 | 8804 | 5f0f0e227b907cf71ed0e85ea2b3aa5d2af2ab9cee0f3c0fa659378dec59cafd | docs/thefinalglance/L-plan/L-20_commit_mined_eval_supply_competitor_arm.md |

### DEEP-M-001

Family **M**; totals: **7 files**, **542 logical lines**, **38550 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-M-001 | numbered/index original | 85 | 6197 | 073b9b85ab3fb4a8f06b5ae6de9337bf0d2a8a5bdf1a1153a2385afc1b7937f0 | docs/thefinalglance/M-plan/M-0_INDEX.md |
| DEEP-M-001 | numbered/index original | 80 | 5643 | b33b28afb89a38af7f16c44ab460711a289e2acdb53725b53c04ece7a47846f1 | docs/thefinalglance/M-plan/M-1_seat_entitlement_and_purchase_rail.md |
| DEEP-M-001 | numbered/index original | 71 | 4871 | 53282816d0e9a65c46d31db764e55e917d081efa238dbab893a6d0a1ba61608c | docs/thefinalglance/M-plan/M-2_org_policy_pack_distribution.md |
| DEEP-M-001 | numbered/index original | 74 | 5103 | 581dd9681d2d808bac5be83a6a670b40d1a3e12f18e4cec44d482e6fe086d377 | docs/thefinalglance/M-plan/M-3_fleet_evidence_rollup.md |
| DEEP-M-001 | numbered/index original | 71 | 4777 | 847be6d785c08a6b9fe5d5a1ef9fa217d4823dfdc59f5c15bb1074afb42799be | docs/thefinalglance/M-plan/M-4_signed_release_channel_updater_support.md |
| DEEP-M-001 | numbered/index original | 72 | 5030 | 0f80fa7736939ca175da1d45971bd60ae191314f4d863f8a9268b9ef4a1aef18 | docs/thefinalglance/M-plan/M-5_commercial_packaging_verifier_oss.md |
| DEEP-M-001 | numbered/index original | 89 | 6929 | 1c36017e8e507a3bc2e80b4ae77f0256d537af2571fc49d48430df2dbe0e085b | docs/thefinalglance/M-plan/M-6_evidence_schema_open_standard.md |

### DEEP-N-001

Family **N**; totals: **7 files**, **710 logical lines**, **64575 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-N-001 | numbered/index original | 373 | 33912 | 380b962c1ebf960ee093477068a2e732a8856f2c819a1e10dfd9597336311614 | docs/thefinalglance/N-plan/N-0_INDEX.md |
| DEEP-N-001 | numbered/index original | 59 | 6268 | ecabff9120f3a85cd012d770799cbbc9bb9a5d88a08e8cd27538ce70784b072a | docs/thefinalglance/N-plan/N-1_measurement_spine.md |
| DEEP-N-001 | numbered/index original | 57 | 4460 | 009029b733c36a472e3db12ebc1dc45302683284d703082ba20e6ae7b1d326ce | docs/thefinalglance/N-plan/N-3_anti_selection_bias_promotion.md |
| DEEP-N-001 | numbered/index original | 55 | 5296 | 67b66bbcb1d36cd0f1eb8b41d6e6eddb143d585198977f88302b0b32355f9962 | docs/thefinalglance/N-plan/N-4_claim_object_receipt_memo.md |
| DEEP-N-001 | numbered/index original | 54 | 3930 | 031868178de3f9eac947220bbfac38e7039c6c5aa6ff44eb7adf213a51341f54 | docs/thefinalglance/N-plan/N-9_counterfactual_ablation_engine.md |
| DEEP-N-001 | numbered/index original | 54 | 4594 | 4d3ce688bcb4443d71343022a3c357207df55e633f5c6b5fafe740a9d1eaecba | docs/thefinalglance/N-plan/N-11_journal_fold_substrate.md |
| DEEP-N-001 | numbered/index original | 58 | 6115 | 07ad46538c0f2e4c3568d9cc904e4d956b9604cf12e4d4463b522e85d5325ca5 | docs/thefinalglance/N-plan/N-12_consistency_dedupe_bundle.md |

### DEEP-N-002

Family **N**; totals: **10 files**, **719 logical lines**, **82251 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-N-002 | numbered/index original | 57 | 5236 | bd230d9d268293c65b22287fdeefed91c81c7248bb916e9e875adb4e5cd027ab | docs/thefinalglance/N-plan/N-2_corpus_composition_validator.md |
| DEEP-N-002 | numbered/index original | 55 | 5037 | 27a3f43935d9a318a133edf0326ae1281fb0bc56e2a135f35892e230f8556189 | docs/thefinalglance/N-plan/N-5_policy_intensity_arbitration.md |
| DEEP-N-002 | numbered/index original | 54 | 5195 | 876746ff47031df3188adaddd4a438efa57b000f3c3102f404e2f58c4633882a | docs/thefinalglance/N-plan/N-6_provenance_lattice_admission.md |
| DEEP-N-002 | numbered/index original | 52 | 4368 | 1923fa7ef56cba8ae3ec387a36b14250a5730925c66b5c46de54dcc1dcb4a62e | docs/thefinalglance/N-plan/N-7_human_render_verdict_field.md |
| DEEP-N-002 | numbered/index original | 59 | 4813 | 0e442ba89a04d07f95bf035bc19ff266483c5876fa2e72cc1cb2ede41f90f9b0 | docs/thefinalglance/N-plan/N-8_governance_coverage_substrate.md |
| DEEP-N-002 | numbered/index original | 55 | 4219 | 19f7948091632513f8fdb33c52d3dc80175578f2b7056c50663b090c8258b8a9 | docs/thefinalglance/N-plan/N-10_clarity_novelty_budget_wires.md |
| DEEP-N-002 | numbered/index original | 104 | 14233 | 69d44f8905a81b1b199120c9679e591d6eb338a55a711b95d76481b7d1c66205 | docs/thefinalglance/N-plan/N-13_output_boundary_and_structured_output_capability_matrix.md |
| DEEP-N-002 | numbered/index original | 115 | 15262 | a55d75ade3125c4f86d953b1b4528032ba5c40a69a8e366c311b8e0db9af1923 | docs/thefinalglance/N-plan/N-14_self_improvement_edit_loop_instruments.md |
| DEEP-N-002 | numbered/index original | 107 | 14044 | 51f58c6b19994de97eae4fe326c5544bf35cc904f7c38461a70b90f19e00217b | docs/thefinalglance/N-plan/N-15_anti_affirmation_stability_measurement.md |
| DEEP-N-002 | numbered/index original | 61 | 9844 | 9e08bcfb2a4021807fc0abc4b68a3e1513289d1d538c0001e9fc095da4384db9 | docs/thefinalglance/N-plan/N-16_governance_closure_grounding_floor_ritual.md |

### DEEP-O-001

Family **O**; totals: **5 files**, **491 logical lines**, **48702 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-O-001 | numbered/index original | 125 | 9838 | a7988ad7c1f9c3dac2bd89d12996f3256d4aef0e1b5c4c11deb0e1f1fa41c217 | docs/thefinalglance/O-plan/O-0_INDEX.md |
| DEEP-O-001 | numbered/index original | 90 | 9520 | d1b23db489099d48d375c58b353da55eb9e40ffc2d7b9ad0b9c3a256bf1ece38 | docs/thefinalglance/O-plan/O-1_research_daemon_generalized_cosmos.md |
| DEEP-O-001 | numbered/index original | 91 | 9517 | db5ba4492f42d55246c7a7c969c2337f86003fb55e2ef9e4bb11da8f2b915584 | docs/thefinalglance/O-plan/O-2_scheduled_taskspec_substrate.md |
| DEEP-O-001 | numbered/index original | 91 | 9992 | a30462d74e297fa7935ec8c62484b27546ef6ce41c5d210e2f634902632754c0 | docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md |
| DEEP-O-001 | numbered/index original | 94 | 9835 | 5deae2ce45ece1bba69c4bdaeead2222a5f727785603b599d7c1a10ac9c71899 | docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md |

### DEEP-P-001

Family **P**; totals: **9 files**, **1070 logical lines**, **122799 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-P-001 | numbered/index original | 400 | 40056 | e4783985880e5c440d4f12e82b7bdb7052d5623907afa918f12c8bccc4ae4f64 | docs/thefinalglance/P-plan/P-0_INDEX.md |
| DEEP-P-001 | numbered/index original | 94 | 12140 | 65b0018490073b8d28e30521864f045daf66fe9bbc101e93e8c22e35ec1e6294 | docs/thefinalglance/P-plan/P-4_living_external_brains.md |
| DEEP-P-001 | numbered/index original | 93 | 11534 | eb70fdf5a46aec0c4014bc913cc4348d2c78a84ff6165857c9b12ead02de5068 | docs/thefinalglance/P-plan/P-5_operator_surface_graduated_ceremony.md |
| DEEP-P-001 | numbered/index original | 94 | 12190 | 4b3e1fb40ddf4de4c1d24aae1a7aed61a9eec5b07935abee30ba0fa90d21537c | docs/thefinalglance/P-plan/P-8_small_team_collaboration_grain.md |
| DEEP-P-001 | numbered/index original | 81 | 10245 | f8d9af7f699c237af747ef55bca147943f6e03ef513364a9b1710fb91b1ef54e | docs/thefinalglance/P-plan/P-12_desktop_operator_lane.md |
| DEEP-P-001 | numbered/index original | 77 | 8680 | 527436dc3556e8b80053fc58a50e09988f801ebf3ad5aadd9f61bfeceb3817ea | docs/thefinalglance/P-plan/P-13_perception_provenance_setofmarks.md |
| DEEP-P-001 | numbered/index original | 77 | 9098 | 227bb3b14f5d6946df430583360d7d910052580066e71a305877914fa9499d46 | docs/thefinalglance/P-plan/P-16_gated_install_catalog.md |
| DEEP-P-001 | numbered/index original | 78 | 9290 | 36743299cd8f04071b10ff08d23c6b59f3bf3adc23510b0750d78e950115f2c6 | docs/thefinalglance/P-plan/P-18_durable_taskspec_drift_binding.md |
| DEEP-P-001 | numbered/index original | 76 | 9566 | 7d980e646b040d25663fc2fb262ac3b0e2d15ab12ecc4da768003c0082bfa3e5 | docs/thefinalglance/P-plan/P-21_gate_credibility_hardening.md |

### DEEP-P-002

Family **P**; totals: **10 files**, **918 logical lines**, **122607 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-P-002 | numbered/index original | 95 | 11822 | d37e4f9bb060c26c2da9ff6aeeaaee089c49127c7dc428026b89b6aaeb279c2f | docs/thefinalglance/P-plan/P-1_app_fleet_layer.md |
| DEEP-P-002 | numbered/index original | 93 | 11112 | a4b1e071309b819073fad0e62f42af5569a368b099cb59d7a1192dc863844ec1 | docs/thefinalglance/P-plan/P-2_inter_tool_ports_world_bus.md |
| DEEP-P-002 | numbered/index original | 94 | 10983 | bd5fe085984c05614ac2943214f12081317ec24613c84620124ba3e04d1ba99a | docs/thefinalglance/P-plan/P-6_life_data_privacy_lattice.md |
| DEEP-P-002 | numbered/index original | 90 | 11089 | 5506a5ca7700859d191ae3e25db952baeb5a3410ed6817f9dcf9e89c19057e02 | docs/thefinalglance/P-plan/P-7_per_user_operating_profile.md |
| DEEP-P-002 | numbered/index original | 101 | 14764 | 3d47ec75ebbaec3d718f69b6426a88579ded6b0ca3b098b4f4b1a0d5c9042530 | docs/thefinalglance/P-plan/P-9_connector_pack_admission_format.md |
| DEEP-P-002 | numbered/index original | 107 | 14909 | b924052721b5b738abe57794f68033662225b536d7f67edde7557c795d26bba1 | docs/thefinalglance/P-plan/P-10_ingress_adapters_credential_vault.md |
| DEEP-P-002 | numbered/index original | 100 | 13218 | d3818f462fb22db5733e2685f76fc5e6fba00357616408f942e881bf45c17824 | docs/thefinalglance/P-plan/P-11_workflow_from_goal.md |
| DEEP-P-002 | numbered/index original | 77 | 9717 | 810344081dedc569caba34343b1c7aa46c6044607cbc682b8e06edaa1dc139af | docs/thefinalglance/P-plan/P-14_explore_then_crystallize_lane.md |
| DEEP-P-002 | numbered/index original | 78 | 9147 | 06567a35e138a1ac7c86497887476bac72a0f88cc03d2cae6ea312caff70e4d1 | docs/thefinalglance/P-plan/P-19_config_ops_lifecycle_doctor.md |
| DEEP-P-002 | numbered/index original | 83 | 15846 | 7981ce57799a720f8a425dc0c1aa82332d840b9c7d4b019a9de6978045f85f3a | docs/thefinalglance/P-plan/P-24_mobile_design_test_lane.md |

### DEEP-P-003

Family **P**; totals: **7 files**, **1035 logical lines**, **153893 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-P-003 | numbered/index original | 93 | 11657 | 4ac4764e4c6ab3881e92598b5b15f0d6aabf96bfa3f17e2b955cbc6fd49495bb | docs/thefinalglance/P-plan/P-3_persona_split_export.md |
| DEEP-P-003 | numbered/index original | 77 | 9049 | c15922c9dd2b136af80d5f61fac6cc43d7d81b383cd7d8de23ccad840b1d474c | docs/thefinalglance/P-plan/P-15_verified_case_graph_hardening.md |
| DEEP-P-003 | numbered/index original | 78 | 9351 | a76a09561afff042a200cec0af05b0a3f1bf5c868905a9752e3dca2047b0ed89 | docs/thefinalglance/P-plan/P-17_provider_failover_economics.md |
| DEEP-P-003 | numbered/index original | 77 | 8801 | 8ba700189493b76a89e2f7d8c4f31b731eb9df0104b66b33c8fead5bf52c69df | docs/thefinalglance/P-plan/P-20_context_memory_lifecycle.md |
| DEEP-P-003 | numbered/index original | 78 | 9265 | c7a0e27a3d10a0ac7a8e3bf0b519da30daa6327fbf0d0f15a883ca2ab1cb1d6c | docs/thefinalglance/P-plan/P-22_control_plane_channel_hardening.md |
| DEEP-P-003 | numbered/index original | 84 | 13182 | 6f6f08489b8d4412276b0c787a872ed2cb8a0c1604b1c83c641635ea50fbc59d | docs/thefinalglance/P-plan/P-23_journaled_cobrowse_lane.md |
| DEEP-P-003 | numbered/index original | 548 | 92588 | f9bcc1b23d664ea03560d42c57f5bb7794a0a01d2587870882f3e9e740ca294f | docs/thefinalglance/P-plan/P-25_exogenous_injection_creativity_falsifier.md |

### DEEP-Q-001

Family **Q**; totals: **9 files**, **1116 logical lines**, **162025 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-Q-001 | auxiliary/grounding/fixture | 113 | 11441 | 7bc3b6ec8fc96ce59a1f082fa4afb1f769aa8301464b16f5650fd95107d76f22 | docs/thefinalglance/Q-plan/_grounding/Q-RESEARCH-CREATIVITY-GROUNDING-MAP.md |
| DEEP-Q-001 | numbered/index original | 162 | 35618 | 4a7ce5061c11bbcf8596ae982b2a4d788d176a91f2d1d690ed843fead9db010e | docs/thefinalglance/Q-plan/Q-0_INDEX.md |
| DEEP-Q-001 | numbered/index original | 93 | 11762 | 6725612644187c939676618863e579dde1243671b0eb40401da4ef39bb4045f8 | docs/thefinalglance/Q-plan/Q-5_local_token_hash_journal.md |
| DEEP-Q-001 | numbered/index original | 94 | 11114 | d6a17cdf6af522a32780a6ec8f87bdc177fd17f47d0b976f353fd27f081bd328 | docs/thefinalglance/Q-plan/Q-7_weak_lane_codestyle_bench.md |
| DEEP-Q-001 | numbered/index original | 91 | 11675 | d4344405ca75fb9e1c7ab8f9c60ce0cf2335363b43f88ee4335d022b29b544db | docs/thefinalglance/Q-plan/Q-8_verifier_size_ceiling.md |
| DEEP-Q-001 | numbered/index original | 92 | 12287 | 3c17750339f9e48f77c6e54f62e7bab50b21ebd58e8548d9f185b7cfa4074b12 | docs/thefinalglance/Q-plan/Q-9_distribution_hygiene.md |
| DEEP-Q-001 | numbered/index original | 131 | 14406 | 9871bc096248ed580ad5bbb557fb2398e420496ce178411cf5c114d8cbdb447d | docs/thefinalglance/Q-plan/Q-13_receipt_bound_briefing.md |
| DEEP-Q-001 | numbered/index original | 164 | 25302 | 187e41ccbba4049cf49f6d6ddd326edbaa7521d284a1551ee9996919661e8c29 | docs/thefinalglance/Q-plan/Q-14_conjecture_engine_instrument_hardening.md |
| DEEP-Q-001 | numbered/index original | 176 | 28420 | ae5dccf0a782912f3930c629dbdd0fc657331a0a364f499bd72a37828ef3d207 | docs/thefinalglance/Q-plan/Q-15_research_bench_and_protocol_hygiene.md |

### DEEP-Q-002

Family **Q**; totals: **9 files**, **1113 logical lines**, **162098 bytes**; target status: **under 1,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-Q-002 | numbered/index original | 95 | 13688 | 95179599ebcfed23e6405975eb1735584b744e6806a045aa44bcdaa322f5e405 | docs/thefinalglance/Q-plan/Q-1_standing_goals.md |
| DEEP-Q-002 | numbered/index original | 91 | 11312 | 7cbde4d3398b283ae56f4a413ef28a7215d031cfbf52e245c21183f1ae84e67c | docs/thefinalglance/Q-plan/Q-2_quorum_wake_gate.md |
| DEEP-Q-002 | numbered/index original | 96 | 12328 | 676630e95d49357032567a4ab846b566b12e16b9f36659adf2d4daf11b24163d | docs/thefinalglance/Q-plan/Q-3_advisor_lane.md |
| DEEP-Q-002 | numbered/index original | 93 | 12330 | abd83fb83b1fc030bcc98a22e9655ef54acb57e3d8d93c60fd203159a06cc476 | docs/thefinalglance/Q-plan/Q-4_swap_restart_attribution.md |
| DEEP-Q-002 | numbered/index original | 93 | 12205 | b658bf9ad2c6d81e94a4281b3403070fdbc98aa78ffd085595c5452e4beb7078 | docs/thefinalglance/Q-plan/Q-6_gate_evidence_corruption_corpus.md |
| DEEP-Q-002 | numbered/index original | 167 | 21667 | 99f138ab6ba391ccbd1aa777bb9f5745ac2b6f6fe5a01ad74f91cda6cdad886f | docs/thefinalglance/Q-plan/Q-10_research_conditioned_creativity.md |
| DEEP-Q-002 | numbered/index original | 158 | 21535 | 102a467785cdfa8549f190f3a90777f3538b1f6c61a42057df6233d1cee43617 | docs/thefinalglance/Q-plan/Q-11_novelty_instrument_and_fixed_budget_bench.md |
| DEEP-Q-002 | numbered/index original | 145 | 21655 | 8c657995b40d3c6a0814272c924b86fddccfdc9e59ce440fdbc2ceaaba1f1504 | docs/thefinalglance/Q-plan/Q-12_research_lane_context_and_search_policy.md |
| DEEP-Q-002 | numbered/index original | 175 | 35378 | caf36288155a25a57ce76dd4be856a690b567e93405f1fd3868d7fc6883a86b3 | docs/thefinalglance/Q-plan/Q-16_elai_on_elai_e2e_functional_suite.md |

### DEEP-PHASE-001

Family **PHASE**; totals: **6 files**, **2692 logical lines**, **165923 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-001 | numbered/index original | 780 | 49096 | 53aba5d44d1ee36a23b8b71cbd8e2f3109f971bbaa2002d55f642b5619d66aaa | docs/specs/guide/steps/phase-0.txt |
| DEEP-PHASE-001 | numbered/index original | 531 | 37863 | 79001cdb64eaac78430acef16e34874d838aebca6ec9fa2684825f1b50c4d1a2 | docs/specs/guide/steps/phase-1C.txt |
| DEEP-PHASE-001 | numbered/index original | 343 | 22625 | 4cf40b6145687d1efd74f3e34b419c5eaefb8ffa3eea13ccaa4313816ac44b09 | docs/specs/guide/steps/phase-3A.txt |
| DEEP-PHASE-001 | numbered/index original | 146 | 6516 | 89c25d16ba40711cb101015ac8daa3159a055ba687c34e72323499c15b77e1e4 | docs/specs/guide/steps/phase-3J-BASELINE.txt |
| DEEP-PHASE-001 | numbered/index original | 427 | 26506 | 986b8dbdda09c2980a3703e34061b272552b29ef4f6635d8cf6e72415b6789a2 | docs/specs/guide/steps/phase-4K.txt |
| DEEP-PHASE-001 | numbered/index original | 465 | 23317 | d4607fa920edeca49355cf134fecb5452d4ce40daf5ad740eb23647d9d0f6ffa | docs/specs/guide/steps/phase-4M.txt |

### DEEP-PHASE-002

Family **PHASE**; totals: **6 files**, **2750 logical lines**, **178707 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-002 | numbered/index original | 513 | 35090 | 69a8a3c800c8d7d7dfcf87e8a9c1afa11ed788536229692f9733691f8df77cac | docs/specs/guide/steps/phase-1A.txt |
| DEEP-PHASE-002 | numbered/index original | 332 | 27167 | 94540288abb14374c3dd04b141f1b9c5cd580c72bc14e3edca1f9073c29258c2 | docs/specs/guide/steps/phase-1D.txt |
| DEEP-PHASE-002 | numbered/index original | 388 | 28121 | 005df90dea6fb7731f33352bf7c02d8c4ac3fe97c5a258a602aa35974225fe5a | docs/specs/guide/steps/phase-3D.txt |
| DEEP-PHASE-002 | numbered/index original | 463 | 28390 | 82d8892683f3dbb18ef43736617debc138af5d920551cd70a0dbbefd3469e051 | docs/specs/guide/steps/phase-3G.txt |
| DEEP-PHASE-002 | numbered/index original | 137 | 4940 | 728470c4ff73b333dc416eb572e703ac1b8cca0bd250315086eee3d2355fc04d | docs/specs/guide/steps/phase-4D-ORACLE.txt |
| DEEP-PHASE-002 | numbered/index original | 917 | 54999 | a55360f45161ddb73570cd00fdebe32b1c2a6c6fcb971f28740b76e04e9f0169 | docs/specs/guide/steps/phase-4G.txt |

### DEEP-PHASE-003

Family **PHASE**; totals: **6 files**, **2751 logical lines**, **182824 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-003 | numbered/index original | 448 | 28063 | d5ed0a32e83f7ca32ed95f9bd90fea7bf62fbd10d88c820963d26d3ce5e623a2 | docs/specs/guide/steps/phase-1B.txt |
| DEEP-PHASE-003 | numbered/index original | 338 | 31345 | 45f6e19763ec91538d7b858de0fdada6ec267e50f553dd7908ff6073485f7053 | docs/specs/guide/steps/phase-1G.txt |
| DEEP-PHASE-003 | numbered/index original | 521 | 33410 | 86c9252f116e45e3ba0d0d807f8d96cf3fc1197c228588a1acdeab227b44214f | docs/specs/guide/steps/phase-3H.txt |
| DEEP-PHASE-003 | numbered/index original | 143 | 6038 | 476cee5599a05897eeb1f2cbdf5b3f4338772a7d6b069db86e3461e42027879a | docs/specs/guide/steps/phase-4D-SURGICAL.txt |
| DEEP-PHASE-003 | numbered/index original | 911 | 53599 | 6842d193804e86909b82f59c901063635a44ef8aad23d89c62dd31338ac560e2 | docs/specs/guide/steps/phase-4E.txt |
| DEEP-PHASE-003 | numbered/index original | 390 | 30369 | f6e224a4175ba3d264ce5f7f41158b47c3c97d9238fa63d66beb6516169e2cf9 | docs/specs/guide/steps/phase-index-3A-4J.txt |

### DEEP-PHASE-004

Family **PHASE**; totals: **6 files**, **2773 logical lines**, **176724 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-004 | numbered/index original | 443 | 24750 | 8ed050b69b6cf2b19a0915fdd7cea9bfa83844c0baa7a4d79f747933083d51cb | docs/specs/guide/steps/phase-1E.txt |
| DEEP-PHASE-004 | numbered/index original | 580 | 37770 | 80df17b315e70cbd3ecaeea8fe22ccf0414bc2b85efc2d0a1807b58b5e9b3f51 | docs/specs/guide/steps/phase-2B.txt |
| DEEP-PHASE-004 | numbered/index original | 496 | 33046 | ef2a2fc06ae8035a3d88a5aad8211be3b77e21507bc8a2982a7000ebe14435ee | docs/specs/guide/steps/phase-2C.txt |
| DEEP-PHASE-004 | numbered/index original | 251 | 13522 | 23ba0f82895dea2027f715e9699759dfc1e9e36e3fe58e0973efcf615dfbc048 | docs/specs/guide/steps/phase-3L.txt |
| DEEP-PHASE-004 | numbered/index original | 380 | 24461 | 9cfe4c5fb4337ac75555bfde25473912c921adf9ed0fda1f6e5300699b76a75e | docs/specs/guide/steps/phase-4C.txt |
| DEEP-PHASE-004 | numbered/index original | 623 | 43175 | e220d899bc5f0b109e424509213c289a54cc06c31f11db810b300a3d0f6a379b | docs/specs/guide/steps/phase-4I.txt |

### DEEP-PHASE-005

Family **PHASE**; totals: **4 files**, **2735 logical lines**, **169614 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-005 | numbered/index original | 332 | 22187 | 9420ec0aae677f6d990efa9a20c6c3974901a801ec9ab82a04598148883053b9 | docs/specs/guide/steps/phase-1F.txt |
| DEEP-PHASE-005 | numbered/index original | 382 | 24128 | a536492e24d32c30904c7009542f0e5d55405a3f04c0481eb90f648c58fd8f0d | docs/specs/guide/steps/phase-2A.txt |
| DEEP-PHASE-005 | numbered/index original | 448 | 27252 | 60b6a091ca87767dd3849eda51bc591a3c8bd00b5ab59e76544a0dca45559e99 | docs/specs/guide/steps/phase-3F.txt |
| DEEP-PHASE-005 | numbered/index original | 1573 | 96047 | 8b3ffcf2840b74cb88e1dddf02d9f7cea8bbfcac56cff662730d5f16f0a863a8 | docs/specs/guide/steps/phase-3J.txt |

### DEEP-PHASE-006

Family **PHASE**; totals: **5 files**, **2791 logical lines**, **174259 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-006 | numbered/index original | 290 | 26430 | 918196c6188165434d0933fc422002ba58571eb4e3c52645fd70e17b14df8a49 | docs/specs/guide/steps/phase-1H.txt |
| DEEP-PHASE-006 | numbered/index original | 465 | 29892 | ee9455d1cdbb4001df33e8d254d3ac7104e50668a764aaf9eab212aa4f5f494f | docs/specs/guide/steps/phase-3I.txt |
| DEEP-PHASE-006 | numbered/index original | 317 | 17348 | 582c9d9b378529533e099d854a17fa1eb28171d8877df4d86af8cf027dc593e6 | docs/specs/guide/steps/phase-3K.txt |
| DEEP-PHASE-006 | numbered/index original | 427 | 25113 | 319b96ed8902212550fb6ece82dfe1c94aa2155254358c31dec65fc4f270ba43 | docs/specs/guide/steps/phase-4A.txt |
| DEEP-PHASE-006 | numbered/index original | 1292 | 75476 | 4c7472fffc73db1c463bae62e06ed05e7a2b6e7812159919d30af26fd5727b38 | docs/specs/guide/steps/phase-4H.txt |

### DEEP-PHASE-007

Family **PHASE**; totals: **5 files**, **2723 logical lines**, **166521 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-007 | numbered/index original | 343 | 24838 | e6b45aa69b8027736c2ce27fef682579be0eb087aa5892f4b6ac3225be9ee429 | docs/specs/guide/steps/phase-1I.txt |
| DEEP-PHASE-007 | numbered/index original | 428 | 28356 | c9a3a7dff05032f0274ad689099e76919b3960ecb129aca23f152b6b669560c7 | docs/specs/guide/steps/phase-2D.txt |
| DEEP-PHASE-007 | numbered/index original | 492 | 30997 | d8a018ef04d8868e5e21a7768cef3c007f87a3a4f67574ffe788f4b28307a034 | docs/specs/guide/steps/phase-3B.txt |
| DEEP-PHASE-007 | numbered/index original | 1260 | 72544 | 96bfd91863d5f645bdace4d3c4f38acac92824baa37d74744a73d70ffb675fdc | docs/specs/guide/steps/phase-4D.txt |
| DEEP-PHASE-007 | numbered/index original | 200 | 9786 | 470a805e489fa0d12e71287178405dffb9ec571beb18cb85ea6508b06903b4eb | docs/specs/guide/steps/pre-phase-0.txt |

### DEEP-PHASE-008

Family **PHASE**; totals: **6 files**, **2697 logical lines**, **162325 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-PHASE-008 | numbered/index original | 471 | 34085 | 9882d9766a3264b45b4bc00f247ddda9228c29d6590ce4acd6c0d4806d353007 | docs/specs/guide/steps/phase-3C.txt |
| DEEP-PHASE-008 | numbered/index original | 366 | 20014 | 68c72aecd1646f95783cfada9ad61df282c07233f00bab884064369738a78406 | docs/specs/guide/steps/phase-3E.txt |
| DEEP-PHASE-008 | numbered/index original | 429 | 24664 | 8159bf3470c9f05bb2e7b8c78e64918f6d524fdf9ecc539bc3f05b7ea7e24cce | docs/specs/guide/steps/phase-4B.txt |
| DEEP-PHASE-008 | numbered/index original | 560 | 31501 | c4ed8971681950477441c281e2e099811e2cb7a3985eb5e8d55e21a8d4b048fd | docs/specs/guide/steps/phase-4F.txt |
| DEEP-PHASE-008 | numbered/index original | 710 | 42175 | 35fe9b4a6c60bfdfda1ecda476ab7d614d4aff07b34bd22d67db658de5a83e55 | docs/specs/guide/steps/phase-4J.txt |
| DEEP-PHASE-008 | numbered/index original | 161 | 9886 | 93359d382f21eaeab9e0c710af2113551bbab04fd10ff8424774334ddd74f6a8 | docs/specs/guide/steps/phase-index-pre0-2D.txt |


// F-9 / Slice D4 fixture: file containing a hallucinated crate import.
// Used by f9_dep_writetime_gate_contract tests to verify DepWriteTimeGate
// returns a Block(NotFound) finding for `reqquests` (known typo in
// StubRegistryChecker::KNOWN_TYPOS).
//
// This file is RAW SOURCE (not a diff): the dep-check scanner reads file
// content directly. The `use reqquests::...` line is the trigger — `reqquests`
// is a known-hallucinated name in StubRegistryChecker -> NotFound -> Block.
// (BLOCK-4 fix: the prior leading '+' diff-marker made extract_rust_deps skip
// the line via strip_prefix("use "), so the contract test never blocked.)

use reqquests::get;
use serde::Deserialize;

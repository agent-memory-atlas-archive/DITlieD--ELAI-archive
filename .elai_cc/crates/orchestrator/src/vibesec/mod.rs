// F-9 / Slice D1 (SCHEMA_VERSION 199): VibeSecDep write-time slopsquat gate.
// Checks dependency names at PreToolUse (file write) time against live
// registry probes.  Emits WriteTimeSlopsquatBlocked SSB rows on NotFound /
// TooNew verdicts; fails-open (Warn) when the registry is unreachable (W-10
// network-axis policy).
pub mod dep_writetime_gate;

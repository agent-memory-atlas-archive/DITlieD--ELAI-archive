Phase 1C Step 2 Decisions
Session: s-742324
Date: 2026-04-07

USEARCH FEATURE GATE
  usearch 2.24.0 fails to compile on Windows MSVC.
  Root cause: index_plugins.hpp:404 uses #warning (GCC/Clang extension).
  MSVC error: fatal error C1021: invalid preprocessor command 'warning'.
  Decision: gate usearch behind optional feature "usearch-hnsw" in Cargo.toml.
  VectorIndex trait is pure Rust (compiles without usearch).
  UsearchIndex impl is #[cfg(feature = "usearch-hnsw")] gated.
  Impact: cargo check and tests pass without usearch. Enabling the feature
  requires either a usearch version that fixes the MSVC issue, or building
  from a VS Developer Command Prompt that sets INCLUDE/LIB correctly.
  2.24.0 is the latest version on crates.io as of 2026-04-07.

ELAI-CORE DEPENDENCY
  Added elai-core as path dependency to knowledge/Cargo.toml.
  Required for CircuitBreaker to use ClockProvider/LogicalInstant.
  Import: use elai_core::{ClockProvider, LogicalInstant};

ORT SESSION API
  ort 2.0.0-rc.12 places Session at ort::session::Session (not ort::Session).
  Session::builder() returns Result, commit_from_file takes &mut self.
  Wrapped in catch_unwind for panic safety on bad model paths.

INVALIDATE_FILE SCHEMA GAP
  vector_cache schema has no file_path column.
  invalidate_file() returns Err(CacheError::NotSupported) with explanation.
  Added invalidate_keys() as alternative (caller provides blake2s keys).
  Needs spec clarification for Step 3 or later.

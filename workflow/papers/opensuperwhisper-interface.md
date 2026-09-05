Source: starmel/OpenSuperWhisper
Reliability: MEDIUM
Files examined:
  F:\.ELAI_workflow\docs\.source\OpenSuperWhisper\Package.resolved
  F:\.ELAI_workflow\docs\.source\OpenSuperWhisper\OpenSuperWhisper\Engines\WhisperEngine.swift
  F:\.ELAI_workflow\docs\.source\OpenSuperWhisper\OpenSuperWhisper\TranscriptionService.swift
  F:\.ELAI_workflow\docs\.source\OpenSuperWhisper\OpenSuperWhisper\Settings.swift (full, 1522 lines)
  F:\.ELAI_workflow\docs\.source\OpenSuperWhisper\OpenSuperWhisper\Whis\WhisperFullParams.swift

Language: Swift
Platform: macOS ONLY (AppKit, AVFoundation, CoreAudioTypes, NSWorkspace, Carbon, KeyboardShortcuts)
Build system: Xcode project (OpenSuperWhisper.xcodeproj)
License: MIT (LICENSE file present)
Default branch: master

Swift Package dependencies (from Package.resolved, version 2):
  GRDB.swift  6.29.3  (groue/GRDB.swift)   -- SQLite ORM, used for transcription history storage
  KeyboardShortcuts  1.17.0  (sindresorhus/KeyboardShortcuts)  -- global hotkey registration

Architecture overview:
  App entry: OpenSuperWhisperApp.swift (SwiftUI @main)
  TranscriptionService (singleton, @MainActor) -- orchestrates engine selection and transcription lifecycle
  TranscriptionEngine protocol -- implemented by WhisperEngine and FluidAudioEngine
  WhisperEngine -- whisper.cpp C binding via Swift wrapper (MyWhisperContext, WhisperFullParams)
  FluidAudioEngine -- Parakeet/NeMo model via FluidAudio Swift package (AsrModels, AsrManager)
  AudioRecorder.swift / MicrophoneService.swift -- AVFoundation audio capture
  TranscriptionQueue.swift -- queues transcription jobs
  WhisperModelManager.shared -- downloads and manages .bin model files
  AppPreferences.shared -- UserDefaults-backed settings singleton

Engine selection:
  AppPreferences.shared.selectedEngine: "whisper" | "fluidaudio"
  TranscriptionService.loadEngine() dispatches Task.detached to initialize chosen engine
  Switching engine in Settings calls TranscriptionService.shared.reloadEngine()

Transcription pipeline (WhisperEngine):
  1. initialize(): loads .bin model file from AppPreferences.shared.selectedWhisperModelPath via MyWhisperContext.initFromFile()
  2. transcribeAudio(url:settings:):
     a. Resolve file URL (detect MP4 header by magic bytes [66 74 79 70] / "ftyp", rename to .m4a if needed)
     b. convertAudioToPCM(): AVAudioFile -> AVAudioConverter -> Float32 PCM at 16000 Hz mono
        - Files > 10 seconds: parallel dispatch (one worker per activeProcessorCount)
        - Files <= 10 seconds: sequential (1M frame chunk)
        - Channel mix: RMS energy per channel, active channels averaged (normalization = 1/activeChannels)
     c. Build WhisperFullParams from Settings (see params section)
     d. Set C abort callback (UnsafeMutablePointer<Bool>, polled by whisper.cpp)
     e. Set C progress callback (maps whisper 0-100% to Swift 10%-95%)
     f. context.full(samples:params:) -- blocking whisper.cpp inference call
     g. Iterate nSegments, collect text (with timestamps if showTimestamps)
     h. Strip "[MUSIC]" and "[BLANK_AUDIO]" artifacts
     i. Apply AutocorrectWrapper.format() for Asian languages if useAsianAutocorrect

WhisperFullParams (WhisperFullParams.swift) -- key fields:
  strategy: .greedy | .beamSearch  (default .greedy)
  nThreads: Int32  (default 1, runtime sets to min(activeProcessorCount, 8), min 2)
  noTimestamps: Bool  (default false)
  suppressBlank: Bool  (default true)
  translate: Bool  (default false, translateToEnglish setting)
  language: String?  (nil = auto-detect off; set from selectedLanguage if not "auto")
  detectLanguage: Bool  (default false -- NOTE: code comment says "only detects, does not process")
  temperature: Float  (default 0.0)
  noSpeechThold: Float  (default 0.6)
  initialPrompt: String?  (THE CUSTOM VOCABULARY MECHANISM -- see below)
  beamSearchBeamSize: Int32  (used when strategy=.beamSearch)
  grammarRules: array  (grammar-constrained decoding, advanced)
  abortCallback: C function pointer (polled to allow cancellation mid-inference)
  progressCallback: C function pointer (whisper calls this with 0-100 int progress)

Initial prompt / custom vocabulary mechanism:
  Field: WhisperFullParams.initialPrompt: String?
  Maps to whisper.cpp's initial_prompt parameter
  Mechanism: prepended as a "fake" prior transcript to bias the model's token predictions
  Effect: model tends to produce spellings and terms that appear in the initial prompt
  UI location: Settings -> Transcription tab -> "Initial Prompt" TextEditor (60px height)
  Persistence: AppPreferences.shared.initialPrompt (UserDefaults)
  Used in video: Kun pastes project names / tool names ("lavish axi", "treehouse", etc.) here to
    improve transcription accuracy for domain-specific vocabulary
  AUTHOR DESIGN CLAIM: "This is what makes the transcription really good" for domain terms
  Limitation: This is a BIAS, not enforcement -- whisper still uses its LM, so rare/unusual
    spellings may not hold for every utterance. Not a grammar constraint.

Audio target format (verified from WhisperEngine.makeTargetFormat):
  PCM Float32, 16000 Hz, non-interleaved
  Channel count: preserved from source (multi-channel mixed to mono-equivalent by appendMixedSamples)
  Sample rate converter quality: AVAudioQuality.medium

Model format (whisper engine):
  ggml .bin files (whisper.cpp quantized format), downloaded from HuggingFace
  Default downloadable models (from Settings.swift:473-496):
    "Turbo V3 large"  -- ggml-large-v3-turbo.bin         1624 MB  high accuracy
    "Turbo V3 medium" -- ggml-large-v3-turbo-q8_0.bin    874 MB   balanced
    "Turbo V3 small"  -- ggml-large-v3-turbo-q5_0.bin    574 MB   fastest
  Download source: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/<filename>
  Models directory: WhisperModelManager.shared.modelsDirectory (application support subdir)

Model format (FluidAudio / Parakeet engine):
  FluidAudio Swift package (external dependency, not in Package.resolved -- likely binary XCFramework or SPM)
  AsrModels.downloadAndLoad(version:) -- async download to ~/Library/Application Support/FluidAudio/Models/<version>/
  Available: Parakeet v3 (multilingual, 25 languages), Parakeet v2 (English-only, higher recall)
  AsrManager(config: .default).initialize(models:) -- loads model for inference

Settings (AppPreferences.shared keys):
  selectedEngine: String  ("whisper" | "fluidaudio")
  selectedWhisperModelPath: String?  (path to .bin file)
  whisperLanguage: String  (ISO language code or "auto")
  translateToEnglish: Bool
  suppressBlankAudio: Bool
  showTimestamps: Bool
  temperature: Double  (0.0-1.0 slider)
  noSpeechThreshold: Double  (0.0-1.0 slider)
  initialPrompt: String  (EMPTY means nil passed to whisper -- no bias)
  useBeamSearch: Bool
  beamSize: Int  (1-10 stepper)
  debugMode: Bool
  autoCopyToClipboard: Bool
  autoPasteTranscription: Bool
  holdToRecord: Bool
  modifierOnlyHotkey: String (ModifierKey rawValue)
  addSpaceAfterSentence: Bool
  useAsianAutocorrect: Bool
  fluidAudioModelVersion: String ("v2" | "v3")

Shortcut/trigger mechanism:
  KeyboardShortcuts library for key combination hotkey (ShortcutManager.swift)
  ModifierKeyMonitor.swift: CGEvent tap monitors modifier-only keys (requires Input Monitoring permission)
  holdToRecord: hold = record, release = stop; toggle = tap to start, tap to stop

Transcription output path:
  Recording.recordingsDirectory: application support subdir
  GRDB SQLite: stores transcription history (Recording.swift model)
  autoPasteTranscription: simulates CGEvent keyboard paste into focused app (FocusUtils.swift / ClipboardUtil.swift)

Tests:
  OpenSuperWhisperTests/OpenSuperWhisperTests.swift  (minimal scaffold)
  OpenSuperWhisperUITests/  (launch test only)
  test_audio.m4a  (real audio file in test bundle)
  No substantive unit tests verified in source

Platform constraints (CRITICAL for ELAI reuse):
  macOS ONLY: AppKit, NSWorkspace, AVFoundation, Carbon, CGEvent, CoreAudio -- none available on Windows or Linux
  No Windows build, no Linux build, no cross-platform path
  SwiftUI + Xcode project -- requires macOS toolchain to build
  whisper.cpp C bindings wrapped in Swift (Bridge.h) -- the core inference code IS portable as whisper.cpp,
    but the entire UI/hotkey/clipboard layer is macOS-specific
  FluidAudio engine: binary dependency, macOS only (no source visible)
  Keyboard shortcut monitoring requires Input Monitoring entitlement (macOS permission)
  Auto-paste requires Accessibility entitlement

Portability verdict for ELAI (win32 + remote Linux sidecar):
  NOT directly portable. The whisper.cpp inference core is portable (C++), but everything
  wrapping it (Swift, AppKit, AVFoundation, CGEvent, FluidAudio) is macOS-specific.
  To replicate the capability on Windows/Linux: use whisper.cpp directly (C++/Python bindings),
    or faster-whisper (Python, CTranslate2) with the same initial_prompt parameter,
    or openai-whisper Python package with initial_prompt= kwarg.
  The initial_prompt mechanism is standard whisper.cpp and openai-whisper, not unique to this app.

Gotchas:
  FluidAudio package not in Package.resolved -- it's a separate dependency resolution path (possibly SPM via Xcode)
  detectLanguage=false in code DESPITE language="auto" -- comment says "only detects and does not process"; actual auto-detect uses language=nil
  cancelTranscription(): sets abort flag pointer directly, polled by whisper.cpp C callback -- not Swift concurrency cancellation
  Parallel audio conversion writes to pre-allocated output array with manual index arithmetic -- race condition possible if ratio calculation is off by > 1024 samples
  Asian autocorrect applied AFTER whisper result, not as a constraint during inference
  "[MUSIC]" and "[BLANK_AUDIO]" stripped as literal strings -- whisper.cpp hallucinations not fully suppressed

Critical assertions (testable):
  assert settings.initialPrompt.isEmpty == False or params.initialPrompt == nil   # empty prompt must pass nil, not ""
  assert targetFormat.sampleRate == 16000   # whisper.cpp requires 16kHz
  assert targetFormat.commonFormat == .pcmFormatFloat32   # whisper.cpp requires float32
  assert nThreads >= 2   # min 2 enforced in WhisperEngine.transcribeAudio line 114
  assert abs(normalizedProgress - (0.10 + whisperProgress/100 * 0.85)) < 0.001   # progress mapping

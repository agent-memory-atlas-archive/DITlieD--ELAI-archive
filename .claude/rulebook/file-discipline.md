File discipline and project structure (Swift / iOS apps, 2026 baseline)

Loaded for ALL coding tasks. Prevents monolith drift and lazy generation.

Source: Gemini Deep Research 2026 (docs/researches/6r6rfile_summary.txt §3).


File-length caps (WARN / BLOCK in LOC)

SwiftUI Views          150 / 250
UIKit View Controllers 250 / 400
View Models            150 / 250
Models                  80 / 150
Services               150 / 250
Utility Extensions     100 / 180
Test Files             300 / 500
Package Manifests      100 / 200

WARN: auditor flags, continue. BLOCK: reject write, split first.


SwiftUI body limit

40 LOC. Extract subviews on distinct logical blocks / conditional branches / deep modifier chains.
@ViewBuilder limit: 10 direct children per block. Workaround = Group {} or extracted properties.


Cyclomatic complexity per function

8 warn / 12 block. SwiftLint cyclomatic_complexity with ignores_case_statements: true.


Type fan-out limits

Max 10 public methods per type.
Max 8 stored properties per type.
Type body length: 200 warn / 300 block.


SPM modularity

Split when: feature self-contained, change triggers cross-module recompile, platform layer separable.
Static SPM targets default (faster pre-main, no dyld cost).
Static libs for perf-critical low-level only.
Dynamic frameworks only for app-extension / resource sharing — adds pre-main cost.


Folder structure (hybrid)

ProjectRoot/
  App/
  Core/
    Network/
    Database/
  Services/
    Analytics/
    Push/
  Features/
    <Feature>/
      Views/
      ViewModels/
      Models/

Scale 10 > 100 features = migrate folders to local SPM modules + .xcconfig for unified build settings.


SwiftLint config (key rules)

file_length 300/500
type_body_length 200/300
function_body_length 40/60
line_length 140/180
cyclomatic_complexity 8/12 (ignores_case_statements)
function_parameter_count 4/6
large_tuple 3/4

Custom rules:
- max 5 @Published per VM (warn)
- ban static let shared = T() (warn — singleton discipline below)


ViewModel anti-patterns

God-object signals: multiple repos, unrelated subview state, > 5 @Published.
VMs MUST NOT instantiate / retain other VMs. Propagate intent via closures, delegates, async streams to a coordinator.
Consolidate @Published into a single ViewState struct / enum.


Singleton discipline

System singletons (UIApplication.shared, NotificationCenter.default, FileManager.default) OK.
Custom app-level singletons banned by default.
Exception protocol:
  1. Prove uniqueness (hardware / file conflict).
  2. Abstract protocol interface.
  3. DI through initializer; never reach .shared inside other types.


Test discipline

Naming: test_[Condition]_[ExpectedOutcome] or sentence-style.
Max 15 cases per test class — split by scenario (success / failure).
AAA pattern (Arrange / Act / Assert).
setUpWithError / tearDownWithError; sut = nil in teardown.


Post-write checks (enforced by Stop hook)

Functions >10 lines: re-read, grep for TODO / fatalError / preconditionFailure / ! / try! / as!.
New file: verify Swift module / target membership (Package.swift product, or Xcode target).
Modified file: confirm it still compiles (xcodebuild -scheme X -configuration Debug build, or swift build).


Cross-platform overlay (Expo / RN)

TS/TSX caps (until coding-standards-expo.md takes precedence):
- TSX component: 200 warn / 350 block
- Hook (useXxx.ts): 100 warn / 200 block
- Cyclomatic complexity: 8 warn / 12 block (ESLint complexity rule)
- Max props per component: 8 (deeper = split)

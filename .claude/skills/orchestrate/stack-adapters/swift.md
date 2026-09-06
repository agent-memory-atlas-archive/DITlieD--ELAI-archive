STACK ADAPTER — swift (xcodebuild / SPM / iOS)

Loaded by orchestrate when detect-stack.ps1 reports stack="swift" (Xcode project + Package.swift).

STATUS: STUB. The apps_workflow primary target for Swift is hirer-side builds (a local Mac was not available in the historical setup), so this adapter is intentionally minimal. Will be filled when a Swift-native gig requires it OR when a local Mac mini joins the toolchain.

WHAT THIS ADAPTER WILL DO (when filled)

Tools required:
- xcodebuild (macOS only — runs hirer-side or on cloud Mac via EAS / Codemagic / Xcode Cloud)
- swift package (SPM commands)
- swiftlint (per file-discipline.md SwiftLint config)
- periphery (https://github.com/peripheryapp/periphery) — Swift dead-code / unused-export detection
- xcrun simctl io booted screenshot (for UI verification fallback when no real iPhone)

GENERIC 6 GATES (Swift flavor — to be filled)

GATE 1 BUILD GREEN  > xcodebuild -scheme <Scheme> -configuration Debug build
GATE 2 TESTS GREEN  > xcodebuild test -scheme <Scheme> + delta coverage via grep
GATE 3 DEP GRAPH    > swift package show-dependencies (cycle check) + periphery scan (orphans)
GATE 4 PUBLIC API   > no first-party Swift tool equivalent to cargo-public-api; use git diff on *.swift + check for new `public` symbols having consumers via periphery
GATE 5 MANIFEST     > Package.resolved consistency check via swift package show-dependencies + xcodebuild -showBuildSettings | grep MARKETING_VERSION
GATE 6 DOCS DRIFT   > rg new public symbols in memory-bank changelog / systemPatterns

EXTRA SWIFT GATES (per anti-stale-swift-ios.md)

S1 BANNED PATTERNS  > rg "\.unwrap\(|try!|as!" --type swift  (force-unwrap / force-try / force-cast banned)
S2 SWIFTLINT CLEAN  > swiftlint --strict (treats warnings as errors)
S3 PRIVACY MANIFEST > if plan touches data-access APIs: verify PrivacyInfo.xcprivacy updated with Required Reason codes per app-store-compliance.md
S4 NO @MAINACTOR DRIFT > rg "@MainActor" --type swift in test of cross-isolation violations

IMPLEMENTATION DEFERRED until a Swift project actually orchestrates. The stub exists so detect-stack.ps1 doesnt error on Swift detection.


END stack-adapters/swift.md

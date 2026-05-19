# B-15A Task 10 / C-14 — Determinism proof harness (Windows-native PowerShell sibling).
#
# Functionally identical to determinism_proof.sh but uses only PowerShell 7+
# primitives so it runs in cmd.exe / pwsh without git-bash, MSYS, or WSL.
# Asserts byte-identical filtered stdout across two runs of the orchestrator's
# `bsp_determinism` integration test under a fixed seed.
#
# Usage:
#   pwsh -File determinism_proof.ps1 [-Help]
# Environment:
#   ELAI_DETERMINISM_SEED — fixed seed for the test (default: 0xB15A)
#
# Exits 0 on byte-identical filtered output across the two runs.
# Exits 1 on divergence or test failure.

[CmdletBinding()]
param(
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

function Show-Usage {
    @'
Usage: determinism_proof.ps1 [-Help]

Runs `cargo test -p elai-orchestrator --test bsp_determinism` twice with a
fixed seed, captures stdout, and asserts byte-identical filtered output
across runs.

Exits 0 on match, 1 on divergence or test failure.

Environment:
  ELAI_DETERMINISM_SEED  fixed seed for the test (default: 0xB15A)
'@ | Write-Output
}

if ($Help) {
    Show-Usage
    exit 0
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$CrateRoot = Resolve-Path (Join-Path $ScriptDir '..')
$Seed = if ($env:ELAI_DETERMINISM_SEED) { $env:ELAI_DETERMINISM_SEED } else { '0xB15A' }

$OutA = [System.IO.Path]::GetTempFileName()
$OutB = [System.IO.Path]::GetTempFileName()

try {
    function Invoke-Run([string]$OutFile) {
        $env:ELAI_DETERMINISM_SEED = $Seed
        $cmd = 'cargo test -p elai-orchestrator --test bsp_determinism -- --nocapture'
        # Capture stdout+stderr, write to $OutFile, check exit code.
        Push-Location $CrateRoot
        try {
            & cmd /c "$cmd > `"$OutFile`" 2>&1"
            $code = $LASTEXITCODE
        } finally {
            Pop-Location
        }
        if ($code -ne 0) {
            Write-Error "determinism_proof: cargo test failed (exit=$code); output below:"
            Get-Content $OutFile | Write-Error
            return $false
        }
        return $true
    }

    Write-Output "determinism_proof: run 1 with seed=$Seed"
    if (-not (Invoke-Run $OutA)) { exit 1 }

    Write-Output "determinism_proof: run 2 with seed=$Seed"
    if (-not (Invoke-Run $OutB)) { exit 1 }

    # Apply the same filter as the POSIX script so timing/path noise does not
    # break the byte-compare.
    function Read-Filtered([string]$Path) {
        Get-Content $Path | ForEach-Object {
            $line = $_
            if ($line -match 'finished in') { return }
            if ($line -match 'running [0-9]+ test') { return }
            $line = $line -replace '[0-9]+ms', 'Nms'
            $line = $line -replace '[0-9]+\.[0-9]+s', 'N.Ns'
            $line
        }
    }

    function Compute-Sha256([string]$Path) {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes(((Read-Filtered $Path) -join "`n"))
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            ($sha.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') }) -join ''
        } finally {
            $sha.Dispose()
        }
    }

    $HashA = Compute-Sha256 $OutA
    $HashB = Compute-Sha256 $OutB

    Write-Output "determinism_proof: hash_a=$HashA"
    Write-Output "determinism_proof: hash_b=$HashB"

    if ($HashA -eq $HashB) {
        Write-Output 'determinism_proof: PASS (byte-identical filtered output across 2 runs)'
        exit 0
    }

    Write-Error 'determinism_proof: FAIL — runs diverged.'
    $diff = Compare-Object (Read-Filtered $OutA) (Read-Filtered $OutB) | Select-Object -First 20
    $diff | Format-Table | Out-String | Write-Error
    exit 1
} finally {
    Remove-Item -Force -ErrorAction SilentlyContinue $OutA, $OutB
}

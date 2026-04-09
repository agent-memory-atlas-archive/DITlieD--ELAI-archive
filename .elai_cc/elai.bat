@echo off
REM ELAI Local Mode — runs the full CLI against local Gemma4 E2B with DoLa + TurboQuant.
REM Routes through ConversationRuntime with FSM tool filtering + ContextBudget.

"%~dp0target\release\elai.exe" --local %*

# AI Decision Record: CLI Interface, Argument Parsing, Exit Code Orchestration, and System Viewer

## Context & Problem Statement
 needs a user-facing CLI binary that parses command-line arguments, orchestrates pre-flight validation, handles the entire pipeline (PDFium loading, text extraction, sequence diffing, canvas layout computation, 4-layer vector compositing), launches an optional system viewer, and maps outcomes to standard 3-tier exit codes (, , ).

## Architectural Decisions
1. **Clap 4.x Argument Parsing & Default Naming**:
   - Implemented  with strongly typed enum parsing for  and .
   - Default output resolution dynamically constructs  if  is omitted.
   - Header height calculation dynamically respects .

2. **3-Tier Exit Code Contract**:
   - Exit Code : Documents are structurally and textually identical (no diff highlights generated).
   - Exit Code : Documents differ (differences detected and visual diff rendered).
   - Exit Code : Error during validation, execution, PDFium loading, text extraction, or composition.

3. **Pre-flight & Error Orchestration**:
   - Verified existence of input files before calling into PDFium C API.
   - Guarded against inadvertent file overwriting unless  is specified.
   - Validated that at least one document contains a searchable text layer, returning  when scanned/empty PDFs are supplied.

4. **Non-fatal System Viewer Launch**:
   - Implemented  using the  crate, logging a warning to stderr if the system viewer fails to launch without aborting or altering exit code logic.

## Verification & Validation
- Unit & integration tests for CLI flags (, , missing args), pre-flight file checks, identical vs differing documents, empty text layers, and corrupt PDF handling.
- Achieved **90.11% overall line coverage** ( at 93.29%,  at 100%).

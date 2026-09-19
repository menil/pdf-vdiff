# AI Decision Record: Nix Flake Support & Modular Packaging

## Context & Goal
- Provide standalone and reproducible building, execution, and integration of `pdf-vdiff` via Nix Flakes across macOS (`aarch64-darwin`, `x86_64-darwin`) and Linux (`aarch64-linux`, `x86_64-linux`).
- Enable other repositories to include `pdf-vdiff` as an input dependency (`nix run`, `nix profile install`, `devShells`, `home-manager`).
- Seamlessly package Google's PDFium C dynamic shared library while preserving macOS SIP resistance and NixOS dynamic linker resolution.

## Architecture & Key Decisions
1. **Modular Derivations (`nix/pdfium.nix`, `nix/package.nix`)**:
   - Separated PDFium extraction and Rust package definition into standalone `callPackage` derivations.
   - `nix/pdfium.nix` handles platform-specific tarball extraction and applies `autoPatchelfHook` on Linux so `libpdfium.so` dynamically resolves `libc` and `libstdc++` on NixOS.
   - `nix/package.nix` scopes Rust sources using `lib.fileset` to prevent unrelated docs/fixtures from invalidating the compilation cache.
2. **First-Class Nixpkgs Overlay (`overlays.default`)**:
   - Exported `overlays.default = final: prev: { ... }` using `final.callPackage`, allowing downstream consumers (Home Manager, NixOS modules) to cleanly inject `pdf-vdiff` into their own Nixpkgs fixpoints without closed-loop anti-patterns.
3. **Executable Wrapping with `makeWrapper`**:
   - Injected `PDFIUM_LIB_DIR` and `PDFIUM_LIB_PATH` as defaults alongside `LD_LIBRARY_PATH` and `DYLD_LIBRARY_PATH`. This avoids relying solely on `DYLD_LIBRARY_PATH` (which macOS System Integrity Protection scrubs across process boundaries).
4. **Single-Threaded Test Flag Propagation**:
   - Explicitly configured `checkFlags = [ "--test-threads=1" ]` to serialize `cargoCheckHook` tests, preventing data races in PDFium's global C library state.
5. **Deduplicated Developer Shell (`shell.nix`)**:
   - Refactored `shell.nix` to call `pkgs.callPackage ./nix/pdfium.nix {}`, eliminating ~74 lines of duplicated derivation code while maintaining full compatibility with non-flake `nix-shell`.

## Alternatives Considered & Rejected
- **Monolithic In-Flake Derivation**: Kept all packaging logic inside `flake.nix`. Rejected because it led to massive code duplication with `shell.nix` and prevented downstream users from using standard `pkgs.callPackage`.
- **Closed Overlay Pattern (`self.packages.${final.system}`)**: Direct indexing into flake outputs in the overlay. Rejected because it bypasses the consumer's Nixpkgs fixpoint and breaks cross-compilation contexts.
- **Unfiltered Source Root (`src = ./.`)**: Passing `./.` directly to `buildRustPackage`. Rejected because any documentation, `.md`, or fixture modification resulted in unnecessary re-compilation of the entire Rust codebase and its dependencies.

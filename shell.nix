let
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  lockedNixpkgs = lock.nodes.nixpkgs.locked;
  defaultPkgs = import (fetchTarball {
    url = "https://github.com/${lockedNixpkgs.owner}/${lockedNixpkgs.repo}/archive/${lockedNixpkgs.rev}.tar.gz";
    sha256 = lockedNixpkgs.narHash;
  }) {};
in
{ pkgs ? defaultPkgs }:

let
  pdfium = pkgs.callPackage ./nix/pdfium.nix {};
in
pkgs.mkShell {
  packages = with pkgs; [
    git
    gh
    just
    jq
    rustc
    cargo
    clippy
    rustfmt
    cargo-llvm-cov
    # Use rustc's paired LLVM rather than the default system LLVM so
    # llvm-cov and llvm-profdata always match the .profraw format version
    # emitted by rustc -C instrument-coverage.
    rustc.llvmPackages.llvm
    pkg-config
    pdfium
  ];

  shellHook = ''
    export LLVM_COV="${pkgs.rustc.llvmPackages.llvm}/bin/llvm-cov"
    export LLVM_PROFDATA="${pkgs.rustc.llvmPackages.llvm}/bin/llvm-profdata"
    export PDFIUM_LIB_DIR="${pdfium}/lib"
    export PDFIUM_LIB_PATH="${pdfium}/lib/libpdfium.${pdfium.ext}"
    export LD_LIBRARY_PATH="${pdfium}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export DYLD_LIBRARY_PATH="${pdfium}/lib''${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
    echo "❄️ Welcome to the pdf-vdiff development shell!"
  '';
}

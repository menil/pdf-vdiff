{ pkgs ? import <nixpkgs> {} }:

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
    llvmPackages.llvm
    pkg-config
    pdfium
  ];

  shellHook = ''
    export LLVM_COV="${pkgs.llvmPackages.llvm}/bin/llvm-cov"
    export LLVM_PROFDATA="${pkgs.llvmPackages.llvm}/bin/llvm-profdata"
    export PDFIUM_LIB_DIR="${pdfium}/lib"
    export PDFIUM_LIB_PATH="${pdfium}/lib/libpdfium.${pdfium.ext}"
    export LD_LIBRARY_PATH="${pdfium}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export DYLD_LIBRARY_PATH="${pdfium}/lib''${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
    echo "❄️ Welcome to the pdf-vdiff development shell!"
  '';
}

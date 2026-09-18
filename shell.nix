{ pkgs ? import <nixpkgs> {} }:

let
  pdfiumVersion = "7881";

  pdfiumPlatform =
    if pkgs.stdenv.isDarwin && pkgs.stdenv.isAarch64 then {
      name = "pdfium-mac-arm64.tgz";
      sha256 = "52e94ca5aa8847934330daf3f8150c190682c5ca93831468794f8b90d4392e40";
      ext = "dylib";
    }
    else if pkgs.stdenv.isDarwin then {
      name = "pdfium-mac-x64.tgz";
      sha256 = "6dedf83990e0e3d6b7c93c9e7589c5a126b0ae14b7464d76120cff7a26afb18b";
      ext = "dylib";
    }
    else if pkgs.stdenv.isAarch64 then {
      name = "pdfium-linux-arm64.tgz";
      sha256 = "ee7f7b7d5468958336a818c1cd580bdd20972846b7377b13f9a923d92d1d4674";
      ext = "so";
    }
    else {
      name = "pdfium-linux-x64.tgz";
      sha256 = "1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d";
      ext = "so";
    };

  pdfium = pkgs.stdenv.mkDerivation {
    pname = "pdfium";
    version = pdfiumVersion;

    src = pkgs.fetchurl {
      url = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/${pdfiumVersion}/${pdfiumPlatform.name}";
      sha256 = pdfiumPlatform.sha256;
    };

    sourceRoot = ".";

    installPhase = ''
      mkdir -p $out/lib $out/include
      cp -r lib/* $out/lib/
      if [ -d include ]; then
        cp -r include/* $out/include/
      fi
    '';
  };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
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
    export PDFIUM_LIB_PATH="${pdfium}/lib/libpdfium.${pdfiumPlatform.ext}"
    export LD_LIBRARY_PATH="${pdfium}/lib:$LD_LIBRARY_PATH"
    export DYLD_LIBRARY_PATH="${pdfium}/lib:$DYLD_LIBRARY_PATH"
    echo "❄️ Welcome to the pdf-vdiff development shell!"
  '';
}

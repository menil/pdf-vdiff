{ lib, rustPlatform, makeWrapper, pdfium }:

let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
      ../tests
      ../README.md
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    makeWrapper
  ];

  buildInputs = [
    pdfium
  ];

  checkFlags = [
    "--test-threads=1"
  ];

  preCheck = ''
    export PDFIUM_LIB_DIR="${pdfium}/lib"
    export PDFIUM_LIB_PATH="${pdfium}/lib/libpdfium.${pdfium.ext}"
    export LD_LIBRARY_PATH="${pdfium}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export DYLD_LIBRARY_PATH="${pdfium}/lib''${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
  '';

  postInstall = ''
    wrapProgram $out/bin/pdf-vdiff \
      --set-default PDFIUM_LIB_DIR "${pdfium}/lib" \
      --set-default PDFIUM_LIB_PATH "${pdfium}/lib/libpdfium.${pdfium.ext}" \
      --prefix LD_LIBRARY_PATH : "${pdfium}/lib" \
      --prefix DYLD_LIBRARY_PATH : "${pdfium}/lib"
  '';

  meta = with lib; {
    description = cargoToml.package.description or "A fast, standalone CLI tool for side-by-side visual PDF diffing";
    homepage = "https://github.com/menil/pdf-vdiff";
    license = licenses.mit;
    platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    mainProgram = "pdf-vdiff";
  };
}

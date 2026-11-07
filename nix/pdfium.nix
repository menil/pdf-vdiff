{ lib, stdenv, fetchurl, autoPatchelfHook ? null }:

let
  pdfiumVersion = "8125";

  pdfiumPlatform =
    if stdenv.hostPlatform.isDarwin && stdenv.hostPlatform.isAarch64 then {
      name = "pdfium-mac-arm64.tgz";
      sha256 = "4939bd935ac83ddc09a3a4f038bc43a19cea1660316a3667f26cb07a070e139d";
      ext = "dylib";
    }
    else if stdenv.hostPlatform.isDarwin && stdenv.hostPlatform.isx86_64 then {
      name = "pdfium-mac-x64.tgz";
      sha256 = "8a5193535b80c7215d8932f6647ae5097465b6dba2811602c21e120c2af112b0";
      ext = "dylib";
    }
    else if stdenv.hostPlatform.isLinux && stdenv.hostPlatform.isAarch64 then {
      name = "pdfium-linux-arm64.tgz";
      sha256 = "8b8818f2ec57d1b213d952ca6d7f0ffeca9d0110487d0e20ad83ba918bc92efa";
      ext = "so";
    }
    else if stdenv.hostPlatform.isLinux && stdenv.hostPlatform.isx86_64 then {
      name = "pdfium-linux-x64.tgz";
      sha256 = "e1c3ce6e270cb5e0c20816b379ad9f2c1309972697c505f508de92bf5e7d6659";
      ext = "so";
    }
    else throw "Unsupported platform for pdfium: ${stdenv.hostPlatform.system}";
in
stdenv.mkDerivation {
  pname = "pdfium";
  version = pdfiumVersion;

  src = fetchurl {
    url = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/${pdfiumVersion}/${pdfiumPlatform.name}";
    sha256 = pdfiumPlatform.sha256;
  };

  sourceRoot = ".";

  nativeBuildInputs = lib.optionals stdenv.hostPlatform.isLinux (
    lib.optional (autoPatchelfHook != null) autoPatchelfHook
  );

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    stdenv.cc.cc.lib
  ];

  installPhase = ''
    mkdir -p $out/lib
    if [ -d lib ]; then
      cp -r lib/* $out/lib/
    fi
  '';

  passthru = {
    ext = pdfiumPlatform.ext;
    version = pdfiumVersion;
  };

  meta = with lib; {
    description = "Google's PDFium library binaries";
    homepage = "https://github.com/bblanchon/pdfium-binaries";
    license = licenses.asl20;
    platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
  };
}

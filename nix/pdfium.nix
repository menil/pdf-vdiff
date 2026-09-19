{ lib, stdenv, fetchurl, autoPatchelfHook ? null }:

let
  pdfiumVersion = "7881";

  pdfiumPlatform =
    if stdenv.hostPlatform.isDarwin && stdenv.hostPlatform.isAarch64 then {
      name = "pdfium-mac-arm64.tgz";
      sha256 = "52e94ca5aa8847934330daf3f8150c190682c5ca93831468794f8b90d4392e40";
      ext = "dylib";
    }
    else if stdenv.hostPlatform.isDarwin && stdenv.hostPlatform.isx86_64 then {
      name = "pdfium-mac-x64.tgz";
      sha256 = "6dedf83990e0e3d6b7c93c9e7589c5a126b0ae14b7464d76120cff7a26afb18b";
      ext = "dylib";
    }
    else if stdenv.hostPlatform.isLinux && stdenv.hostPlatform.isAarch64 then {
      name = "pdfium-linux-arm64.tgz";
      sha256 = "ee7f7b7d5468958336a818c1cd580bdd20972846b7377b13f9a923d92d1d4674";
      ext = "so";
    }
    else if stdenv.hostPlatform.isLinux && stdenv.hostPlatform.isx86_64 then {
      name = "pdfium-linux-x64.tgz";
      sha256 = "1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d";
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

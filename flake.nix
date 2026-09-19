{
  description = "A fast, standalone CLI tool for side-by-side visual PDF diffing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      overlay = final: prev: {
        pdfium = final.callPackage ./nix/pdfium.nix { };
        pdf-vdiff = final.callPackage ./nix/package.nix { };
      };
    in
    {
      overlays.default = overlay;
    } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ overlay ];
        };
      in
      {
        packages = {
          default = pkgs.pdf-vdiff;
          inherit (pkgs) pdf-vdiff pdfium;
        };

        checks = {
          default = pkgs.pdf-vdiff;
          inherit (pkgs) pdf-vdiff;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = pkgs.pdf-vdiff;
          };
          pdf-vdiff = flake-utils.lib.mkApp {
            drv = pkgs.pdf-vdiff;
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ pkgs.pdf-vdiff ];

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
          ];

          shellHook = ''
            export LLVM_COV="${pkgs.llvmPackages.llvm}/bin/llvm-cov"
            export LLVM_PROFDATA="${pkgs.llvmPackages.llvm}/bin/llvm-profdata"
            export PDFIUM_LIB_DIR="${pkgs.pdfium}/lib"
            export PDFIUM_LIB_PATH="${pkgs.pdfium}/lib/libpdfium.${pkgs.pdfium.ext}"
            export LD_LIBRARY_PATH="${pkgs.pdfium}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            export DYLD_LIBRARY_PATH="${pkgs.pdfium}/lib''${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
            echo "❄️ Welcome to the pdf-vdiff development shell!"
          '';
        };
      }
    );
}

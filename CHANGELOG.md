# Changelog

## [0.1.1](https://github.com/menil/pdf-vdiff/compare/pdf-vdiff-v0.1.0...pdf-vdiff-v0.1.1) (2026-09-21)


### Features

* **assets:** add generator binary and output man/completion files ([c5d0113](https://github.com/menil/pdf-vdiff/commit/c5d01131b345b905045516b664fa6f8a57b04fe7))
* **cli:** add clap_complete and clap_mangen dependencies and flags ([a0146f4](https://github.com/menil/pdf-vdiff/commit/a0146f40bf66d5306af4c9ba9abe4b460177a02c))
* **cli:** implement CLI interface, exit codes, and viewer launcher ([3091cdb](https://github.com/menil/pdf-vdiff/commit/3091cdb436819bca329d6df426b7e11f3d51f99f))
* **cluster:** implement spatial clustering and reading-order sort ([2eec315](https://github.com/menil/pdf-vdiff/commit/2eec3155c3cf2431af075f8f78402437b673bb11))
* **diff:** implement hierarchical 2-pass sequence diff engine ([8325155](https://github.com/menil/pdf-vdiff/commit/832515541300ebd3cdfca0f7af0d26319db1e466))
* isolate punctuation changes in diff highlighting ([2441752](https://github.com/menil/pdf-vdiff/commit/2441752232ad1508ed9115cee01865b69a57331d))
* **layout:** implement canvas layout and coordinate projections ([1f32ce4](https://github.com/menil/pdf-vdiff/commit/1f32ce404e70e5aa98dfeda9314bcc36eeee08cf))
* **models:** implement core geometric models, themes, and errors ([a1da58f](https://github.com/menil/pdf-vdiff/commit/a1da58f5853832584cfb81f6e8a9664ae661a6f7))
* **nix:** add Nix Flake support and modular package derivations ([243c9a0](https://github.com/menil/pdf-vdiff/commit/243c9a09b955223d9faab165b2448ca1a0b2e725))
* **nix:** install man page and shell completions via package.nix ([a1ce0cd](https://github.com/menil/pdf-vdiff/commit/a1ce0cd94c0a8544a4717473984acee06bd768d0))
* **pdf:** implement PDFium extraction adapter and vector compositor ([6172569](https://github.com/menil/pdf-vdiff/commit/6172569fc52a5539b4a1c6cf491b9278092dd32f))
* render tight text-only highlights in compositor ([2b19ad5](https://github.com/menil/pdf-vdiff/commit/2b19ad52f986f7f78ae2f0c5f467b3eb00836bf3))
* **scaffold:** initialize Rust toolchain and Cargo layout ([734b529](https://github.com/menil/pdf-vdiff/commit/734b5293dd36ff70eb79dd64d64fe5d472a9378d))
* stream word tokens directly in diff engine ([ee90ecb](https://github.com/menil/pdf-vdiff/commit/ee90ecbfcdf9687493eb383243df0c31c2916c82))
* **tests:** add synthetic fixtures, integration suite, and coverage ([e37bc3f](https://github.com/menil/pdf-vdiff/commit/e37bc3f282d21847e078c0424cea75be94d1b8f0))


### Bug Fixes

* **ci:** allow idempotent publish runs on unversioned main commits ([2edddc7](https://github.com/menil/pdf-vdiff/commit/2edddc7f91073975ba191802e68afe9f360d2048))
* **ci:** fix release trigger condition for published crate ([568ddd4](https://github.com/menil/pdf-vdiff/commit/568ddd4339569a8c35f24488e4fdaa5374cd7841))
* code scanning alert no. 1: Workflow does not contain permissions ([81e0597](https://github.com/menil/pdf-vdiff/commit/81e0597f115c679dbc33b4b9aed57d89b1ac91af))

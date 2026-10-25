# Structured host observations

Recorded output for issue #199. This static research appendix contains the
source pins, exact commands and observed outputs as JSON data; it is not an
executable runner or CI configuration. See the [reader guide](README.md) and
[qualification report](../../research/0053-native-omemo-reuse.md) for interpretation.

```json
{
  "schema": 1,
  "issue": 199,
  "result": "candidate-for-further-qualification-only",
  "client_date": "2026-10-24",
  "product_adoption": false,
  "product_runtime_sources_changed": false,
  "independent_bidirectional_interop": false,
  "netbsd_execution": false,
  "protocol_sce_complete": false,
  "host": {
    "system": "Darwin",
    "release": "27.0.0",
    "machine": "arm64"
  },
  "toolchain": {
    "compiler": "Apple clang version 21.0.0 (clang-2100.3.20.102)\nTarget: arm64-apple-darwin27.0.0\nThread model: posix\nInstalledDir: /Applications/Xcode-beta.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin",
    "make": "GNU Make 3.81\nCopyright (C) 2006  Free Software Foundation, Inc.\nThis is free software; see the source for copying conditions.\nThere is NO warranty; not even for MERCHANTABILITY or FITNESS FOR A\nPARTICULAR PURPOSE.\n\nThis program built for i386-apple-darwin11.3.0",
    "python_reference": "Python 3.13.15",
    "openssl": "3.6.3"
  },
  "picomemo": {
    "version": "1.3.1",
    "revision": "835d8277108b96c691fb6a992512f37315cb52d7",
    "archive_url": "https://codeload.github.com/mierenhoop/picomemo/tar.gz/835d8277108b96c691fb6a992512f37315cb52d7",
    "archive_sha256": "7b1519bb1edfd6a86d6595f6357270b6999a0393cafdb63d76257beb95647301",
    "source_files_sha256": {
      "LICENSE": "6927842639b360071cc6d24fb4b72351dc05e396ac8c549268a22337afd6acdc",
      "README.md": "ed5876f21b591842c68afffc022f9840eb701a6adb3ffd78798ba00feead7648",
      "Makefile": "b08b5420c2c3ec5d052cb425f1f3c64fecf110e290c56cd6d3ffd8abbfbcbcd6",
      "driver.h": "4847d58fd3388e73bbe911417753ff153e6fa9f5a99af55f549f2f8df57bc792",
      "omemo.h": "9eef3409554f1653493fa90d1a7c4ba80ce24701f4d452a31e50de7930757b98",
      "omemo.c": "bbcdf88d28cee63610b21867898558d7ecc404f674ae08be930c074a6237b95a",
      "gen/omemo2.c": "45e0ce3589cc779e47b73aee937b9324c23d5de68d4ca5184e846e3511593a9e",
      "gen/omemo2.h": "893fa655fe5f19d50cfa4813eaa52d1c94edead3fe33944e93cc5e28ab099f03",
      "hacl.c": "c204f117deb68ab30a204c052227384734d0f0424330487dde45f48792dca8e9",
      "c25519.c": "29a4269cf60a24cbac4dc5e2302bc04e3a56f782346dc390417ce4e8bda1cea1",
      "openssl.c": "4e4877dca3dd2ebff93fc6a8adbff117b67a1c28d665eea2721831c20c0583ce",
      "test/build.mk": "f9f99075014b3230b8c7dd85117d4ad6bd1535f0580d1ed619cd84d2de9898c8",
      "test/omemo.c": "fb236fea46e65ab464ba8b787191c088662760aad9b900d220bb70d2a4d7804e",
      "test/generate.c": "2c5abec5077ba62a332885629088b43d4f74771ba8c4172102b8508aa6954eac",
      "test/initsession.py": "b36402b95107a6b8e83b06559abcfe43fc8c273d267776a8305e43a9a1377426",
      ".github/SECURITY.md": "f85d3f081a028990d79e9042427cb5c49a20755dadc23b6d4a6498aada426bde"
    },
    "source_edits": false,
    "selected_namespace": "urn:xmpp:omemo:2",
    "selected_generated_api": "gen/omemo2.c / gen/omemo2.h",
    "native_source_js_ts_paths": []
  },
  "comparison_sources": {
    "prose-im/prose-core-client": {
      "revision": "1ef3752575e6d9cf0684dfec3b44f916258e0e83",
      "files": {
        "LICENSE.md": {
          "url": "https://raw.githubusercontent.com/prose-im/prose-core-client/1ef3752575e6d9cf0684dfec3b44f916258e0e83/LICENSE.md",
          "sha256": "8d40c261ec2a7585b2f0eae933ea5906d20f917b7f8a5e8cfa7baac279a0fc2c"
        },
        "crates/prose-core-client/Cargo.toml": {
          "url": "https://raw.githubusercontent.com/prose-im/prose-core-client/1ef3752575e6d9cf0684dfec3b44f916258e0e83/crates/prose-core-client/Cargo.toml",
          "sha256": "452798064144ae039b74cadf0da281fdca981309afb35cacd1e879d56154a916"
        },
        "crates/prose-xmpp/src/mods/omemo.rs": {
          "url": "https://raw.githubusercontent.com/prose-im/prose-core-client/1ef3752575e6d9cf0684dfec3b44f916258e0e83/crates/prose-xmpp/src/mods/omemo.rs",
          "sha256": "d42e8a595bd83c53e49e16286b7dddef74bbc8342247a500e73f63dc6275e675"
        }
      }
    },
    "signalapp/libsignal": {
      "revision": "e465b77cb63f4f61bb9df919ada60e5a298b1828",
      "files": {
        "LICENSE": {
          "url": "https://raw.githubusercontent.com/signalapp/libsignal/e465b77cb63f4f61bb9df919ada60e5a298b1828/LICENSE",
          "sha256": "0d96a4ff68ad6d4b6f1f30f713b18d5184912ba8dd389f86aa7710db079abcb0"
        },
        "rust/protocol/Cargo.toml": {
          "url": "https://raw.githubusercontent.com/signalapp/libsignal/e465b77cb63f4f61bb9df919ada60e5a298b1828/rust/protocol/Cargo.toml",
          "sha256": "1fc49d48f2f88d6900f0ba83215a2a0865cd0969a74b7efaf8fc65bd53a1cd79"
        }
      }
    },
    "dino/libomemo-c": {
      "revision": "1f0b37fee3ebefc46c32f882623481899a1ba568",
      "files": {
        "LICENSE": {
          "url": "https://raw.githubusercontent.com/dino/libomemo-c/1f0b37fee3ebefc46c32f882623481899a1ba568/LICENSE",
          "sha256": "c61f12da7cdad526bdcbed47a4c0a603e60dbbfdaf8b66933cd088e9132c303f"
        },
        "protobuf/meson.build": {
          "url": "https://raw.githubusercontent.com/dino/libomemo-c/1f0b37fee3ebefc46c32f882623481899a1ba568/protobuf/meson.build",
          "sha256": "5bf01dbc49f6c754168d1197fb00ce99a71f8a4bb2a202ef852f7c2a640ab7bb"
        },
        "src/signal_protocol.h": {
          "url": "https://raw.githubusercontent.com/dino/libomemo-c/1f0b37fee3ebefc46c32f882623481899a1ba568/src/signal_protocol.h",
          "sha256": "dda7bab42d54422f7045097b0e8d2a218f73cd002d06a0cd0b6d229c359a970d"
        }
      }
    }
  },
  "reference_packages": [
    {
      "name": "Twomemo",
      "version": "2.1.0",
      "wheel_url": "https://files.pythonhosted.org/packages/c5/f4/9ce83bc98c6cdf9df88f9a1949cb8d4f34dea7bf0ae7e34a7aebc8382e05/twomemo-2.1.0-py3-none-any.whl",
      "sha256": "dc799851d85f8badde8632511b5ad1091d9de95588a8d9c4d555764238f17f1a",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "Oldmemo",
      "version": "2.1.0",
      "wheel_url": "https://files.pythonhosted.org/packages/79/7d/ea851d1d967e7427153dec064b44143f52167d496c70c6251caa2b5f053f/oldmemo-2.1.0-py3-none-any.whl",
      "sha256": "8081164ff9d2d621624841efbb4b9b0b26ab59b32225ee0ca9351d4d265ca70a",
      "license_expression": "AGPL-3.0-only",
      "license_metadata": "not specified"
    },
    {
      "name": "DoubleRatchet",
      "version": "1.3.0",
      "wheel_url": "https://files.pythonhosted.org/packages/8e/50/d5769527876b1b2c2883eeb7307dd1509be1162e55b16799b55b3bd5b7b0/doubleratchet-1.3.0-py3-none-any.whl",
      "sha256": "a869c81aa175abd9d7cfcf0069a0fa1261116b2ff2684c1ac95a1b83b875b18d",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "OMEMO",
      "version": "2.1.0",
      "wheel_url": "https://files.pythonhosted.org/packages/d4/dd/3c2a55c6c38e560b47db5348b31db8b4dbd5e8f5c5ce89f421d16f6dd619/omemo-2.1.0-py3-none-any.whl",
      "sha256": "8319fae4a9764ff9038504384553862c49011d447353ab1890bf871349c146c0",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "X3DH",
      "version": "1.3.0",
      "wheel_url": "https://files.pythonhosted.org/packages/bb/f5/c18d9e408a3cf4eb4c7c568b1ef0507cf3a5938325aed8c1684084a3313a/x3dh-1.3.0-py3-none-any.whl",
      "sha256": "92548974e0cd9eccf19315f59014e5796ee4c5440b6fe753a9168fb92dd6c914",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "XEdDSA",
      "version": "1.2.0",
      "wheel_url": "https://files.pythonhosted.org/packages/04/63/f240e12a30d2006eafc188e57166fd74320c00efaae04ab79fc769ae0c5a/xeddsa-1.2.0-cp313-cp313-macosx_15_0_arm64.whl",
      "sha256": "b9912475590c0018a4323f7315490725c82b39243bf3a39922dad5d3422dcc9a",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "cffi",
      "version": "2.1.1",
      "wheel_url": "https://files.pythonhosted.org/packages/55/41/4c7042f317b9217502988f0873af87e16ad606dc20f84e546e3e6ce9764c/cffi-2.1.1-cp313-cp313-macosx_11_0_arm64.whl",
      "sha256": "19ee6127ee34de7d83ce3d371ebc5ed91addbdcc39f9ab15ce4eb35a4e534971",
      "license_expression": "MIT-0",
      "license_metadata": "not specified"
    },
    {
      "name": "cryptography",
      "version": "50.0.2",
      "wheel_url": "https://files.pythonhosted.org/packages/e5/56/d194340cc4a57535e82e1bee9e89667ac4b7c13b5d3f59686deae3094dd5/cryptography-50.0.2-cp311-abi3-macosx_11_0_arm64.whl",
      "sha256": "fa8f5efb344d6908a1ce62f4a24e2e5780f825d6f53f5f50ec5ffacac72936cb",
      "license_expression": "Apache-2.0 OR BSD-3-Clause",
      "license_metadata": "not specified"
    },
    {
      "name": "protobuf",
      "version": "7.36.2",
      "wheel_url": "https://files.pythonhosted.org/packages/32/72/98342feb672507c8f3a69e34b4fa8961f608edba5c1a48a6f47156d92cb5/protobuf-7.36.2-cp310-abi3-macosx_10_9_universal2.whl",
      "sha256": "cbc70b17ee27e28894c7fee8bb04be1abead49e936bc70eb60052531eee2079e",
      "license_expression": null,
      "license_metadata": "3-Clause BSD License"
    },
    {
      "name": "pydantic",
      "version": "2.14.1",
      "wheel_url": "https://files.pythonhosted.org/packages/35/ea/a56b9fe5066f3537b7882f77e9c5dfb26d8c2eefdaed9b5fc73d57b4dc22/pydantic-2.14.1-py3-none-any.whl",
      "sha256": "9195d967ec791692a04438115466764fb8b9a27b31f14a760437694f40d6b454",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "pydantic_core",
      "version": "2.50.1",
      "wheel_url": "https://files.pythonhosted.org/packages/f7/b9/c720e56858d4e1539503297ed37063e0c08e0f3541c41b777f6a800f75fa/pydantic_core-2.50.1-cp313-cp313-macosx_11_0_arm64.whl",
      "sha256": "0c003c3b7f49debb893d2d85ae099ac5959c9839e2f330fadb1fcdf7a6594482",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "annotated-types",
      "version": "0.8.0",
      "wheel_url": "https://files.pythonhosted.org/packages/99/91/8acff4f5e50511b911bbccb72b8628a49c68ce14148cd9f6431094859a90/annotated_types-0.8.0-py3-none-any.whl",
      "sha256": "f072f4d804ea359e4eaf198b1af7a8b0943881a87f31bb764f8bf219bb9419e0",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "typing_extensions",
      "version": "4.16.0",
      "wheel_url": "https://files.pythonhosted.org/packages/49/d3/b8441a820a491ddfc024b0b0cf0393375b75ea13866d9c66727e54c2fc80/typing_extensions-4.16.0-py3-none-any.whl",
      "sha256": "481caa481374e813c1b176ada14e97f1f67a4539ce9cfeb3f350d78d6370c2e8",
      "license_expression": "PSF-2.0",
      "license_metadata": "not specified"
    },
    {
      "name": "typing-inspection",
      "version": "0.4.4",
      "wheel_url": "https://files.pythonhosted.org/packages/67/81/4add07e5172b7ac40d8ed5ff580409a7801a4fe26d529bdd915401dabfbe/typing_inspection-0.4.4-py3-none-any.whl",
      "sha256": "65b8397ba37ccbce054456aaccddfc91e6e3083c92824df348d96ca832f3f147",
      "license_expression": "MIT",
      "license_metadata": "not specified"
    },
    {
      "name": "pycparser",
      "version": "3.12",
      "wheel_url": "https://files.pythonhosted.org/packages/b6/ae/315bbeee126a769cbde43cf4908acb4b40408d8bf8bbf334dd7368d8dadf/pycparser-3.12-py3-none-any.whl",
      "sha256": "a076f3f489b39b43ed13563a1dafe42447445bec2e294154ec5c57fcdda4c85e",
      "license_expression": "BSD-3-Clause",
      "license_metadata": "not specified"
    }
  ],
  "setup_failure": {
    "stage": "research-environment-package-resolution",
    "cause": "Lead initially requested nonexistent oldmemo 1.0.5; not a library build/test failure. Corrected to published 2.1.0.",
    "exit_code": 1,
    "stderr": "Collecting twomemo==2.1.0\n  Using cached twomemo-2.1.0-py3-none-any.whl.metadata (4.2 kB)\nERROR: Could not find a version that satisfies the requirement oldmemo==1.0.5 (from versions: 1.0.0, 1.0.1, 1.0.2, 1.0.3, 1.0.4, 1.1.0, 1.2.0, 2.0.0, 2.1.0)\nERROR: No matching distribution found for oldmemo==1.0.5\n"
  },
  "bootstrap_scope": {
    "pip_browser_asset_found": "pip/_vendor/urllib3/contrib/emscripten/emscripten_fetch_worker.js",
    "javascript_executed": false,
    "bootstrap_pip_removed_before_final_reference_run": true,
    "final_reference_js_ts_paths": [],
    "python_product_adoption": false
  },
  "cost_limits": {
    "binary_bytes_not_whole_client_ram": true,
    "hacl_maximum_resident_bytes_one_host_run": 5259264,
    "openssl_libcrypto_file_bytes": 4828048,
    "openssl_libssl_file_bytes_not_needed_by_crypto_only_link": 867008,
    "throughput_or_sbc_claim": false
  },
  "teardown": {
    "owned_build_and_reference_runtime_removed": true,
    "synthetic_key_fixture_removed": true,
    "no_listener_started": true,
    "saved_playground_untouched": true
  },
  "build": [
    {
      "argv": [
        "make",
        "-n",
        "o/openssl.o",
        "DRIVERS=hacl.c openssl.c"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.011,
      "stdout": "mkdir -p o\ncc -c -o o/openssl.o openssl.c -O2 -g -Wall -Wno-pointer-sign -Wno-unused-function -I. -MMD -MP -I gen\n",
      "stderr": ""
    },
    {
      "argv": [
        "clang",
        "-O2",
        "-Wall",
        "-Wno-pointer-sign",
        "-Wno-unused-function",
        "-I.",
        "-Igen",
        "-fPIC",
        "-c",
        "gen/omemo2.c",
        "-o",
        "o/omemo2.o",
        "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.198,
      "stdout": "",
      "stderr": ""
    },
    {
      "argv": [
        "clang",
        "-O2",
        "-Wall",
        "-Wno-pointer-sign",
        "-Wno-unused-function",
        "-I.",
        "-Igen",
        "-fPIC",
        "-c",
        "hacl.c",
        "-o",
        "o/hacl.o",
        "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.746,
      "stdout": "",
      "stderr": ""
    },
    {
      "argv": [
        "clang",
        "-O2",
        "-Wall",
        "-Wno-pointer-sign",
        "-Wno-unused-function",
        "-I.",
        "-Igen",
        "-fPIC",
        "-c",
        "openssl.c",
        "-o",
        "o/openssl.o",
        "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.054,
      "stdout": "",
      "stderr": ""
    },
    {
      "argv": [
        "ar",
        "-rcs",
        "o/libpicomemo2.a",
        "o/omemo2.o",
        "o/hacl.o",
        "o/openssl.o"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.063,
      "stdout": "",
      "stderr": ""
    },
    {
      "argv": [
        "clang",
        "-O2",
        "-DOMEMO2",
        "-I.",
        "test/generate.c",
        "omemo.c",
        "hacl.c",
        "openssl.c",
        "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include",
        "-L/opt/homebrew/Cellar/openssl@3/3.6.3/lib",
        "-lssl",
        "-lcrypto",
        "-o",
        "o/generate2"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.888,
      "stdout": "",
      "stderr": "omemo.c:608:29: warning: passing 'char[27]' to parameter of type 'const uint8_t *' (aka 'const unsigned char *') converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n  608 |   TRY(DeriveKey(Zero32, mk, HkdfInfoMessageKeys, kdfout));\n      |                             ^~~~~~~~~~~~~~~~~~~\nomemo.c:41:29: note: expanded from macro 'HkdfInfoMessageKeys'\n   41 | #define HkdfInfoMessageKeys \"OMEMO Message Key Material\"\n      |                             ^~~~~~~~~~~~~~~~~~~~~~~~~~~~\n./driver.h:27:104: note: passing argument to parameter 'info' here\n   27 | int omemoDriverHkdf(const uint8_t *salt, size_t saltn, const uint8_t *key, size_t keyn, const uint8_t *info, size_t infon, uint8_t *out, size_t outn);\n      |                                                                                                        ^\nomemo.c:671:36: warning: passing 'char[17]' to parameter of type 'const uint8_t *' (aka 'const unsigned char *') converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n  671 |   TRY(DeriveKey(state->rk, secret, HkdfInfoRootChain, masterkey));\n      |                                    ^~~~~~~~~~~~~~~~~\nomemo.c:40:29: note: expanded from macro 'HkdfInfoRootChain'\n   40 | #define HkdfInfoRootChain   \"OMEMO Root Chain\"\n      |                             ^~~~~~~~~~~~~~~~~~\n./driver.h:27:104: note: passing argument to parameter 'info' here\n   27 | int omemoDriverHkdf(const uint8_t *salt, size_t saltn, const uint8_t *key, size_t keyn, const uint8_t *info, size_t infon, uint8_t *out, size_t outn);\n      |                                                                                                        ^\nomemo.c:694:33: warning: passing 'char[11]' to parameter of type 'const uint8_t *' (aka 'const unsigned char *') converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n  694 |   TRY(DeriveKey(Zero32, secret, HkdfInfoKeyExchange, tmpkey));\n      |                                 ^~~~~~~~~~~~~~~~~~~\nomemo.c:39:29: note: expanded from macro 'HkdfInfoKeyExchange'\n   39 | #define HkdfInfoKeyExchange \"OMEMO X3DH\"\n      |                             ^~~~~~~~~~~~\n./driver.h:27:104: note: passing argument to parameter 'info' here\n   27 | int omemoDriverHkdf(const uint8_t *salt, size_t saltn, const uint8_t *key, size_t keyn, const uint8_t *info, size_t infon, uint8_t *out, size_t outn);\n      |                                                                                                        ^\nomemo.c:908:29: warning: passing 'char[27]' to parameter of type 'const uint8_t *' (aka 'const unsigned char *') converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n  908 |   TRY(DeriveKey(Zero32, mk, HkdfInfoMessageKeys, kdfout));\n      |                             ^~~~~~~~~~~~~~~~~~~\nomemo.c:41:29: note: expanded from macro 'HkdfInfoMessageKeys'\n   41 | #define HkdfInfoMessageKeys \"OMEMO Message Key Material\"\n      |                             ^~~~~~~~~~~~~~~~~~~~~~~~~~~~\n./driver.h:27:104: note: passing argument to parameter 'info' here\n   27 | int omemoDriverHkdf(const uint8_t *salt, size_t saltn, const uint8_t *key, size_t keyn, const uint8_t *info, size_t infon, uint8_t *out, size_t outn);\n      |                                                                                                        ^\nomemo.c:1061:28: warning: passing 'char[14]' to parameter of type 'const uint8_t *' (aka 'const unsigned char *') converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n 1061 |   TRY(DeriveKey(Zero32, k, HkdfInfoPayload, kdfout));\n      |                            ^~~~~~~~~~~~~~~\nomemo.c:42:29: note: expanded from macro 'HkdfInfoPayload'\n   42 | #define HkdfInfoPayload     \"OMEMO Payload\"\n      |                             ^~~~~~~~~~~~~~~\n./driver.h:27:104: note: passing argument to parameter 'info' here\n   27 | int omemoDriverHkdf(const uint8_t *salt, size_t saltn, const uint8_t *key, size_t keyn, const uint8_t *info, size_t infon, uint8_t *out, size_t outn);\n      |                                                                                                        ^\nomemo.c:1106:28: warning: passing 'char[14]' to parameter of type 'const uint8_t *' (aka 'const unsigned char *') converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n 1106 |   TRY(DeriveKey(Zero32, k, HkdfInfoPayload, kdfout));\n      |                            ^~~~~~~~~~~~~~~\nomemo.c:42:29: note: expanded from macro 'HkdfInfoPayload'\n   42 | #define HkdfInfoPayload     \"OMEMO Payload\"\n      |                             ^~~~~~~~~~~~~~~\n./driver.h:27:104: note: passing argument to parameter 'info' here\n   27 | int omemoDriverHkdf(const uint8_t *salt, size_t saltn, const uint8_t *key, size_t keyn, const uint8_t *info, size_t infon, uint8_t *out, size_t outn);\n      |                                                                                                        ^\n6 warnings generated.\nopenssl.c:124:42: warning: passing 'char[1]' to parameter of type 'const unsigned char *' converts between pointers to integer types where one is of the unique plain 'char' type and the other is not [-Wpointer-sign]\n  124 |   TRY(EVP_DecryptUpdate(ctx, NULL, &len, \"\", 0));\n      |                                          ^~\n/opt/homebrew/Cellar/openssl@3/3.6.3/include/openssl/evp.h:806:37: note: passing argument to parameter 'in' here\n  806 |     int *outl, const unsigned char *in, int inl);\n      |                                     ^\n1 warning generated.\n"
    }
  ],
  "fixture_generator": {
    "argv": [
      "o/generate2",
      "o/store2.inc",
      "o/bundle2.py"
    ],
    "exit_code": 0,
    "elapsed_seconds": 0.191,
    "stdout": "",
    "stderr": ""
  },
  "reference_fixture": {
    "argv": [
      "reference-venv/bin/python",
      "test/initsession.py",
      "bundle2"
    ],
    "env": {
      "PYTHONPATH": "o"
    },
    "exit_code": 0,
    "elapsed_seconds": 0.736,
    "stdout": "",
    "stderr": ""
  },
  "tests": {
    "host": {
      "system": "Darwin",
      "release": "27.0.0",
      "machine": "arm64"
    },
    "runs": [
      {
        "argv": [
          "clang",
          "-O2",
          "-Wall",
          "-Wno-pointer-sign",
          "-Wno-unused-function",
          "-DOMEMO2",
          "-I.",
          "test/omemo.c",
          "hacl.c",
          "openssl.c",
          "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include",
          "-L/opt/homebrew/Cellar/openssl@3/3.6.3/lib",
          "-lssl",
          "-lcrypto",
          "-o",
          "o/test-omemo2-hacl"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.905,
        "stdout": "",
        "stderr": "test/omemo.c:311:67: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                               ~~~                 ^~~~~~~~~~~\n      |                                               %llu\ntest/omemo.c:311:80: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                                           ~~~                  ^\n      |                                                           %llu\n2 warnings generated.\n"
      },
      {
        "argv": [
          "o/test-omemo2-hacl"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.103,
        "stdout": "\u001b[34mRunning test TestParseProtobuf\u001b[0m\n\u001b[32mFinished test TestParseProtobuf\u001b[0m\n\u001b[34mRunning test TestFormatProtobuf\u001b[0m\n\u001b[32mFinished test TestFormatProtobuf\u001b[0m\n\u001b[34mRunning test TestFormattings\u001b[0m\n\u001b[32mFinished test TestFormattings\u001b[0m\n\u001b[34mRunning test TestEncryptSize\u001b[0m\n\u001b[32mFinished test TestEncryptSize\u001b[0m\n\u001b[34mRunning test TestProtobufPrekey\u001b[0m\n\u001b[32mFinished test TestProtobufPrekey\u001b[0m\n\u001b[34mRunning test TestCurve25519\u001b[0m\n\u001b[32mFinished test TestCurve25519\u001b[0m\n\u001b[34mRunning test TestRotate\u001b[0m\n\u001b[32mFinished test TestRotate\u001b[0m\n\u001b[34mRunning test TestSignature\u001b[0m\n\u001b[32mFinished test TestSignature\u001b[0m\n\u001b[34mRunning test TestEncryption\u001b[0m\n\u001b[32mFinished test TestEncryption\u001b[0m\n\u001b[34mRunning test TestHkdf\u001b[0m\n\u001b[32mFinished test TestHkdf\u001b[0m\n\u001b[34mRunning test TestRatchet\u001b[0m\n\u001b[32mFinished test TestRatchet\u001b[0m\n\u001b[34mRunning test TestDeriveChainKey\u001b[0m\n\u001b[32mFinished test TestDeriveChainKey\u001b[0m\n\u001b[34mRunning test TestSerialization\u001b[0m\n\u001b[32mFinished test TestSerialization\u001b[0m\n\u001b[34mRunning test TestSerializationLongPreKeyId\u001b[0m\n\u001b[32mFinished test TestSerializationLongPreKeyId\u001b[0m\n\u001b[34mRunning test TestSessionIntegration\u001b[0m\n\u001b[32mFinished test TestSessionIntegration\u001b[0m\n\u001b[34mRunning test TestReceive\u001b[0m\n\u001b[32mFinished test TestReceive\u001b[0m\n\u001b[34mRunning test TestSession\u001b[0m\n\u001b[32mFinished test TestSession\u001b[0m\nAll tests succeeded\n",
        "stderr": ""
      },
      {
        "argv": [
          "clang",
          "-O2",
          "-Wall",
          "-Wno-pointer-sign",
          "-Wno-unused-function",
          "-DOMEMO2",
          "-I.",
          "test/omemo.c",
          "c25519.c",
          "openssl.c",
          "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include",
          "-L/opt/homebrew/Cellar/openssl@3/3.6.3/lib",
          "-lssl",
          "-lcrypto",
          "-o",
          "o/test-omemo2-c25519"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.582,
        "stdout": "",
        "stderr": "test/omemo.c:311:67: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                               ~~~                 ^~~~~~~~~~~\n      |                                               %llu\ntest/omemo.c:311:80: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                                           ~~~                  ^\n      |                                                           %llu\n2 warnings generated.\n"
      },
      {
        "argv": [
          "o/test-omemo2-c25519"
        ],
        "exit_code": 0,
        "elapsed_seconds": 1.243,
        "stdout": "\u001b[34mRunning test TestParseProtobuf\u001b[0m\n\u001b[32mFinished test TestParseProtobuf\u001b[0m\n\u001b[34mRunning test TestFormatProtobuf\u001b[0m\n\u001b[32mFinished test TestFormatProtobuf\u001b[0m\n\u001b[34mRunning test TestFormattings\u001b[0m\n\u001b[32mFinished test TestFormattings\u001b[0m\n\u001b[34mRunning test TestEncryptSize\u001b[0m\n\u001b[32mFinished test TestEncryptSize\u001b[0m\n\u001b[34mRunning test TestProtobufPrekey\u001b[0m\n\u001b[32mFinished test TestProtobufPrekey\u001b[0m\n\u001b[34mRunning test TestCurve25519\u001b[0m\n\u001b[32mFinished test TestCurve25519\u001b[0m\n\u001b[34mRunning test TestRotate\u001b[0m\n\u001b[32mFinished test TestRotate\u001b[0m\n\u001b[34mRunning test TestSignature\u001b[0m\n\u001b[32mFinished test TestSignature\u001b[0m\n\u001b[34mRunning test TestEncryption\u001b[0m\n\u001b[32mFinished test TestEncryption\u001b[0m\n\u001b[34mRunning test TestHkdf\u001b[0m\n\u001b[32mFinished test TestHkdf\u001b[0m\n\u001b[34mRunning test TestRatchet\u001b[0m\n\u001b[32mFinished test TestRatchet\u001b[0m\n\u001b[34mRunning test TestDeriveChainKey\u001b[0m\n\u001b[32mFinished test TestDeriveChainKey\u001b[0m\n\u001b[34mRunning test TestSerialization\u001b[0m\n\u001b[32mFinished test TestSerialization\u001b[0m\n\u001b[34mRunning test TestSerializationLongPreKeyId\u001b[0m\n\u001b[32mFinished test TestSerializationLongPreKeyId\u001b[0m\n\u001b[34mRunning test TestSessionIntegration\u001b[0m\n\u001b[32mFinished test TestSessionIntegration\u001b[0m\n\u001b[34mRunning test TestReceive\u001b[0m\n\u001b[32mFinished test TestReceive\u001b[0m\n\u001b[34mRunning test TestSession\u001b[0m\n\u001b[32mFinished test TestSession\u001b[0m\nAll tests succeeded\n",
        "stderr": ""
      },
      {
        "argv": [
          "clang",
          "-O2",
          "-I.",
          "-c",
          "c25519.c",
          "-o",
          "o/c25519.o"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.384,
        "stdout": "",
        "stderr": ""
      },
      {
        "argv": [
          "ar",
          "-rcs",
          "o/libpicomemo2-c25519.a",
          "o/omemo2.o",
          "o/c25519.o",
          "o/openssl.o"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.013,
        "stdout": "",
        "stderr": ""
      },
      {
        "argv": [
          "clang",
          "-O2",
          "-Wall",
          "-Wno-pointer-sign",
          "-Wno-unused-function",
          "-DOMEMO2",
          "-I.",
          "-fsanitize=address,undefined",
          "-fno-omit-frame-pointer",
          "test/omemo.c",
          "hacl.c",
          "openssl.c",
          "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include",
          "-L/opt/homebrew/Cellar/openssl@3/3.6.3/lib",
          "-lssl",
          "-lcrypto",
          "-o",
          "o/test-omemo2-sanitized"
        ],
        "exit_code": 0,
        "elapsed_seconds": 1.98,
        "stdout": "",
        "stderr": "test/omemo.c:311:67: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                               ~~~                 ^~~~~~~~~~~\n      |                                               %llu\ntest/omemo.c:311:80: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                                           ~~~                  ^\n      |                                                           %llu\n2 warnings generated.\n"
      },
      {
        "argv": [
          "o/test-omemo2-sanitized"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.177,
        "stdout": "\u001b[34mRunning test TestParseProtobuf\u001b[0m\n\u001b[32mFinished test TestParseProtobuf\u001b[0m\n\u001b[34mRunning test TestFormatProtobuf\u001b[0m\n\u001b[32mFinished test TestFormatProtobuf\u001b[0m\n\u001b[34mRunning test TestFormattings\u001b[0m\n\u001b[32mFinished test TestFormattings\u001b[0m\n\u001b[34mRunning test TestEncryptSize\u001b[0m\n\u001b[32mFinished test TestEncryptSize\u001b[0m\n\u001b[34mRunning test TestProtobufPrekey\u001b[0m\n\u001b[32mFinished test TestProtobufPrekey\u001b[0m\n\u001b[34mRunning test TestCurve25519\u001b[0m\n\u001b[32mFinished test TestCurve25519\u001b[0m\n\u001b[34mRunning test TestRotate\u001b[0m\n\u001b[32mFinished test TestRotate\u001b[0m\n\u001b[34mRunning test TestSignature\u001b[0m\n\u001b[32mFinished test TestSignature\u001b[0m\n\u001b[34mRunning test TestEncryption\u001b[0m\n\u001b[32mFinished test TestEncryption\u001b[0m\n\u001b[34mRunning test TestHkdf\u001b[0m\n\u001b[32mFinished test TestHkdf\u001b[0m\n\u001b[34mRunning test TestRatchet\u001b[0m\n\u001b[32mFinished test TestRatchet\u001b[0m\n\u001b[34mRunning test TestDeriveChainKey\u001b[0m\n\u001b[32mFinished test TestDeriveChainKey\u001b[0m\n\u001b[34mRunning test TestSerialization\u001b[0m\n\u001b[32mFinished test TestSerialization\u001b[0m\n\u001b[34mRunning test TestSerializationLongPreKeyId\u001b[0m\n\u001b[32mFinished test TestSerializationLongPreKeyId\u001b[0m\n\u001b[34mRunning test TestSessionIntegration\u001b[0m\n\u001b[32mFinished test TestSessionIntegration\u001b[0m\n\u001b[34mRunning test TestReceive\u001b[0m\n\u001b[32mFinished test TestReceive\u001b[0m\n\u001b[34mRunning test TestSession\u001b[0m\n\u001b[32mFinished test TestSession\u001b[0m\nAll tests succeeded\n",
        "stderr": ""
      },
      {
        "argv": [
          "otool",
          "-L",
          "o/test-omemo2-hacl"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.703,
        "stdout": "o/test-omemo2-hacl:\n\t/opt/homebrew/opt/openssl@3/lib/libssl.3.dylib (compatibility version 3.0.0, current version 3.0.0)\n\t/opt/homebrew/opt/openssl@3/lib/libcrypto.3.dylib (compatibility version 3.0.0, current version 3.0.0)\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1359.0.0)\n",
        "stderr": ""
      },
      {
        "argv": [
          "otool",
          "-L",
          "o/test-omemo2-c25519"
        ],
        "exit_code": 0,
        "elapsed_seconds": 0.009,
        "stdout": "o/test-omemo2-c25519:\n\t/opt/homebrew/opt/openssl@3/lib/libssl.3.dylib (compatibility version 3.0.0, current version 3.0.0)\n\t/opt/homebrew/opt/openssl@3/lib/libcrypto.3.dylib (compatibility version 3.0.0, current version 3.0.0)\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1359.0.0)\n",
        "stderr": ""
      }
    ],
    "artifacts": {
      "o/libpicomemo2.a": {
        "bytes": 154936,
        "sha256": "361e513c0ccf8478f8a83589c2c8af0c17c3800eb2bed9ad0d163d031c2f62b9"
      },
      "o/libpicomemo2-c25519.a": {
        "bytes": 88416,
        "sha256": "6ff1f39183540440efc999befceda3756c27c3c235fb811fccbefc90fbd3eb5c"
      },
      "o/test-omemo2-hacl": {
        "bytes": 215680,
        "sha256": "4d44bd141ea9bd3c67a6bed3b54823c53b3e92527a8ab4aeb1ce7e6237f4a9e7"
      },
      "o/test-omemo2-c25519": {
        "bytes": 163296,
        "sha256": "f0fab77575d6e9d64bf5b7eff61c2b64f6ac5e46653a82c85c87788f1a8281f9"
      },
      "o/test-omemo2-sanitized": {
        "bytes": 937944,
        "sha256": "30b246cc50229741b6fc13e463fbce519ab916570c18146a0e458b42e970bdfa"
      }
    }
  },
  "crypto_only": [
    {
      "argv": [
        "clang",
        "-O2",
        "-Wall",
        "-Wno-pointer-sign",
        "-Wno-unused-function",
        "-DOMEMO2",
        "-I.",
        "test/omemo.c",
        "hacl.c",
        "openssl.c",
        "-I/opt/homebrew/Cellar/openssl@3/3.6.3/include",
        "-L/opt/homebrew/Cellar/openssl@3/3.6.3/lib",
        "-lcrypto",
        "-o",
        "o/test-omemo2-crypto-only"
      ],
      "exit_code": 0,
      "elapsed_seconds": 1.198,
      "stdout": "",
      "stderr": "test/omemo.c:311:67: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                               ~~~                 ^~~~~~~~~~~\n      |                                               %llu\ntest/omemo.c:311:80: warning: format specifies type 'long' but the argument has type 'uint64_t' (aka 'unsigned long long') [-Wformat]\n  311 |     printf(\"StoreMessageKey: expected n to be %ld but got %ld\\n\", expectnskip, n);\n      |                                                           ~~~                  ^\n      |                                                           %llu\n2 warnings generated.\n"
    },
    {
      "argv": [
        "o/test-omemo2-crypto-only"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.199,
      "stdout": "\u001b[34mRunning test TestParseProtobuf\u001b[0m\n\u001b[32mFinished test TestParseProtobuf\u001b[0m\n\u001b[34mRunning test TestFormatProtobuf\u001b[0m\n\u001b[32mFinished test TestFormatProtobuf\u001b[0m\n\u001b[34mRunning test TestFormattings\u001b[0m\n\u001b[32mFinished test TestFormattings\u001b[0m\n\u001b[34mRunning test TestEncryptSize\u001b[0m\n\u001b[32mFinished test TestEncryptSize\u001b[0m\n\u001b[34mRunning test TestProtobufPrekey\u001b[0m\n\u001b[32mFinished test TestProtobufPrekey\u001b[0m\n\u001b[34mRunning test TestCurve25519\u001b[0m\n\u001b[32mFinished test TestCurve25519\u001b[0m\n\u001b[34mRunning test TestRotate\u001b[0m\n\u001b[32mFinished test TestRotate\u001b[0m\n\u001b[34mRunning test TestSignature\u001b[0m\n\u001b[32mFinished test TestSignature\u001b[0m\n\u001b[34mRunning test TestEncryption\u001b[0m\n\u001b[32mFinished test TestEncryption\u001b[0m\n\u001b[34mRunning test TestHkdf\u001b[0m\n\u001b[32mFinished test TestHkdf\u001b[0m\n\u001b[34mRunning test TestRatchet\u001b[0m\n\u001b[32mFinished test TestRatchet\u001b[0m\n\u001b[34mRunning test TestDeriveChainKey\u001b[0m\n\u001b[32mFinished test TestDeriveChainKey\u001b[0m\n\u001b[34mRunning test TestSerialization\u001b[0m\n\u001b[32mFinished test TestSerialization\u001b[0m\n\u001b[34mRunning test TestSerializationLongPreKeyId\u001b[0m\n\u001b[32mFinished test TestSerializationLongPreKeyId\u001b[0m\n\u001b[34mRunning test TestSessionIntegration\u001b[0m\n\u001b[32mFinished test TestSessionIntegration\u001b[0m\n\u001b[34mRunning test TestReceive\u001b[0m\n\u001b[32mFinished test TestReceive\u001b[0m\n\u001b[34mRunning test TestSession\u001b[0m\n\u001b[32mFinished test TestSession\u001b[0m\nAll tests succeeded\n",
      "stderr": ""
    },
    {
      "argv": [
        "otool",
        "-L",
        "o/test-omemo2-crypto-only"
      ],
      "exit_code": 0,
      "elapsed_seconds": 0.014,
      "stdout": "o/test-omemo2-crypto-only:\n\t/opt/homebrew/opt/openssl@3/lib/libcrypto.3.dylib (compatibility version 3.0.0, current version 3.0.0)\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1359.0.0)\n",
      "stderr": ""
    }
  ],
  "clean_reference": {
    "reference_runtime_js_ts_files": [],
    "reference_fixture_exit_code": 0,
    "reference_fixture_stderr": "",
    "hacl_test_exit_code": 0,
    "hacl_test_stdout": "\u001b[34mRunning test TestParseProtobuf\u001b[0m\n\u001b[32mFinished test TestParseProtobuf\u001b[0m\n\u001b[34mRunning test TestFormatProtobuf\u001b[0m\n\u001b[32mFinished test TestFormatProtobuf\u001b[0m\n\u001b[34mRunning test TestFormattings\u001b[0m\n\u001b[32mFinished test TestFormattings\u001b[0m\n\u001b[34mRunning test TestEncryptSize\u001b[0m\n\u001b[32mFinished test TestEncryptSize\u001b[0m\n\u001b[34mRunning test TestProtobufPrekey\u001b[0m\n\u001b[32mFinished test TestProtobufPrekey\u001b[0m\n\u001b[34mRunning test TestCurve25519\u001b[0m\n\u001b[32mFinished test TestCurve25519\u001b[0m\n\u001b[34mRunning test TestRotate\u001b[0m\n\u001b[32mFinished test TestRotate\u001b[0m\n\u001b[34mRunning test TestSignature\u001b[0m\n\u001b[32mFinished test TestSignature\u001b[0m\n\u001b[34mRunning test TestEncryption\u001b[0m\n\u001b[32mFinished test TestEncryption\u001b[0m\n\u001b[34mRunning test TestHkdf\u001b[0m\n\u001b[32mFinished test TestHkdf\u001b[0m\n\u001b[34mRunning test TestRatchet\u001b[0m\n\u001b[32mFinished test TestRatchet\u001b[0m\n\u001b[34mRunning test TestDeriveChainKey\u001b[0m\n\u001b[32mFinished test TestDeriveChainKey\u001b[0m\n\u001b[34mRunning test TestSerialization\u001b[0m\n\u001b[32mFinished test TestSerialization\u001b[0m\n\u001b[34mRunning test TestSerializationLongPreKeyId\u001b[0m\n\u001b[32mFinished test TestSerializationLongPreKeyId\u001b[0m\n\u001b[34mRunning test TestSessionIntegration\u001b[0m\n\u001b[32mFinished test TestSessionIntegration\u001b[0m\n\u001b[34mRunning test TestReceive\u001b[0m\n\u001b[32mFinished test TestReceive\u001b[0m\n\u001b[34mRunning test TestSession\u001b[0m\n\u001b[32mFinished test TestSession\u001b[0m\nAll tests succeeded\n",
    "hacl_cost_stderr": "        0.01 real         0.01 user         0.00 sys\n             5259264  maximum resident set size\n                   0  average shared memory size\n                   0  average unshared data size\n                   0  average unshared stack size\n                 514  page reclaims\n                  38  page faults\n                   0  swaps\n                   0  block input operations\n                   0  block output operations\n                   0  messages sent\n                   0  messages received\n                   0  signals received\n                   0  voluntary context switches\n                  24  involuntary context switches\n           275188136  instructions retired\n            62054066  cycles elapsed\n             2113872  peak memory footprint\n"
  }
}
```

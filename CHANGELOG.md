# Changelog

All notable changes to `dsc` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Releases are grouped from conventional-commit messages by [git-cliff](https://git-cliff.org).

## [0.10.15] - 2026-06-17

### Documentation

- **spec**: Audit downstream code for negative user-id impact ([bb18cd6](https://github.com/pacharanero/dsc/commit/bb18cd6aa8d04b08f9e0b2ca0b16f1d22d73e823))

### Refactor

- **utils**: Use slug crate for slugify, handles Unicode ([690b321](https://github.com/pacharanero/dsc/commit/690b3210ad8b523172b5e706e87130027d5c988e))

## [0.10.14] - 2026-06-17

### Bug fixes

- **user**: Tolerate negative IDs for Discourse system accounts ([d3c6d55](https://github.com/pacharanero/dsc/commit/d3c6d5516ac912ebabdbcda5607e07a5c05b1988))

## [0.10.13] - 2026-06-10

### Features

- **cli**: Add 'dsc man' for generating Unix man pages ([77d20ed](https://github.com/pacharanero/dsc/commit/77d20eda89c26b3d07699f83adfa62882624094b))

## [0.10.12] - 2026-06-10

### Bug fixes

- **cli**: Bring 6 empty-list + 1 error message in line with spec ([bab77a7](https://github.com/pacharanero/dsc/commit/bab77a7536a3a8e2a42a480ede4622c471660aa3))

### Documentation

- Pre-1.0 polish batch (CHANGELOG, CONTRIBUTING, issue templates) ([038d3c7](https://github.com/pacharanero/dsc/commit/038d3c7285061af3a1c7b631842e93bf8d8a491d))

## [0.10.11] - 2026-06-10

### Documentation

- **spec**: Merge .marcus notes + integrate field-driven specs ([51b6fc7](https://github.com/pacharanero/dsc/commit/51b6fc75f5e6139311b2df60aa4fbc320250be7e))

- **roadmap**: Note git-cliff as recommended changelog tool ([0364d28](https://github.com/pacharanero/dsc/commit/0364d2869e1826351c34644863a1134fcdebe0ca))

- **roadmap**: Add pre-1.0 launch checklist ([9cacf84](https://github.com/pacharanero/dsc/commit/9cacf84121002c8490de3cafd989d0618cfe77a9))

- Rename agents.md to AGENTS.md ([ff49b3f](https://github.com/pacharanero/dsc/commit/ff49b3f246476bb42f0d3cabb30ab7e390828eca))

- Add agents.md - guide for LLMs using dsc in other sessions ([5390ccb](https://github.com/pacharanero/dsc/commit/5390ccb2ab5ec7b74141e62a88dc94f7f4673f1d))

- Accuracy pass + roadmap cleanup ([2fec564](https://github.com/pacharanero/dsc/commit/2fec56453d303e17cefbe6ddf8c104274b1d4586))

### Features

- **topic**: Add 'dsc topic pull --full' for whole-thread export ([3bda807](https://github.com/pacharanero/dsc/commit/3bda80786cab0d2e2cf9a97fcaa8eb33a3f53842))

## [0.10.10] - 2026-06-09

### Features

- **tag**: Add 'dsc tag rename' preserving topic associations ([65b2a65](https://github.com/pacharanero/dsc/commit/65b2a65d8d2f6277ccec9aeedd0c71406bb00ba5))

## [0.10.9] - 2026-06-09

### CI / dependencies

- **deps**: Bump checkout v6.0.3, upload-artifact v7.0.1, download-artifact v8.0.1 ([d66c74c](https://github.com/pacharanero/dsc/commit/d66c74cdc9ee8425de0419a3eb428c0778bbf832))

### Documentation

- **roadmap**: Mark setting-sync (Phases 1-4) as completed ([1f3a318](https://github.com/pacharanero/dsc/commit/1f3a31805fabc915244a270bdce11a025ebabdc9))

### Features

- **config**: Add $DSC_CONFIG and $DSC_CONFIG_HOME resolution ([216b848](https://github.com/pacharanero/dsc/commit/216b848b7c54be0c105a97b784b5ca91e97092e8))

- **tag**: Add declarative pull/push spec for managing tag taxonomy ([13aa024](https://github.com/pacharanero/dsc/commit/13aa02490cc7688ec6dbeb02d71ddc8d1332956a))

- **harden**: Enhance SSH algorithm checks to prevent weak crypto usage ([979c3d1](https://github.com/pacharanero/dsc/commit/979c3d1c8d1b9add9310f7e50e56644a67e0f7d7))

## [0.10.8] - 2026-06-07

### Features

- **setting**: Add 'dsc setting diff' for cross-source comparison (Phase 3) ([603a58e](https://github.com/pacharanero/dsc/commit/603a58e8968d808ca850c989287626d88bb2b5fe))

## [0.10.7] - 2026-06-07

### Features

- **setting**: Add 'dsc setting push' for idempotent apply (Phase 2) ([edaa0ad](https://github.com/pacharanero/dsc/commit/edaa0ad62c479069b1bc76666877b6d6aa48276a))

## [0.10.6] - 2026-06-07

### Features

- **setting**: Add 'dsc setting pull' for declarative snapshots (Phase 1) ([9d48885](https://github.com/pacharanero/dsc/commit/9d48885d8861bebcb142f849902e505cd6dd9e91))

## [0.10.5] - 2026-06-07

### Bug fixes

- Add cooldown configuration for dependencies in dependabot.yml ([725cf6d](https://github.com/pacharanero/dsc/commit/725cf6d0cac8584d56a797b97baad9703a894999))

- Support rootless Docker in dsc update ([c3db942](https://github.com/pacharanero/dsc/commit/c3db942a984326132581916b73f68371871edadb))

- TagInfo.id is u64, use text field for tag names ([a889d5a](https://github.com/pacharanero/dsc/commit/a889d5a148f850854ad28caf17c744035cf09314))

### Chores

- Pin GitHub Actions to commit SHAs (supply-chain security) ([b0e7823](https://github.com/pacharanero/dsc/commit/b0e782344c6f1c9dc6b581060bbdd9c8668d3629))

- Bump version to 0.10.4 ([9b2a170](https://github.com/pacharanero/dsc/commit/9b2a17079409a6558758bda29c88ad6f3873c5b2))

### Documentation

- Fix setting.md inaccuracies and reference bulk pull/push spec ([1884f52](https://github.com/pacharanero/dsc/commit/1884f52ac9fdfaa6059bfe53ffc32f26ec362df9))

- **harden**: Refine SSH configuration details and clarify algorithm policies ([01f1287](https://github.com/pacharanero/dsc/commit/01f12879468a6366722ffd935e1a37baf6afc938))

- **dsc.example.toml**: Update SSH algorithm comments for clarity and accuracy ([cad6d66](https://github.com/pacharanero/dsc/commit/cad6d66783449fbb4f7f607b78c497f307f1a5db))

- **index**: Consolidate dsc-rs naming note into the Cargo tab ([e5d67bc](https://github.com/pacharanero/dsc/commit/e5d67bc8ff85635f9b5ac5ee96fd1c67cd34f06f))

- **index**: Add platform icons to install tabs ([e7c3e0d](https://github.com/pacharanero/dsc/commit/e7c3e0d4b7479c7cf9fb98e8a4bc8ef8218cccca))

- **index**: Convert install section to content tabs ([8255a93](https://github.com/pacharanero/dsc/commit/8255a9352c19d2556290cd462b2a28ac00b03c06))

- Move top-level nav from header tabs to left sidebar ([9cef00f](https://github.com/pacharanero/dsc/commit/9cef00faa83fda17ef8132e29de937b5115080a9))

- Scheme-conditional logo + 2× size ([1d61171](https://github.com/pacharanero/dsc/commit/1d61171c39b61762edcc4321de722c1a173042ec))

- Switch to Zensical modern variant + brand-orange accent ([d277823](https://github.com/pacharanero/dsc/commit/d27782346366cb1138e89c988973232bc277dbd0))

- Add analytics + harden to nav, enable dark-mode toggle ([cfa4d30](https://github.com/pacharanero/dsc/commit/cfa4d3023914bd3f427c322dd55c402b172ed7c7))

### Features

- **setting**: Make set --tags reachable from CLI (Phase 4) ([04f3fc1](https://github.com/pacharanero/dsc/commit/04f3fc1b6cd9bde1854ec6aef26a8ee4ee9b6ea4))

- Declarative tag taxonomy pull/push, move topic tagging to dsc topic ([e61b531](https://github.com/pacharanero/dsc/commit/e61b53148a8b408f12e9cd1cbbc3862c23c06b6a))

- Enhance config command to display active config and search order ([3d53c1a](https://github.com/pacharanero/dsc/commit/3d53c1aaf18554b5383b20eb15f0ae7a5dd9fd1c))

- Harmonise post, backup, emoji with pull/push pattern ([cfc7826](https://github.com/pacharanero/dsc/commit/cfc782628938bebf39f97706c5237ffea716f181))

- **harden**: Stage 2 — sshd tightening + ssh.socket patch ([010a3d8](https://github.com/pacharanero/dsc/commit/010a3d8ee6b9046266f7c84e9b0bc043371ab0f5))

### Spec

- Setting sync (bulk pull/push) and project roadmap ([46019d8](https://github.com/pacharanero/dsc/commit/46019d8dda0ca81970f4400cf7f09fe20ccf28bb))

## [0.10.3] - 2026-04-27

### Bug fixes

- **utils**: `1m` means 1 month, not 1 minute ([1442875](https://github.com/pacharanero/dsc/commit/1442875378b69c0b4325a7c72c6e39609df62c23))

## [0.10.2] - 2026-04-27

### Features

- **analytics**: --format table + --snapshot multi-window mode ([b021c3f](https://github.com/pacharanero/dsc/commit/b021c3ffdd9c8e5d6ed1f62cda0eccf93a940df8))

## [0.10.1] - 2026-04-27

### Bug fixes

- **analytics**: Stacked-chart aggregation + new_contributors wiring ([d6c83d1](https://github.com/pacharanero/dsc/commit/d6c83d14f83697309e6de43eea375b4ed42d52de))

## [0.10.0] - 2026-04-27

### Build

- **docs**: Serve install.sh and install.ps1 proxies from the docs site ([3672df4](https://github.com/pacharanero/dsc/commit/3672df4eb26b7fbfe2f255d9a2969e7ebfdfd6bd))

### Features

- **analytics**: Implement spec/analytics.md (v1) ([0ea3407](https://github.com/pacharanero/dsc/commit/0ea34076f3dbb24c15e68eeeaf5ec8045decc999))

- **harden**: Config block + flag override, SECURITY.md, docs ([8b8a994](https://github.com/pacharanero/dsc/commit/8b8a994180121465d77e15c31d41ccd0454bf3fa))

- **harden**: Stage 1 — user creation + pubkey install + self-lockout guard ([e49ce15](https://github.com/pacharanero/dsc/commit/e49ce15ed551f5e161dce3276442e242bc40c1bc))

### Spec

- Add analytics command spec ([07a8ff0](https://github.com/pacharanero/dsc/commit/07a8ff0a95e78b2278926bba62f3ab000144d1df))

## [0.9.0] - 2026-04-21

### Features

- Complete Phase 2 — user create, password-reset, email-set ([470b1a8](https://github.com/pacharanero/dsc/commit/470b1a8590c0cc7cb7b11b64db076c4040f944db))

## [0.8.3] - 2026-04-21

### Build

- **dist**: Add Homebrew tap, PowerShell, and MSI installers ([3676476](https://github.com/pacharanero/dsc/commit/367647628c313c09ff55653b49e12050d7359b61))

### CI / dependencies

- **deps**: Bump action pins (consolidates #11, #12, #13) ([8086b26](https://github.com/pacharanero/dsc/commit/8086b26e6b5238eafe06e61ff542c47c595380d2))

### Documentation

- **s/docs**: Detect inotify saturation and print a fixable error ([5657e84](https://github.com/pacharanero/dsc/commit/5657e84e732f411d7726c3b8a8b512bbaf095016))

- Make s/docs surface the inotify gotcha before it bites ([ea39481](https://github.com/pacharanero/dsc/commit/ea39481a36fbc87b38760c0ac959489ce9966ebf))

- Add Zensical site with GitHub Pages deploy ([be4ac2a](https://github.com/pacharanero/dsc/commit/be4ac2a6c7fb6a5e8b045dbbd3dde44e2ea07bfc))

### Features

- **docs**: Add initial bash script to serve Zensical ([a8a4501](https://github.com/pacharanero/dsc/commit/a8a45017e439608a68242f1586a71918db462303))

## [0.8.2] - 2026-04-19

### Bug fixes

- Update forum references in topic commands for user activity examples ([74c6864](https://github.com/pacharanero/dsc/commit/74c6864cea5fcfd811f060084472554f59f1ee6b))

### Features

- **user activity**: Work without an API key for public forums ([182a0e4](https://github.com/pacharanero/dsc/commit/182a0e48648d112da87b469311a6bf1b4435e057))

## [0.8.1] - 2026-04-19

### Features

- Dsc user activity — archive public activity to a journal forum ([32868ac](https://github.com/pacharanero/dsc/commit/32868ac9f55a934c413334af0f189cd87838ec24))

## [0.8.0] - 2026-04-19

### Features

- Dsc pm send + list (Phase 3 starter) ([4173260](https://github.com/pacharanero/dsc/commit/4173260637049bca6d8d449e88328642fa13d0d2))

## [0.7.0] - 2026-04-19

### Features

- Dsc api-key list / create / revoke ([61870e5](https://github.com/pacharanero/dsc/commit/61870e5a5b02c311b7d1618f12dd78d7f02249bc))

## [0.6.0] - 2026-04-19

### Features

- Invites + user moderation toolkit (silence, promote, demote) ([98437ae](https://github.com/pacharanero/dsc/commit/98437ae06a6fe510599843988bd7180b6578daad))

## [0.5.0] - 2026-04-19

### Features

- Phase 2 start — dsc user list / info / suspend / unsuspend ([add8057](https://github.com/pacharanero/dsc/commit/add8057b226e8d6c67cd03dd987ad5aad2d00747))

## [0.4.0] - 2026-04-19

### Chores

- Stop vendoring generated shell completions ([c19224a](https://github.com/pacharanero/dsc/commit/c19224a6c74d055a388f5227681023de9741397c))

### Features

- Phase 1 remainder — post ops, group/user membership, full dry-run ([8b959ff](https://github.com/pacharanero/dsc/commit/8b959fffbd2292be2c0ee36d1aed83a581a83ec9))

- Add search, tag, and upload commands with documentation ([27a458a](https://github.com/pacharanero/dsc/commit/27a458aac51fb1c8342a6c31f3555b35836d504f))

## [0.3.0] - 2026-04-17

### Features

- Phase 0 — foundations, new commands, and retry/config/dry-run ([9d7999b](https://github.com/pacharanero/dsc/commit/9d7999b1176bd986ff39c30cc07fd034a43f73cf))

## [0.2.1] - 2026-04-10

### Chores

- Upgrade cargo-dist to 0.31.0 and regenerate release.yml ([40d4dfb](https://github.com/pacharanero/dsc/commit/40d4dfbbfb89f3c08143766c74a00ac5eab32717))

## [0.2.0] - 2026-04-10

### CI

- Add crates.io publish workflow ([4886464](https://github.com/pacharanero/dsc/commit/4886464a5bba2987cf63f098fc63eccdbf62b1ba))

### Chores

- Rename crate to dsc-rs and add crates.io metadata ([07d4f1e](https://github.com/pacharanero/dsc/commit/07d4f1e24c29de29fbc9a4ffe3c1b66184855dc0))

### Features

- **cli**: Add abbreviated aliases for all subcommands ([36c0846](https://github.com/pacharanero/dsc/commit/36c08462ce9883e4e8d1bfee7081a9369b92e8f2))

- Add FUNDING.yml enable GitHub Sponsors ([42c91a5](https://github.com/pacharanero/dsc/commit/42c91a5339faede92f451a9a008df76a03654264))

- Add theme management commands for pull, push, and duplicate ([70993e4](https://github.com/pacharanero/dsc/commit/70993e49b57b192b512ddcbece15e3ed43f71664))

- Enhance site settings management in dsc CLI ([695334a](https://github.com/pacharanero/dsc/commit/695334a0772ac7fa7246022a958afb9a03c2bd2f))

## [0.1.6] - 2026-03-04

### CI / dependencies

- **deps**: Bump actions/upload-artifact from 6 to 7 ([d8f15f2](https://github.com/pacharanero/dsc/commit/d8f15f2021d2e834abc3691fad2092b0d89e4cef))

- **deps**: Bump actions/download-artifact from 7 to 8 ([0caf332](https://github.com/pacharanero/dsc/commit/0caf332535ed7848eaa321fc4e8ac3b457cdd266))

- **deps**: Bump actions/download-artifact from 4 to 7 ([a9f78e7](https://github.com/pacharanero/dsc/commit/a9f78e75b1616999cea4def18af798a187502aeb))

- **deps**: Bump actions/checkout from 4 to 6 ([dc06aed](https://github.com/pacharanero/dsc/commit/dc06aed40dcaf7ac8a9e79af915d2ecde1b72cb0))

- **deps**: Bump actions/upload-artifact from 4 to 6 ([36dd5a3](https://github.com/pacharanero/dsc/commit/36dd5a3d8ea93be53368f101ec58f60bc4570203))

- **deps**: Update toml requirement from 0.9 to 1.0 ([db03400](https://github.com/pacharanero/dsc/commit/db034000e5231ae0cb1eeeefef4d5d07e63f0d59))

### Features

- Bump version to 0.1.5; enhance CLI help text for commands and flags ([2f02c7e](https://github.com/pacharanero/dsc/commit/2f02c7ec634f9ff997780c0d5deaef582fd5913c))

## [0.1.5] - 2026-03-03

### Features

- Bump version to 0.1.4 and update dependencies; enhance update command flags and documentation ([209f3d5](https://github.com/pacharanero/dsc/commit/209f3d5abf8b6c5edec27399fdc771fa26a98631))

- Add version bump script for automated tagging ([4e1c51c](https://github.com/pacharanero/dsc/commit/4e1c51c97dbe289840641c05ac177b45853d7666))

## [0.1.3] - 2026-03-03

### Chores

- Update indicatif dependency to version 0.18 ([78d86b3](https://github.com/pacharanero/dsc/commit/78d86b3584702d892b38ca454bbd522c9a59cb1f))

- Regenerate cargo-dist release workflow ([c7742ef](https://github.com/pacharanero/dsc/commit/c7742eff6e3d1ba84ac5fa987fc70d6d67cbb07c))

### Documentation

- Update README with new environment variables for `dsc update` and name recommendations ([73d15a1](https://github.com/pacharanero/dsc/commit/73d15a173f374eff42c4acf4dc514757c91131a2))

### Features

- Add --yes flag to update commands for auto-confirming changelog posts ([bc409c2](https://github.com/pacharanero/dsc/commit/bc409c201caf69820e22a01227a30226d934d707))

- Enhance update checklist with detailed versioning and disk usage information ([2d3d9b6](https://github.com/pacharanero/dsc/commit/2d3d9b6c439ebfaf5535727ea9f61e1f1f158264))

- Add dynamic discourse completion to zsh scripts and improve update command feedback ([3541c1f](https://github.com/pacharanero/dsc/commit/3541c1f10dcb3a25a1465c2efad35d31618ab30b))

## [0.1.2] - 2026-02-01

### Bug fixes

- Mark changelog path and interactive prompt decisions as complete ([b0adc9c](https://github.com/pacharanero/dsc/commit/b0adc9ce29b914cd9a52ec39008930c06db7f58e))

- Broaden emoji list parsing ([d4edc5f](https://github.com/pacharanero/dsc/commit/d4edc5f6373650ad21768f214ff8241e7f5ef36e))

- Remove duplicate config module ([8a475a9](https://github.com/pacharanero/dsc/commit/8a475a9c3290ed2755d85a4c5e28c0f9fb9457b3))

### Documentation

- Reorganize roadmap and merge todo ([b53fcbc](https://github.com/pacharanero/dsc/commit/b53fcbc9aa8d2cdb420435e702f879c593ea795a))

- Drop incorrect add prompt note ([443c0e3](https://github.com/pacharanero/dsc/commit/443c0e3767c79845101909c60dde993631e569a3))

### Features

- Remove update-all logging ([fb7f4d5](https://github.com/pacharanero/dsc/commit/fb7f4d5b84a5ce27a64f9a6416b542f8cecec8a7))

- Add inline emoji listing ([4f2abdd](https://github.com/pacharanero/dsc/commit/4f2abdd23aa00ca7942cd62f800d81956a2130ed))

- Add theme management commands ([5910532](https://github.com/pacharanero/dsc/commit/5910532ffae9c54d1e4aa7e1c2be5819a843780e))

- Add plugin management commands ([9420b3d](https://github.com/pacharanero/dsc/commit/9420b3d1555cada2fbbb9fd95ec86c11a5a50b8a))

- Add palette commands ([e112ee3](https://github.com/pacharanero/dsc/commit/e112ee3bbbc9a5a3a21fab688d2f126b8d45c414))

- Improve backup list and os update handling ([b73cecd](https://github.com/pacharanero/dsc/commit/b73cecd03585d2f0d3ecf2533db311c13ac01c55))

- Add site setting updates and format options ([4f3db77](https://github.com/pacharanero/dsc/commit/4f3db77aac555db7236940320f559f6e1bd6890c))

- Add site setting update helper ([4b8eb21](https://github.com/pacharanero/dsc/commit/4b8eb210cda74dfde593b618b09305e458d960df))

- Enhance CLI with tag filtering and emoji upload improvements ([dfc0268](https://github.com/pacharanero/dsc/commit/dfc02681f6c274da53c40723f8ad57b882961f86))

### Refactor

- Modularize cli and api code ([d02e0ff](https://github.com/pacharanero/dsc/commit/d02e0ff510989603c7f9a1f85b4bb4d93cc20a09))

### Tests

- Add common module to new tests ([b69a254](https://github.com/pacharanero/dsc/commit/b69a2544c1549cd7b32a84dae689eb9b7c8047a6))

- Add completions e2e and refresh scripts ([333d2bb](https://github.com/pacharanero/dsc/commit/333d2bb2c76de5f84c2cf838da7c25b4750fbd01))

## [0.1.1] - 2026-01-30



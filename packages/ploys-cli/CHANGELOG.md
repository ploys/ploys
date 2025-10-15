# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-10-15

### Changed

- Add `package init` command ([#297](https://github.com/ploys/ploys/pull/297))
- Remove `project init` command `template` option ([#299](https://github.com/ploys/ploys/pull/299))

## [0.3.0] - 2025-08-14

### Changed

- Add `tracing` integration ([#183](https://github.com/ploys/ploys/pull/183))
- Add `project init` command ([#213](https://github.com/ploys/ploys/pull/213))
- Add support for inspecting file system projects ([#216](https://github.com/ploys/ploys/pull/216))
- Add directory creation to `project init` command ([#228](https://github.com/ploys/ploys/pull/228))
- Add `project init` command default name from path ([#229](https://github.com/ploys/ploys/pull/229))
- Add `project init` command initial package template ([#233](https://github.com/ploys/ploys/pull/233))
- Add `project init` command version control system option ([#236](https://github.com/ploys/ploys/pull/236))
- Add `project init` command author option ([#239](https://github.com/ploys/ploys/pull/239))
- Upgrade packages to Rust 2024 edition ([#241](https://github.com/ploys/ploys/pull/241))
- Refactor collapsible `if let` chains ([#250](https://github.com/ploys/ploys/pull/250))

## [0.2.0] - 2024-12-13

### Changed

- Add initial package changelogs ([#99](https://github.com/ploys/ploys/pull/99))
- Add `package release` command ([#110](https://github.com/ploys/ploys/pull/110))
- Remove `release` command ([#111](https://github.com/ploys/ploys/pull/111))
- Hide `project info` command `token` argument env ([#158](https://github.com/ploys/ploys/pull/158))

## [0.1.0] - 2024-10-16

### Changed

- Add command line application boilerplate ([#1](https://github.com/ploys/ploys/pull/1))
- Add inspect command ([#2](https://github.com/ploys/ploys/pull/2))
- Add support for inspecting remote repositories ([#3](https://github.com/ploys/ploys/pull/3))
- Add inspect command authentication token option ([#4](https://github.com/ploys/ploys/pull/4))
- Refactor inspect command to use library utility ([#6](https://github.com/ploys/ploys/pull/6))
- Add inspect command discovered packages output ([#11](https://github.com/ploys/ploys/pull/11))
- Rename inspect command to project info ([#14](https://github.com/ploys/ploys/pull/14))
- Add release command ([#47](https://github.com/ploys/ploys/pull/47))
- Add `project info` command git revision arguments ([#54](https://github.com/ploys/ploys/pull/54))
- Add CI test target `aarch64-apple-darwin` ([#77](https://github.com/ploys/ploys/pull/77))
- Add release workflow build job ([#78](https://github.com/ploys/ploys/pull/78))
- Add release workflow publish for `ploys-cli` package ([#79](https://github.com/ploys/ploys/pull/79))
- Set `ploys` dependency version ([#80](https://github.com/ploys/ploys/pull/80))

[0.4.0]: https://github.com/ploys/ploys/releases/tag/ploys-cli-0.4.0
[0.3.0]: https://github.com/ploys/ploys/releases/tag/ploys-cli-0.3.0
[0.2.0]: https://github.com/ploys/ploys/releases/tag/ploys-cli-0.2.0
[0.1.0]: https://github.com/ploys/ploys/releases/tag/ploys-cli-0.1.0

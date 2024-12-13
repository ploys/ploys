# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2024-12-13

### Changed

- Add package release `repository_dispatch` event handler ([#108](https://github.com/ploys/ploys/pull/108))
- Remove release branch creation webhook event handler ([#131](https://github.com/ploys/ploys/pull/131))
- Add project package release request builder ([#134](https://github.com/ploys/ploys/pull/134))
- Add project package release builder ([#154](https://github.com/ploys/ploys/pull/154))
- Bump `shuttle-runtime` to `0.49.0` ([#173](https://github.com/ploys/ploys/pull/173))

## [0.2.1] - 2024-10-24

### Changed

- Fix missing `time` crate `parsing` and `formatting` features ([#103](https://github.com/ploys/ploys/pull/103))

## [0.2.0] - 2024-10-24

### Changed

- Add CI test target `aarch64-apple-darwin` ([#77](https://github.com/ploys/ploys/pull/77))
- Set `ploys` dependency version ([#80](https://github.com/ploys/ploys/pull/80))
- Add release notes and changelog file generation ([#95](https://github.com/ploys/ploys/pull/95))
- Add initial package changelogs ([#99](https://github.com/ploys/ploys/pull/99))

## [0.1.0] - 2024-10-14

### Changed

- Add shuttle application boilerplate ([#24](https://github.com/ploys/ploys/pull/24))
- Bump shuttle-runtime to 0.45.0 and axum to 0.7.5 ([#29](https://github.com/ploys/ploys/pull/29))
- Add GitHub webhook endpoint ([#30](https://github.com/ploys/ploys/pull/30))
- Bump shuttle-runtime to 0.47.0 ([#32](https://github.com/ploys/ploys/pull/32))
- Add GitHub webhook endpoint secret validation ([#33](https://github.com/ploys/ploys/pull/33))
- Add automated release pull request creation ([#58](https://github.com/ploys/ploys/pull/58))
- Add automated release creation ([#64](https://github.com/ploys/ploys/pull/64))
- Bump `shuttle-runtime` to `0.48.0` ([#68](https://github.com/ploys/ploys/pull/68))
- Add release workflow with deploy job ([#69](https://github.com/ploys/ploys/pull/69))

[0.3.0]: https://github.com/ploys/ploys/releases/tag/ploys-api-0.3.0
[0.2.1]: https://github.com/ploys/ploys/releases/tag/ploys-api-0.2.1
[0.2.0]: https://github.com/ploys/ploys/releases/tag/ploys-api-0.2.0
[0.1.0]: https://github.com/ploys/ploys/releases/tag/ploys-api-0.1.0

# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-10-24

### Changed

- Add CI test target `aarch64-apple-darwin` ([#77](https://github.com/ploys/ploys/pull/77))
- Add support for package dependencies ([#82](https://github.com/ploys/ploys/pull/82))
- Add changelog parser ([#87](https://github.com/ploys/ploys/pull/87))
- Add changelog builder ([#88](https://github.com/ploys/ploys/pull/88))
- Add changelog release generator ([#90](https://github.com/ploys/ploys/pull/90))
- Add ability to commit additional files ([#92](https://github.com/ploys/ploys/pull/92))
- Fix changelog generation for release with no commits ([#93](https://github.com/ploys/ploys/pull/93))
- Add changelog release generator date and URL ([#94](https://github.com/ploys/ploys/pull/94))
- Add initial package changelogs ([#99](https://github.com/ploys/ploys/pull/99))

## [0.1.0] - 2024-10-15

### Changed

- Add utility to load basic project information ([#5](https://github.com/ploys/ploys/pull/5))
- Combine local project Git error variants ([#7](https://github.com/ploys/ploys/pull/7))
- Add method to query project files ([#8](https://github.com/ploys/ploys/pull/8))
- Add method to get project file content ([#9](https://github.com/ploys/ploys/pull/9))
- Add package discovery ([#10](https://github.com/ploys/ploys/pull/10))
- Remove project name readme lookup ([#12](https://github.com/ploys/ploys/pull/12))
- Rename local and remote project variants ([#13](https://github.com/ploys/ploys/pull/13))
- Add generic project source ([#15](https://github.com/ploys/ploys/pull/15))
- Use toml_edit crate to preserve formatting ([#16](https://github.com/ploys/ploys/pull/16))
- Add package bump method ([#17](https://github.com/ploys/ploys/pull/17))
- Add project package cache ([#18](https://github.com/ploys/ploys/pull/18))
- Add project name cache ([#19](https://github.com/ploys/ploys/pull/19))
- Add project package bump method ([#20](https://github.com/ploys/ploys/pull/20))
- Move package discovery from project source ([#21](https://github.com/ploys/ploys/pull/21))
- Add initial lockfile support ([#22](https://github.com/ploys/ploys/pull/22))
- Refactor inconsistent error naming ([#23](https://github.com/ploys/ploys/pull/23))
- Add debug implementation for cargo dependencies ([#26](https://github.com/ploys/ploys/pull/26))
- Bump toml_edit to 0.22.14 ([#27](https://github.com/ploys/ploys/pull/27))
- Bump gix to 0.63.0 ([#28](https://github.com/ploys/ploys/pull/28))
- Bump gix to 0.66.0 ([#31](https://github.com/ploys/ploys/pull/31))
- Add ability to load GitHub project at a specific commit ([#34](https://github.com/ploys/ploys/pull/34))
- Add `git2` integration ([#44](https://github.com/ploys/ploys/pull/44))
- Add project source feature flags ([#45](https://github.com/ploys/ploys/pull/45))
- Add project package release methods ([#46](https://github.com/ploys/ploys/pull/46))
- Add method to get project file changes ([#49](https://github.com/ploys/ploys/pull/49))
- Add ability to load Git project at a specific commit ([#50](https://github.com/ploys/ploys/pull/50))
- Change project reference from commit to branch on release ([#51](https://github.com/ploys/ploys/pull/51))
- Change git source to create branch on current reference ([#52](https://github.com/ploys/ploys/pull/52))
- Unify project source `Reference` types as `Revision` ([#53](https://github.com/ploys/ploys/pull/53))
- Add project commit methods ([#57](https://github.com/ploys/ploys/pull/57))
- Add release workflow publish job ([#73](https://github.com/ploys/ploys/pull/73))

[0.2.0]: https://github.com/ploys/ploys/releases/tag/0.2.0
[0.1.0]: https://github.com/ploys/ploys/releases/tag/0.1.0

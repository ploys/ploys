# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2024-12-13

### Changed

- Add project `initiate_package_release` method ([#109](https://github.com/ploys/ploys/pull/109))
- Remove project `release_package` method ([#112](https://github.com/ploys/ploys/pull/112))
- Remove local git project `commit` method ([#113](https://github.com/ploys/ploys/pull/113))
- Remove `git2` integration ([#114](https://github.com/ploys/ploys/pull/114))
- Remove project source configuration ([#115](https://github.com/ploys/ploys/pull/115))
- Replace project source generic with enum ([#116](https://github.com/ploys/ploys/pull/116))
- Add project fileset for packages and lockfiles ([#117](https://github.com/ploys/ploys/pull/117))
- Rename project `Source` to `Repository` and move module ([#118](https://github.com/ploys/ploys/pull/118))
- Move lockfile types to `package` module ([#119](https://github.com/ploys/ploys/pull/119))
- Move `File` and `Fileset` types to root `file` module ([#120](https://github.com/ploys/ploys/pull/120))
- Implement `PartialEq` and `Eq` for files ([#121](https://github.com/ploys/ploys/pull/121))
- Refactor project changed files method ([#122](https://github.com/ploys/ploys/pull/122))
- Remove path from files ([#123](https://github.com/ploys/ploys/pull/123))
- Refactor project file discovery ([#124](https://github.com/ploys/ploys/pull/124))
- Move `PackageKind` to separate module ([#125](https://github.com/ploys/ploys/pull/125))
- Remove `Project::get_files` method ([#126](https://github.com/ploys/ploys/pull/126))
- Implement `Display` for files ([#128](https://github.com/ploys/ploys/pull/128))
- Add new `PackageRef` type to restore `path` method ([#129](https://github.com/ploys/ploys/pull/129))
- Rename project package release initiate to request ([#130](https://github.com/ploys/ploys/pull/130))
- Add `Remote` repository trait ([#132](https://github.com/ploys/ploys/pull/132))
- Add `Remote::update_branch` method ([#133](https://github.com/ploys/ploys/pull/133))
- Add project package release request builder ([#134](https://github.com/ploys/ploys/pull/134))
- Remove `Project::commit` method ([#135](https://github.com/ploys/ploys/pull/135))
- Remove `Project` package version mutation methods ([#136](https://github.com/ploys/ploys/pull/136))
- Remove notion of changed files ([#137](https://github.com/ploys/ploys/pull/137))
- Add `File::Changelog` enum variant ([#138](https://github.com/ploys/ploys/pull/138))
- Derive `strum::EnumIter` for `PackageKind` ([#139](https://github.com/ploys/ploys/pull/139))
- Export updated `Manifest` and `CargoManifest` types ([#140](https://github.com/ploys/ploys/pull/140))
- Remove `Fileset::get_package_by_name` methods ([#141](https://github.com/ploys/ploys/pull/141))
- Derive `EnumIs` and `EnumTryAs` for `File` ([#142](https://github.com/ploys/ploys/pull/142))
- Derive `EnumIs` and `EnumTryAs` for `Lockfile` ([#143](https://github.com/ploys/ploys/pull/143))
- Refactor `CargoLockfile` as a newtype ([#144](https://github.com/ploys/ploys/pull/144))
- Update version methods to use `Version` type ([#145](https://github.com/ploys/ploys/pull/145))
- Use `ThreadSafeRepository` in `Git` repository type ([#146](https://github.com/ploys/ploys/pull/146))
- Elide needless lifetimes to resolve `clippy` lint ([#147](https://github.com/ploys/ploys/pull/147))
- Unify `Package` and `PackageRef` types ([#148](https://github.com/ploys/ploys/pull/148))
- Replace upfront file discovery with lazy loading ([#149](https://github.com/ploys/ploys/pull/149))
- Add package changelog method ([#150](https://github.com/ploys/ploys/pull/150))
- Redesign package module and error structure ([#151](https://github.com/ploys/ploys/pull/151))
- Move project bump and not found error variants to package ([#152](https://github.com/ploys/ploys/pull/152))
- Add `to_owned` method for changelog `ReleaseRef` ([#153](https://github.com/ploys/ploys/pull/153))
- Add project package release builder ([#154](https://github.com/ploys/ploys/pull/154))
- Fix changelog `Change::to_owned` text trimming ([#155](https://github.com/ploys/ploys/pull/155))
- Remove `Project::get_file_contents` method ([#156](https://github.com/ploys/ploys/pull/156))
- Add `RepoSpec` type to specify remote repository ([#157](https://github.com/ploys/ploys/pull/157))
- Add `Project::open` constructor ([#160](https://github.com/ploys/ploys/pull/160))
- Fix project packages iterator implementation ([#163](https://github.com/ploys/ploys/pull/163))
- Fix GitHub repository file cache on 404 not found ([#164](https://github.com/ploys/ploys/pull/164))
- Redesign project and repository constructors ([#165](https://github.com/ploys/ploys/pull/165))
- Move project package release methods to package type ([#167](https://github.com/ploys/ploys/pull/167))
- Add project configuration file ([#168](https://github.com/ploys/ploys/pull/168))
- Remove unused `Fileset` type ([#169](https://github.com/ploys/ploys/pull/169))
- Add internal file loading error handling ([#170](https://github.com/ploys/ploys/pull/170))

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
- Revert "Release `0.2.0` (#101)" ([#102](https://github.com/ploys/ploys/pull/102))
- Fix missing `time` crate `parsing` and `formatting` features ([#103](https://github.com/ploys/ploys/pull/103))

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

[0.3.0]: https://github.com/ploys/ploys/releases/tag/0.3.0
[0.2.0]: https://github.com/ploys/ploys/releases/tag/0.2.0
[0.1.0]: https://github.com/ploys/ploys/releases/tag/0.1.0

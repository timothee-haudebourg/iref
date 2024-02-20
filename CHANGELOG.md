# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.1.4] - 2024-02-20

### Fixed

- [d545415] Fixes #21
- [7fb32b3] Fix incorrect extraction of URI components.

## [3.1.3] - 2023-10-20

### Fixed

- [ded66a8] Fix `README.md`

## [3.1.2] - 2023-08-23

### Fixed

- [db16295] Fix path resolution with empty segments.

## [3.1.1] - 2023-08-23

### Fixed

- [e562701] Fix Errata 4547 implementation, only on relative paths.

## [3.1.0] - 2023-08-23

### Added

- [5ef5b65] Impl `Display` for error types.

## [3.0.2] - 2023-08-23

### Added

- [47b1f24] Add `relative_to`, `suffix` and `base` to `Uri` and `Iri`.

## [3.0.1] - 2023-08-22

### Fixed

- [204b002] Fix & Test all the parsing query functions.
- [204b002] Fixes #20

## [3.0.0] - 2023-08-17

### Fixed

- [3d09f00] Fix panic in `parse_ipv6_literal`. ([#17](https://github.com/timothee-haudebourg/grdf/issues/17))
- [d054f27] Fix clippy CI.

## [2.2.3] - 2023-01-11

### Added

- [a3b99a5] Add `hashbrown` optional feature.
- [a3b99a5] Impl `hashbrown::Equivalent` for `Iri`.
- [a3b99a5] Impl `hashbrown::Equivalent` for `IriRef`.

## [2.2.2] - 2022-12-20

### Added

- [88f65d9] Add `into_string` functions.

## [2.2.1] - 2022-12-20

### Added

- [5bb37c8] Add missing `AsRef` & `Borrow` implementations.
- [4265293] Add `serde` support.

### Changed

- [dcf2dd0] Move to version 2.2.1.

### Fixed

- [fdd4855] Fix #15 IPv4/6 parser bug.

## [2.1.2] - 2022-03-23

### Fixed

- [b076269] Fix IRI reference resolution. Fixes #14

## [2.1.1] - 2022-02-24

### Fixed

- [8f51470] Fix UTF-8 decoder bug.

## [2.1.0] - 2021-12-02

### Added

- [8d80e7e] Add from_vec/string and from/to raw parts.

### Changed

- [2a1df9e] Move to version 2.0.3.

## [2.0.2] - 2021-09-09

### Changed

- [deed1ca] Move to 2.0.2.

### Fixed

- [8e97fa5] Fix #13

## [2.0.1] - 2021-09-02

### Changed

- [de909f9] Move to 2.0.1

### Fixed

- [f345670] Fix `set_query` bug.
- [4c00407] Fix rust fmt.

## [2.0.0] - 2021-09-02

### Added

- [4846893] impl AsIri/Ref for &'a T.
- [fd6554b] Add CI to run test, rustfmt and clippy on push/PR
- [56f18b0] Add CI to run test, rustfmt and clippy on push/PR
- [350ae25] Add from_str and to_owned methods
- [538fc8a] Add from_str and to_owned methods
- [7a43a39] Impl `Eq` for `Error`.
- [a3e8e33] Add tests for fragment parsing issue.

### Changed

- [63cbec1] Move to 1.4.0
- [cabafde] Move to 2.0.0

### Fixed

- [efbf824] Fix `IriRef::relative_to`. Version 1.4.1.
- [5d42595] Fix `IriRef::relative_to` again + proper tests.
- [d539b60] Fix corner case for `IriRef::relative_to`
- [5aae749] Fixing some clippy warnings.
- [4f0423a] Fix typo to link to correct type
- [7bfb545] Fix FUNDING.yml
- [fbfaaa5] Fixes #12
- [64d2642] Fix doc link.

### Removed

- [82cc03d] Remove warnings.

## [1.3.0] - 2020-10-02

### Added

- [44b4f08] Impl From<Path> for IriRef.
- [693da03] Impl From<&PathBuf> for IriRef.
- [0805e2a] Add IriRef::relative_to.

## [1.2.0] - 2020-09-10

### Added

- [4e8d67f] Implement Clone and Error for Error enum
- [3c6c077] Implement Clone and Error for Error enum
- [9493dc9] Add a changelog. Move to version 1.2.0.

## [1.1.4] - 2020-04-19

### Added

- [6de9a3a] Add a new test catching issue #2.

### Changed

- [c2f57e5] Move to version 1.1.4.

### Fixed

- [b6d9389] Fix the path/segment parser.
- [c541e07] Fix#2

## [1.1.3] - 2020-03-31

### Added

- [6277102] Add into_* methods

### Changed

- [8b2541f] Move to version 1.1.3

### Fixed

- [df9cbfd] Fix lifetimes

## [1.1.2] - 2020-03-31

### Changed

- [c2af478] Move to 1.1.2

## [1.1.1] - 2020-03-31

### Added

- [ebf37fa] Add infos about `static-iref`.

## [1.1.0] - 2020-03-31

### Added

- [b705b91] Add methods to inspect and build IRIs in `static-iref`.

### Changed

- [b614ead] Move to version 1.1

### Fixed

- [1d6adc7] Fix typo.

## [1.0.1] - 2020-03-16

### Removed

- [d3de7bb] Remove build files

## [1.0.0] - 2020-03-16

### Added

- [043a15e] Add gitignore.

### Changed

- [3afe663] Move percent-encoded strings in a dedicated crate.
- [de90577] Refactoring.
- [8f94dac] Refactoring.

### Removed

- [d76fd66] Remove useless files.
- [3f05716] Remove warnings.


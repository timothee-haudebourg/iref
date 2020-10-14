# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [1.4.0] - 2020-10-14
### Added
- `AsIri` and `AsIriRef`

## [1.3.0] - 2020-10-02
### Added
- `Path::into_ref`
- `PathBuf::as_ref, into_bytes`
- `IriRefBuf::into_raw_parts, into_bytes, as_ref`
- Convertions operations between `Path`/`PathBuf` and `IriRef`/`IriRefBuf`.
- `IriRef::relative_to`

### Changed
- `#[inline]` almost all the API.

## [1.2.0] - 2020-09-10
### Added
- A `CHANGELOG.md` file.
- Implementation of `Clone`, `Display` and `std::error::Error` for the `Error` type.

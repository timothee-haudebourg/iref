# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.1] - 2021-09-02
### Changed
- Fix `set_query` bug that used the `:` character instead of `?`.

## [2.0.0] - 2021-09-02 [YANKED]
### Yanking reason
- Found a bug in the `set_query` function that uses the `:` character
  instead of `?` to prefix the query. Introduced by #11.

### Changed
- Fix #12 by checking that the entire input buffer has been parsed.
- Rename all inherent `as_ref` methods into `as_bytes`
  for `Iri`, `IriRef`, `Authority`, `Fragment`, `Host`,
  `Path`, `Port`, `Query`, `Scheme`, `Segment` and `UserInfo`.
- Rename `IriRef::into_ref` and `Path::into_ref` into `into_bytes`.
- No more clippy warnings!

### Added
- Proper `AsRef<[u8]>` impl for `IriRef`, `Authority`,
  `Fragment`, `Host`, `Path`, `Port`, `Query`, `Scheme`,
  `Segment` and `UserInfo`.

## [1.4.3] - 2020-10-16
### Changed
- Fixed corner case `IriRef::relative_to`

## [1.4.2] - 2020-10-15
### Added
- Actual test for `IriRef::relative_to`

### Changed
- Fixed `IriRef::relative_to`

## [1.4.1] - 2020-10-15
### Added
- `IriRef::base`
- `Path::len` and `Path::closed_len`

### Changed
- Fixed `IriRef::relative_to`

## [1.4.0] - 2020-10-14
### Added
- `AsIri` and `AsIriRef`

## [1.3.1] - 2020-10-02
### Changed
- Use generic `Into<IriRef>` type parameter in `IriRef::suffix` and `IriRef::relative_to`.

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

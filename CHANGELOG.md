# Change Log
All notable changes to this project will be documented in this file

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Load CLI arguments via `yaml` file
- `GIError` type wrapper for better error handling
- New `pretty_print` function outputs all templates separated in 3 columns

### Changed
- Split certain logics into functions (mostly HTTP requests, though some further work needs to be done)
- `request_api` now carries the hardcoded API link. A specific template can be fetched by specifying the name as an `Option<String>`
- `req` as alias for `reqwest`
- New `get_bodies` function handles the tokio tasks

## [0.1.0] - 2021-08-03
### Added
- Project upload (first release)

### [Unreleased](https://github.com/appositum/gitignore/compare/0.1.0...dev)
### [0.1.0](https://github.com/appositum/gitignore/releases/tag/0.1.0)

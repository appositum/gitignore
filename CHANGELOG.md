# Change Log
All notable changes to this project will be documented in this file

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
### Changed
### Removed

## [0.2.0] - 2021-08-10
### Added
- Load CLI arguments via `yaml` file, better arg parse with `clap`
- `GIError` type wrapper for better error handling
- New `pretty_print` function outputs all templates separated in 3 columns
- `Template` and `TemplateList` structs for json deserialization

### Changed
- `request_api` now carries the hardcoded API link. A specific template can be fetched by specifying the name as an `Option<String>`
- New `get_templates` function handles the tokio tasks

## [0.1.0] - 2021-08-03
### Added
- Project upload (first release)

### [Unreleased](https://github.com/appositum/gitignore/compare/0.2.0...dev)
### [0.2.0](https://github.com/appositum/gitignore/releases/tag/0.2.0)
### [0.1.0](https://github.com/appositum/gitignore/releases/tag/0.1.0)

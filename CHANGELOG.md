# Change Log
All notable changes to this project will be documented in this file

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [1.2.0] - 2025-12-22
### Added
- `search` arg option
### Changed
- Number of columns when listing templates now adapts to terminal size

## [1.1.0] - 2025-12-11
### Added
- `append` now adds newline before inserting text if the gitignore file exists and is not empty
- Extra newline between different template sections
### Changed
- Fixed helper for `output` arg option

## [1.0.0] - 2025-12-10
### Added
- `Cargo.lock` file
### Changed
- Bumped `clap` version from 2 to 4
- Template list input is now space-separated instead of comma-separated
### Removed
- `yaml` feature from `clap` (deprecated in version 4)

## [0.3.0] - 2025-12-08
### Added
- Implement output redirection (append, overwrite or output to new file)
- Implement GitHub API authentication with personal token via `$GITHUB_TOKEN` to avoid exceeding rate limits
- Case insensitivity for template list input
### Changed
- Refactor main function to pretty print errors (`tokio::main` is now in a separate function called `run`)
- Move CLI args treatment to separate file (`cli.rs`)
- Move `tokion::main` function to `lib.rs`
- Pretty print `TemplateNotFound` error

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

### [Unreleased](https://github.com/appositum/gitignore/compare/1.2.0...dev)
### [1.2.0](https://github.com/appositum/gitignore/releases/tag/1.2.0)
### [1.1.0](https://github.com/appositum/gitignore/releases/tag/1.1.0)
### [1.0.0](https://github.com/appositum/gitignore/releases/tag/1.0.0)
### [0.3.0](https://github.com/appositum/gitignore/releases/tag/0.3.0)
### [0.2.0](https://github.com/appositum/gitignore/releases/tag/0.2.0)
### [0.1.0](https://github.com/appositum/gitignore/releases/tag/0.1.0)

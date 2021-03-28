## cycler v0.3.0
- Removed rust fmt linting

## cycler v0.2.0
- Removed commented out code
- Added more default build functions
- Changed traits module to be exported out of the main module
- Added the `allow_unsafe` feature that is required for all unsafe code
  - Unsafe is denied if `allow_unsafe` is not enabled

### cycler v0.1.3
- Added github actions to verify with clippy and miri
- Added github actions to run tests
- Ran rustfmt on the whole repo and added rustfmt.toml

### cycler v0.1.2
- Fixed use after free bug present when `unsafe_cleanup` is enabled
  - This was due to drop ordering being incorrect in `RwLockCyclerWriter` and`RwLockCyclerReader`
- Yanked v0.1.1 as this had a use after free bug by default

### cycler v0.1.1
- Added documentation to `unsafe_cleanup` feature and turned it on by default

# cycler v0.1.0
- Initial Version!

## cycler v0.1.2
- Fixed use after free bug present when `unsafe_cleanup` is enabled
  - This was due to drop ordering being incorrect in `RwLockCyclerWriter` and`RwLockCyclerReader`
- Yanked v0.1.1 as this had a use after free bug by default

## cycler v0.1.1
- Added documentation to `unsafe_cleanup` feature and turned it on by default

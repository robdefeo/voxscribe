# Changelog

All notable changes to this project will be documented in this file.
## [0.2.0] - 2026-04-24

### Bug Fixes

- Allow dirty CI file to permit manual additions to release.yml (#16) ([1287453](https://github.com/robdefeo/voxscribe/commit/1287453e87c37dafec9e88eb7533820acbf7bbd2))
- Publish Homebrew formula to tap in announce job (#14) ([dd3471a](https://github.com/robdefeo/voxscribe/commit/dd3471a91ea5972c10eac4a38a286457d9ced215))
- Release docs and version-guard cargo pkgid format (#12) ([c591516](https://github.com/robdefeo/voxscribe/commit/c591516b436d46a9c3e72fec2651fc8e06e601b4))
- Remove empty ignore_tags and skip_tags from cliff.toml (#9) ([3f36ea0](https://github.com/robdefeo/voxscribe/commit/3f36ea0e04405b032512781a1bf20bb36fcdc2c1))

### Features

- Route whisper.cpp logs through tracing (#17) ([54064c5](https://github.com/robdefeo/voxscribe/commit/54064c5700b770780b562958784b5c020debd305))
- Enforce CHANGELOG.md on release commits (#11) ([7f587b4](https://github.com/robdefeo/voxscribe/commit/7f587b45ac7a3548cccf69b075a565a92e77f69e))
- Show transcription progress percentage and elapsed time (#6) ([f55c77d](https://github.com/robdefeo/voxscribe/commit/f55c77dc604aec22b26a81444ea222ab85723303))
- Add WebVTT output format (#7) ([27587e2](https://github.com/robdefeo/voxscribe/commit/27587e2cf5a405100f72510c236cce504105cc77))
- Use HF Hub cache for model discovery and auto-download (#2) ([0079f88](https://github.com/robdefeo/voxscribe/commit/0079f88484ee6053e8c282538092a7659df6e56e))
- Initial voxscribe implementation ([80e7b04](https://github.com/robdefeo/voxscribe/commit/80e7b0471ea0c64ffbfdee1256640a9dc5d35e75))

### Refactor

- Replace ffmpeg shell-out with pure Rust audio decoding (#15) ([599e2b2](https://github.com/robdefeo/voxscribe/commit/599e2b2b9ce956ff77076b92572dca11ed3a9f1e))

### Testing

- Align exclusions with policy (#1) ([2ccd62c](https://github.com/robdefeo/voxscribe/commit/2ccd62c13f67032bcffe0914165897809eca877d))


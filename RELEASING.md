# Releasing

Releases are driven by pushing a version tag. cargo-dist builds binaries for all targets, publishes a GitHub Release with changelog notes, and updates the Homebrew tap automatically.

## Prerequisites

First-time setup only:

- The `robdefeo/homebrew-tap` GitHub repo must exist and cargo-dist must have push access (via `GITHUB_TOKEN` — this is automatic for repos in the same account).
- Confirm the release workflow is healthy by checking a recent CI run on `main`.

## Steps

### 1. Bump the version

Edit `Cargo.toml`:

```toml
version = "x.y.z"
```

Commit directly to `main`:

```bash
git add Cargo.toml
git commit -m "chore: bump version to x.y.z"
git push
```

### 2. Tag the release

```bash
git tag vx.y.z
git push origin vx.y.z
```

This triggers the Release workflow. cargo-dist will:

- Build binaries for macOS (arm64, x86_64), Linux (x86_64), and Windows (x86_64)
- Generate release notes from commits since the last tag using git-cliff
- Create a GitHub Release at https://github.com/robdefeo/voxscribe/releases
- Publish the shell installer and update the Homebrew formula in `robdefeo/homebrew-tap`

### 3. Verify

- [ ] GitHub Release created with correct version and changelog
- [ ] Shell installer works: `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/robdefeo/voxscribe/releases/latest/download/voxscribe-installer.sh | sh`
- [ ] Homebrew formula updated in `robdefeo/homebrew-tap`

## Versioning

Follow [Semantic Versioning](https://semver.org):

- `patch` — bug fixes, no behaviour change
- `minor` — new functionality, backwards compatible
- `major` — breaking changes

## Troubleshooting

**Release workflow failed** — check the Actions tab. Re-running is safe; pushing the same tag again requires deleting it first:

```bash
git tag -d vx.y.z
git push origin :refs/tags/vx.y.z
# fix the issue, then re-tag
git tag vx.y.z
git push origin vx.y.z
```

**Homebrew formula not updated** — cargo-dist pushes to `robdefeo/homebrew-tap` as part of the workflow. If it fails, check that the repo exists and the workflow has write access.

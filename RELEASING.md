# Releasing

Releases are driven by pushing a version tag from a local release commit. cargo-dist builds binaries for all targets, generates release notes via git-cliff, publishes a GitHub Release, and updates the Homebrew tap automatically.

`main` always keeps `version = "0.0.0"` in `Cargo.toml`. This is enforced by a lefthook pre-commit hook and a CI check on push to `main`. Release commits are never merged back to `main` — they are only reachable via their tag.

## Prerequisites

First-time setup only:

- The `robdefeo/homebrew-tap` GitHub repo must exist and cargo-dist must have push access (via `GITHUB_TOKEN` — this is automatic for repos in the same account).
- Confirm the release workflow is healthy by checking a recent CI run on `main`.

## Steps

### 1. Create a local release commit

From the latest `main`, create a local branch (never pushed), bump the version, and commit:

```bash
git fetch origin
git checkout -b release/vx.y.z origin/main

# Edit Cargo.toml: set version = "x.y.z"

git commit -am "chore: release vx.y.z"
```

### 2. Tag and push the tag only

```bash
git tag vx.y.z
git push origin vx.y.z
```

Do not push the branch. The release commit is reachable only via the tag.

### 3. What happens next

The release workflow fires on the tag. cargo-dist will:

- Validate that the tag version matches `Cargo.toml` — fails fast if they diverge
- Build binaries for macOS (arm64, x86_64), Linux (x86_64), and Windows (x86_64)
- Generate release notes from commits since the last tag using git-cliff
- Create a GitHub Release at https://github.com/robdefeo/voxscribe/releases
- Publish the shell installer and update the Homebrew formula in `robdefeo/homebrew-tap`

### 4. Verify

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

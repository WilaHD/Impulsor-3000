# Release automation

This project uses semantic versioning through `release-plz` and Conventional
Commits.

## Commit messages

Use these commit prefixes on changes that should affect the next version:

- `fix: ...` for patch releases, for example `0.2.2 -> 0.2.3`
- `feat: ...` for feature releases, for example `0.2.2 -> 0.3.0`
- `feat!: ...` or a `BREAKING CHANGE:` footer for major releases

## GitHub secret

Create a GitHub Actions secret named `RELEASE_PLZ_TOKEN`.

Create a fine-grained personal access token:

1. Open your GitHub account settings.
2. Go to `Developer settings` -> `Personal access tokens` -> `Fine-grained tokens`.
3. Choose `Generate new token`.
4. Select the repository owner and this repository.
5. Set an expiration date.
6. Set repository permissions:
   - Contents: read and write
   - Pull requests: read and write

If the repository uses protected tags or tag rules, the token also needs enough
permission to create tags that match `v*`, or the tag rules need to allow this
token to create them.

Also check repository Actions settings:

1. Open repository `Settings`.
2. Go to `Actions` -> `General`.
3. Under `Workflow permissions`, allow read and write permissions.
4. Enable `Allow GitHub Actions to create and approve pull requests`.

Add the token in GitHub:

1. Open the repository on GitHub.
2. Go to `Settings`.
3. Go to `Secrets and variables` -> `Actions`.
4. Choose `New repository secret`.
5. Name it `RELEASE_PLZ_TOKEN`.
6. Paste the token value and save it.

If `release-plz release-pr` fails with HTTP 422, inspect the lines above the
summary in the workflow log. Common causes are missing pull request creation
permission, tag/branch protection rules, or an existing release PR branch that
GitHub rejects. The workflow enables debug logging for `release-plz` to make the
GitHub validation reason visible.

## Release flow

1. Merge normal feature/fix commits into `main`.
2. `release-plz` opens or updates a release PR.
3. Review and merge the release PR.
4. `release-plz` creates a `vX.Y.Z` tag.
5. The existing tag-based release workflow builds and publishes:
   - Windows installer
   - Linux AppImage
   - macOS app zip

## Prerelease flow

Use the manual `Prerelease` GitHub Actions workflow when you want a prerelease.

1. Open the repository on GitHub.
2. Go to `Actions`.
3. Select `Prerelease`.
4. Choose `Run workflow`.
5. Enter a semantic prerelease version, for example:
   - `0.3.0-alpha.1`
   - `0.3.0-beta.1`
   - `0.3.0-rc.1`

The workflow creates an annotated tag like `v0.3.0-beta.1`. The tag triggers
the normal release workflow. Because the tag contains `-`, the GitHub release
is marked as a prerelease.

## Version propagation

`Cargo.toml` is the canonical version source for stable releases.

For tag builds, including prereleases, the release workflow uses the tag version
first. This allows a tag like `v0.3.0-beta.1` to produce assets labeled
`0.3.0-beta.1` without changing `Cargo.toml` to a prerelease version.

The release workflow reads that version and applies it to:

- the build-time `IMPULSOR_APP_VERSION` value embedded in the binary
- the macOS `Info.plist` inside the built `.app`
- the Windows executable resource metadata
- the Windows NSIS installer metadata and filename
- the AppImage desktop metadata and filename
- the GitHub release asset filenames

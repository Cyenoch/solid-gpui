# Publishing npm packages

Applications install `@solid-gpui/core`, `@solid-gpui/vite`, and the optional
`@solid-gpui/router` and `@solid-gpui/shiki` from npm. All four packages share one
release version with the Rust workspace. Scoped packages declare public access
and the npm registry in `publishConfig`.

The [Publish npm packages workflow](../.github/workflows/npm-publish.yml) runs
when a `vMAJOR.MINOR.PATCH` tag is pushed. It verifies committed versions,
builds and tests packages, audits npm dependencies, then publishes the verified
archives through npm Trusted Publishing (OIDC). No npm token is stored in GitHub.
Only stable, nonzero versions are accepted; prerelease tags are rejected.

## One-time configuration

You need an npm account with publish access to the `@solid-gpui` scope. If the
packages do not exist on npm yet, complete the first publication manually as
described below; npm's package settings must exist before configuring a trusted
publisher. Use the real checked release, not placeholder packages.

In GitHub repository **Settings → Environments**, create an environment named
`npm`. Under deployment branches and tags, allow release tags matching `v*`.
Required reviewers are not needed for automatic publication.

For **each** of `@solid-gpui/core`, `@solid-gpui/vite`, `@solid-gpui/router`, and
`@solid-gpui/shiki`, open npm **Settings → Trusted publishing**, add GitHub Actions,
and enter these exact values:

| Field | Value |
| --- | --- |
| Organization or user | `Cyenoch` |
| Repository | `solid-gpui` |
| Workflow filename | `npm-publish.yml` |
| Environment name | `npm` |
| Allowed actions | Enable direct `npm publish` |

The workflow filename is not a path. Fields are case-sensitive. New npm trusted
publishers allow staged publishing by default; explicitly permit `npm publish`
for this automatic workflow. No GitHub `NPM_TOKEN` or `NODE_AUTH_TOKEN` secret is
required. The workflow uses GitHub-hosted runners, Node 24, and pinned npm 11.19.0
(OIDC requires npm 11.5.1+ and Node 22.14.0+).

See the [npm trusted-publisher reference](https://docs.npmjs.com/trusted-publishers/)
for the current settings UI and requirements.

## Prepare the release

Add the intended `MAJOR.MINOR.PATCH` section to `CHANGELOG.md`, then run from the
SDK checkout, replacing `VERSION` with that version:

```sh
bun install --frozen-lockfile
bun run task release-prep VERSION
bun run ci
bun run audit
```

`release-prep` synchronizes the Rust workspace, all four npm versions, and their
core peer ranges. It updates and validates locks and regenerates dependency
notices. The CI gate builds packages and runs the packed-consumer smoke: actual
archives are installed in a temporary application to verify exports, types,
Vite builds, native binding export, and the public test runner. Its minimal Rust
exporter does not qualify native windows or platform rendering.

Commit the release changes and merge them into `main`. Tag that checked commit:

```sh
git tag vVERSION
git push origin vVERSION
```

The tag starts the npm workflow. CI validates the tag against all four manifests,
core peer ranges, the Rust workspace version and changelog without modifying any
of them. Its package job runs `package-ci` and `bun audit`, then uses `npm pack`
to upload four versioned archives. Only the separate `publish` job has
`id-token: write`; it downloads this run's archives and publishes core before its
integrations under `latest`, with provenance. Publishing runs no package lifecycle
scripts and installs no application dependencies. Release runs are serialized
and an active publication is not cancelled by a newer tag.

The tag also identifies the matching Rust sources, vendor patches, and workspace
profiles. JavaScript package installation does not install those native inputs.
The automated npm gate checks JS packages and minimal Rust exporter fixtures;
it does not replace the native CI and release qualification required before
tagging. It does not create native application releases.

## First publication

The preceding checks build `dist`. From that same checked release checkout,
inspect npm's package contents:

```sh
for package in solid-gpui solid-gpui-vite solid-gpui-router solid-gpui-shiki; do
  (cd "packages/$package" && npm pack --dry-run) || exit 1
done
```

Packages include their built JavaScript/declarations, explicit source exports,
README and license. They have no install hook that compiles a native host.
The source exports support SDK debugging and this repository's examples; normal
application imports use `dist`.

Push the first release tag and let the workflow's package job finish. The publish
job cannot authenticate until the packages and trusted publishers exist. Download
the `npm-release` artifact from that run and extract it. On a machine with npm
installed, authenticate interactively and publish those exact archives, core
first. From the extracted directory, set `VERSION` to the release version:

```sh
npm login
VERSION=0.3.0
for package in core vite router shiki; do
  npm publish "solid-gpui-$package-$VERSION.tgz" --access public --ignore-scripts || exit 1
done
```

npm performs any required account verification. These commands create the package
settings and publish under `latest`; the first manual publication has no GitHub
OIDC provenance. Configure all four trusted publishers, then re-run the failed
publish job: it recognizes the identical archives and completes without
republishing. Subsequent version tags publish through OIDC automatically.
Confirm all four versions before announcing the release:

```sh
npm view @solid-gpui/core version
npm view @solid-gpui/vite version
npm view @solid-gpui/router version
npm view @solid-gpui/shiki version
```

## Retry a partial publication

Four npm publishes are not atomic. If publication stops partway, use GitHub's
**Re-run failed jobs** on the same workflow run while its package artifact is
available (seven days). The publisher checks all four registry versions first,
skips only versions whose `dist.integrity` matches the exact archive bytes, and
publishes missing versions. A different existing archive or any registry error
other than HTTP 404 stops publication. Do not move the source tag or rebuild a
partly published release expecting different bytes to be accepted.

The manual Release Prep workflow still verifies and uploads candidates without
publishing. Local archive inspection and packed-consumer tests are release
verification, rather than a separate application installation workflow. There
are no custom per-package pack commands.

## Consume the release

Follow [Getting started](getting-started.md). Pin the SDK packages to the same
published version, commit the application lockfile, and use the matching source
tag or exact commit for the native host. Rebuild the host and regenerate bindings
when upgrading the SDK. Bun remains the development/runtime tool even though npm
is the package registry.

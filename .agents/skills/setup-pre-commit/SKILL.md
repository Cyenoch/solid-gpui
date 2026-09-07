---
name: setup-pre-commit
description: Set up Husky pre-commit hooks with lint-staged (Oxfmt), type checking, and tests. Use when adding pre-commit hooks, configuring Husky or lint-staged, or adding commit-time checks.
---

# Setup Pre-Commit Hooks

## What This Sets Up

- **Husky** pre-commit hook
- **lint-staged** running Oxfmt on staged files
- **Oxfmt** config (if missing)
- **typecheck** and **test** scripts in the pre-commit hook

## Steps

### 1. Detect package manager

Use the repository's `packageManager` field and lockfile: `package-lock.json` (npm), `pnpm-lock.yaml` (pnpm), `yarn.lock` (Yarn), or `bun.lock` (Bun). Match its existing command conventions in the steps below.

### 2. Install dependencies

Install as devDependencies:

```
husky lint-staged oxfmt
```

### 3. Initialize Husky

```bash
npx husky init
```

This creates `.husky/` and adds `prepare: "husky"` to package.json. Preserve existing hooks and compose with an existing `prepare` script.

### 4. Create `.husky/pre-commit`

Write this file (no shebang needed for Husky v9+):

```
npx lint-staged
npm run typecheck
npm run test
```

**Adapt**: Replace `npm` with detected package manager. If repo has no `typecheck` or `test` script in package.json, omit those lines and tell the user.

### 5. Configure lint-staged

Update the existing lint-staged configuration, or create `.lintstagedrc` if none exists:

```json
{
  "*": "oxfmt --no-error-on-unmatched-pattern"
}
```

Oxfmt writes by default. The flag allows a staged file set with no supported matching files. Keep other staged checks intact. See the [official pre-commit setup](https://oxc.rs/docs/guide/usage/formatter/quickstart#pre-commit-with-lint-staged).

### 6. Configure Oxfmt

Reuse the existing Oxfmt configuration. If migrating an existing Prettier configuration, run `oxfmt --migrate=prettier` with the package manager's local binary runner, then inspect the converted options and ignore patterns. Otherwise, initialize `.oxfmtrc.json` with `oxfmt --init` and match the repository's established formatting conventions.

A minimal configuration is:

```json
{
  "$schema": "./node_modules/oxfmt/configuration_schema.json"
}
```

Keep formatter options in the config, shared by hooks, editors, and CI. Use `oxfmt --check` for checks. Oxfmt's default print width is 100; preserve an existing width explicitly when migrating. See [configuration](https://oxc.rs/docs/guide/usage/formatter/config) and the [migration guide](https://oxc.rs/docs/guide/usage/formatter/migrate-from-prettier).

### 7. Verify and report

- [ ] `.husky/pre-commit` exists and is executable
- [ ] The existing or new lint-staged config invokes Oxfmt
- [ ] The `prepare` script initializes Husky and preserves any prior setup
- [ ] Oxfmt configuration matches the repository's formatting conventions
- [ ] Run the local Oxfmt `--check` command on the changed configuration and supported files
- [ ] Run the configured typecheck and test commands
- [ ] If supported files are already staged, run the local lint-staged command to exercise the hook's formatter step

Report the configured checks and their results. Leave staging and commits to the user's requested workflow.

## Notes

- Husky v9+ doesn't need shebangs in hook files
- The pre-commit runs lint-staged first (fast, staged-only), then full typecheck and tests

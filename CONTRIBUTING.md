# Contributing to forge

## Branches

```
main      ●───────────────●───────────●──────────   stable: every commit here is a released version
           \             /           / \
dev         ●──●───●────●───●───●───●───●───────   integration: next release is built here
                \ /        \   /
feature/…        ●          ●─●                     one branch per feature or fix
```

| Branch | Purpose | Branches off | Merges into |
|---|---|---|---|
| `main` | Stable. Only released code. Every release is a tag (`v0.3.0`) on `main`. | – | – |
| `dev` | Integration. Finished features land here; this is what the next release will contain. | `main` | `main` (as a release) |
| `feature/<name>` | A new feature, e.g. `feature/json-parser` | `dev` | `dev` |
| `fix/<name>` | A bug fix, e.g. `fix/router-trailing-slash` | `dev` | `dev` |
| `chore/<name>`, `docs/<name>`, `refactor/<name>`, `test/<name>`, `ci/<name>`, `perf/<name>` | Everything else | `dev` | `dev` |
| `hotfix/<name>` | Urgent fix for a released version | `main` | `main` (then synced back to `dev` automatically) |

The **PR checks** workflow enforces this: PRs into `main` must come from `dev` or `hotfix/*`, and PRs into `dev` must use one of the prefixes above.

## Commit messages and PR titles

Use [Conventional Commits](https://www.conventionalcommits.org). The release workflow reads them to choose the next version and write the changelog.

```
<type>(<scope>): <description>

feat(http): add keep-alive support
fix(json): handle escaped quotes in keys
perf(router): match static segments first
docs: explain module imports
feat(core)!: rename Module::routes to Module::controllers
```

- **Types:** `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `style`, `revert`
- **Scope** (optional): the crate without the `forge-` prefix: `core`, `http`, `json`, `di`, `macros`, `validation`, `config`
- **Breaking change:** add `!` after the type/scope, or a `BREAKING CHANGE:` line in the commit body

## Day-to-day workflow

```bash
git checkout dev
git pull
git checkout -b feature/json-parser

# ...work, commit...

bash scripts/ci.sh
git push -u origin feature/json-parser
```

Then open a PR into `dev` with a Conventional Commit title and **squash merge** it. The squashed commit uses the PR title, so `dev` gets one clean commit per feature.

## Releasing

1. Open a PR from `dev` into `main` titled e.g. `chore: release`. Merge it with a **merge commit** (not squash), so each feature commit reaches `main` and appears in the changelog.
2. Go to **Actions → Release → Run workflow** on `main`, and leave the bump on `auto`.

The workflow then:
1. runs every check (`scripts/ci.sh`)
2. picks the next version from the commits since the last tag
3. updates the version in `Cargo.toml` (all crates share one version) and `Cargo.lock`
4. adds a section to `CHANGELOG.md`
5. commits `chore(release): vX.Y.Z`, tags it `vX.Y.Z` and pushes
6. creates a GitHub Release with the changelog section as notes
7. publishes every crate to crates.io (only if the `CARGO_REGISTRY_TOKEN` secret is set)
8. merges `main` back into `dev`

### How `auto` picks the version

| Commits since last release | 1.0.0 and later | Before 1.0.0 |
|---|---|---|
| Any breaking change (`!` or `BREAKING CHANGE`) | major (`2.0.0`) | minor (`0.3.0`) |
| Any `feat` | minor (`1.3.0`) | patch (`0.2.1`) |
| Only `fix` / `perf` | patch (`1.2.1`) | patch (`0.2.1`) |
| Nothing else | fails: nothing to release | fails: nothing to release |

Before 1.0, Cargo treats `0.x` → `0.(x+1)` as a breaking change, so breaking changes bump the minor number. Choose `major` by hand when you're ready for `1.0.0`.

The very first release (no tags yet) uses the version already in `Cargo.toml`.

## Rules

- **Zero external crates.** `scripts/check-no-deps.sh` fails CI if anything outside this repo appears in `cargo tree`. Exceptions must be agreed and listed in `ROADMAP.md`.
- `cargo fmt` and `cargo clippy -- -D warnings` must pass.
- Add tests for new behaviour.

Run everything CI runs with:

```bash
bash scripts/ci.sh
```

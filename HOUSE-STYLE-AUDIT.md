# House-Style Audit

Audit date: 2026-07-22

Audited against: `~/code/house-style`, especially `agents.md`, `ci.md`, `commits.md`, `dependencies.md`, `distribution.md`, `docs.md`, `licensing.md`, `rust-cli.md`, `scripts.md`, `security.md`, `specs.md`, and `testing.md`.

Scope: release-readiness static audit of `origin/main` at `12cedb6` after PR #38. GitHub repository settings, the current public `v0.10.30` release, and release automation were inspected. No project files were changed except this report. Cargo, docs, and live-forum tests were not run because this checkout has an ignored live test configuration and the audit was non-mutating.

## Work Tracking

Each finding has a stable `HSA-<priority>-<number>` code. Checkboxes track completed implementation work; branch and external-service prerequisites are recorded beside the relevant finding. The evidence records the audit commit's state.

## Summary

`dsc` has the substance for a 1.0 release: a real field-driven product, a clear sync workflow, mature documentation, an existing five-platform cargo-dist release, crates.io publishing, generated changelogs, and a useful security/disclosure policy.

It is **not ready to announce 1.0 yet**, because several public safety and release-process promises are not reliably enforced. These are finite, high-value fixes rather than missing feature work. Complete the P1s, decide the public compatibility boundary, run a clean release rehearsal, then a Meta launch is appropriate.

Main improvements:

- Make the global `--dry-run` promise true for every mutating path.
- Move release authority out of an unprotected local tag workflow, and land the pending crates.io Trusted Publishing migration.
- Publish an explicit 1.0 contract for the CLI, Rust library, and supported Discourse versions.
- Correct the public `harden` claim and resolve third-party asset provenance before broad promotion.

## Priority Findings

### [x] `HSA-P1-01` - The documented global dry-run safety guarantee is false

Status (2026-07-23): implemented and verified on `feat/r30-dry-run-guard` at `935aa6a`; it still requires merge to `origin/main`.

Evidence:

- `spec/cli-design.md` says every mutating command previews a complete plan while touching nothing; `AGENTS.md` and `SECURITY.md` make the same operator-facing claim.
- `src/main.rs` obtains `cli.dry_run` but dispatches `update_one` / `update_all` without it. `src/commands/update.rs` performs `apt` upgrades, a reboot, and `./launcher rebuild`.
- Other mutating paths that do not receive the global flag include `emoji push`, `backup create`, `palette push`, `theme push` / `duplicate`, `upload`, `topic sync`, and local config writes through `add`, `import`, and `list tidy`.
- `theme update --dry-run` sends a remote `PUT` for `remote_check`, so it also violates the literal no-write guarantee.

House style:

- `rust-cli.md` and the local CLI design require every mutation to honour `--dry-run` and leave remote state untouched.

Suggested change:

- Treat this as a safety bug, not documentation debt. Audit every mutation, thread a plan-only mode to it, and add a table-driven regression test that proves each mutator either receives `dry_run` or is explicitly read-only.
- For operations that cannot provide a meaningful plan yet, make `-n` refuse safely rather than executing.
- Use a disposable test forum or a mocked HTTP/SSH boundary to prove no request or command with a side effect is sent.

### [ ] `HSA-P1-02` - The 1.0 release path can publish from an unprotected local tag

Evidence:

- GitHub reports that `main` has neither branch protection nor a ruleset.
- `s/version++` creates a local release commit and annotated tag, then pushes `main` and the tag. It does not reject untracked files, verify it is synchronised with `origin/main`, or abort on `cargo update --workspace` failure (`|| true`). A non-fast-forward `main` push can therefore leave a local release commit/tag behind.
- `.github/workflows/release.yml` is tag-driven and sets workflow-wide `contents: write`, including its `pull_request` execution path.
- The public release build does not run the full test suite; it relies on `main` already being safe.

House style:

- `distribution.md` identifies this local-tag model as a legacy exception and sets the protected-main CI auto-tag cascade as the target model.
- `ci.md` requires least-privilege workflow permissions and read-only build/test jobs.

Suggested change:

- Before 1.0, enable protection for `main` and migrate to `s/version++` landing a release commit/PR, an `auto-tag.yml` workflow, and a `workflow_call`-capable release workflow. The tag must be created only after the reviewed commit reaches `main`.
- Set workflow defaults to `contents: read`; give write permissions only to the tag/release job. Keep release PR and build jobs read-only.
- Until that migration is complete, harden `s/version++`: use porcelain status (including untracked files), require `HEAD == origin/main`, fail on failed lockfile refresh, and document a recovery path for a partially pushed release.

### [ ] `HSA-P1-03` - Crates.io Trusted Publishing is ready locally but not part of the published release path

Remediation status (2026-07-23):

- [x] `HSA-P1-03A` Replace the long-lived GitHub secret with the SHA-pinned crates.io OIDC action, job-scoped `id-token: write`, and a read-only checkout. The workflow now runs locked release validation with live-forum tests disabled.
- [x] `HSA-P1-03B` Configure crates.io Trusted Publishing for repository `pacharanero/dsc`, workflow `.github/workflows/publish-crates.yml`, and its GitHub Environment if one is selected. Confirmed by the maintainer on 2026-07-23.
- [ ] `HSA-P1-03C` Make the first successful OIDC publication, then remove `CARGO_REGISTRY_TOKEN` from GitHub.

Evidence:

- `origin/main:.github/workflows/publish-crates.yml` still exports `secrets.CARGO_REGISTRY_TOKEN`.
- This checkout contains an uncommitted Trusted Publishing migration using `rust-lang/crates-io-auth-action` and job-level OIDC permission. It has not reached `origin/main`.
- The publish workflow tests only `cargo test --lib`, whereas `s/test-fmt-clippy` is the repository's full local gate.

House style:

- `distribution.md` requires one reliable release path. `ci.md` requires least privilege and the same meaningful quality gates locally and in CI.

Suggested change:

- Land and review the OIDC workflow before the 1.0 tag; confirm the exact crates.io Trusted Publisher repository, workflow, and optional GitHub Environment match the workflow configuration.
- Remove the long-lived registry token once the first OIDC publication succeeds.
- Use the full non-live test suite and `--locked` build/publish preflight in the publishing workflow.

### [ ] `HSA-P1-04` - The 1.0 compatibility boundary is still undecided

Evidence:

- The roadmap's explicit launch item `R2` remains incomplete: a stable `dsc --help` surface and a deprecation policy have not been written. `R6` also leaves `dsc open` and `dsc import` undecided before the surface is locked.
- `src/lib.rs` publicly exports `api`, `cli`, `commands`, `config`, and `utils`. At 1.0 this unintentionally promises semver stability for a large implementation surface, not just the `dsc` binary.
- `Cargo.toml` has no declared `rust-version`, and the project has no supported-Discourse-version policy despite the upstream admin API being unversioned.

House style:

- `distribution.md`, `rust-cli.md`, and `specs.md` require an intentional stable contract rather than freezing accidental implementation details.

Suggested change:

- Decide the public contract before `v1.0.0`:
  1. state which CLI commands, flags, text output, JSON/YAML structures, and exit behaviours are stable;
  2. document deprecation and removal policy;
  3. either make Rust implementation modules private or explicitly commit to maintaining an intentionally small supported Rust API;
  4. state MSRV and the tested/supported Discourse release range.
- Keep incomplete field-driven features on the roadmap. They do not need to delay 1.0 unless they are within the stated stable contract.

### [ ] `HSA-P1-05` - Public hardening guidance overstates what ships

Evidence:

- `docs/harden.md` correctly labels `harden` WIP and says only SSH-focused stages 1–2 are shipped; firewall, Docker, swap, journald, unattended upgrades, and fail2ban are stage 3 work.
- `README.md` describes `harden` as provisioning firewall, SSH, Docker, swap, and fail2ban in both the feature table and documentation index.
- `README.md` quick start ends by running a remote update without first directing an operator to protect the config, run `dsc config check`, or preview planned writes.

House style:

- `security.md` requires accurate, safe operational guidance. `rust-cli.md` requires destructive workflows to lead with their safe alternative.

Suggested change:

- Correct the README before launch: describe the actual SSH-hardening scope and link visibly to the stage-3 gap.
- Make the first-run path: protect `dsc.toml` → `dsc config check` → read-only pull/list → `--dry-run` → mutation. Do not showcase an SSH rebuild as the first action.

### [ ] `HSA-P1-06` - Third-party asset provenance must be settled before broad distribution

Evidence:

- `s/get-discourse-ui-icons` imports 278 tracked SVGs from a pinned `discourse/discourse` commit, combining Font Awesome sprites and Discourse-specific additions.
- There is no `REUSE.toml`, notice/provenance file, or per-asset SPDX coverage; `git grep` finds no SPDX identifier in Rust source either.
- `LICENSE`, `Cargo.toml`, `README.md`, and `CONTRIBUTING.md` consistently publish the repository as MIT, but that alone does not identify the licence obligations of imported assets.

House style:

- `licensing.md` requires source headers or a REUSE manifest plus explicit third-party provenance and notices.

Suggested change:

- Determine the applicable licences and required attribution for each imported source class from the upstream commit. Record provenance, commit, licence, and regeneration instructions in a tracked notice/REUSE configuration.
- Confirm that MIT remains the intended licence for original `dsc` code and documentation. This is a legal/provenance review item, not an assumption that the vendored assets are incompatible.

### [ ] `HSA-P2-01` - Live end-to-end tests mutate shared forum state without reliable cleanup or isolation

Evidence:

- `spec/implementation.md` requires each command's e2e test to clean up created test data.
- Several tests create categories, replies, or changelog markers without deletion; topic tests overwrite configured topic content, and settings tests reset to an assumed value rather than restoring captured state.
- The ignored `test-dsc.toml` is automatically discovered by `tests/common/mod.rs`, so a normal local `cargo test` can touch the configured forum. CI deliberately skips these tests.

House style:

- `testing.md` requires isolated, observable tests and safe filesystem/git boundaries. The same principle applies to an owned test forum.

Suggested change:

- Make live tests opt-in with an explicit environment variable, use a disposable category/topic namespace, restore all changed state in cleanup guards, and serialise tests that share a forum.
- Keep offline integration tests as the normal local/CI gate, then add a separately authorised compatibility job against a dedicated demo instance.

### [ ] `HSA-P2-02` - CI and docs deployment need the final supply-chain and least-privilege pass

Evidence:

- `.github/workflows/ci.yml` has useful formatting, strict Clippy, tests, and a separate `cargo audit` job, but lacks explicit workflow read-only permissions, `persist-credentials: false`, REUSE validation, Zizmor, and `workflow_dispatch`. Its cargo-audit installer has no `fallback: none`.
- `.github/workflows/deploy-docs-to-ghpages.yml` gives Pages write and OIDC permissions to the whole workflow, has no pull-request documentation build, and gives its build checkout persisted credentials.
- `.github/dependabot.yml` covers Cargo and Actions with a cooldown, but not the Python docs dependency and has no weekly routine-update grouping.

House style:

- `ci.md` and `docs.md` require blocking workflow-security checks, least-privilege permissions, and a documentation build gate before merge.

Suggested change:

- Add read-only defaults and checkout hardening; add Zizmor and REUSE after the licence/provenance decision; restrict Pages permissions to the deploy job; and build docs on pull requests.
- Add the `pip` Dependabot ecosystem and group routine minor/patch updates.

### [ ] `HSA-P2-03` - Public docs and support need a small launch pass

Evidence:

- The README feature matrix says category admonition conversion and theme field/assets/remote update remain roadmap work, while command documentation describes them as shipped.
- `README.md` omits the `log` command from its documentation index; `docs/development.md` contains a stale `src/discourse.rs` reference.
- `SECURITY.md` points security updates at a nonexistent `README.md#community` anchor.
- `CONTRIBUTING.md` has a clear best-effort support stance, but the README/docs do not expose a public support route. GitHub Issues are enabled and Discussions are disabled.

House style:

- `docs.md` requires task-oriented, accurate, runnable documentation. `new-repos.md` calls for an explicit contribution/support story in public projects.

Suggested change:

- Complete `R23` as a focussed docs/CLI reality pass rather than adding more feature docs.
- Decide the public route for user questions before launch: a Meta topic is likely appropriate for Discourse usage questions, while GitHub remains the bug/spec tracker.

### [ ] `HSA-P3-01` - Useful post-1.0 CLI and distribution polish

Evidence:

- Bare `dsc` correctly renders the full command summary without loading configuration, but exits with Clap's missing-subcommand status (`2`); the house CLI standard prefers the same helpful summary with a successful exit. This is exit-semantics polish, not a discovery problem.
- There are many `PathBuf` CLI arguments but no shared `~` expansion parser or Clap path hints.
- `dsc version` loads configuration before dispatch even when reporting its own version.
- The HTTP client has no configured timeout.
- `Cargo.toml` lacks `rust-version` and cargo-binstall metadata; release assets expose `sha256.sum` rather than the house-standard canonical `SHA256SUMS` name.

House style:

- `rust-cli.md` and `distribution.md` recommend these patterns, but they do not block a truthful 1.0 release once the P1 contract documents the current supported surface.

Suggested change:

- Put these into a post-launch ergonomics batch unless one is selected as part of the public 1.0 contract.

## Compliant / Good Patterns

- The product is genuinely field-driven: `spec/from-the-field.md`, command specs, and the stable `RXX` roadmap form a strong decision trail.
- `spec/cli-design.md` defines excellent intent: stdout/stderr separation, declarative pull/edit/push/diff, semantic sync comparison, destructive-action guards, and explicit empty-list conventions.
- The existing public `v0.10.30` release demonstrates the distribution model: five platform targets, shell/PowerShell installers, MSI, Homebrew formula, source archive, and checksums. crates.io publishing has also worked.
- `Cargo.lock` is committed, cargo-dist targets are explicit, action references are SHA-pinned, Dependabot and GitHub secret scanning/push protection are enabled, and CI has strict Clippy plus a separate cargo-audit job.
- `SECURITY.md` offers private GitHub Advisory and email reporting. `CONTRIBUTING.md` clearly distinguishes bugs, small features, and field-driven specs.
- Generated shell completions/manpages derive from Clap, and notification PR #38 now has an independently reviewed, safety-preserving correction.

## Not Applicable

- Clinical-safety file sets: `dsc` is operational tooling, not software that performs clinical decision-making. It should maintain its high standard for security and privacy, but DCB0129 artefacts are not proportionate here.
- Tauri/UI standards: no graphical application is shipped.
- Presentation standards: no presentation artefact currently exists. A short terminal recording for launch is sufficient.
- Library extraction: there is no identified multi-surface consumer that justifies extracting a separate engine crate now.

## Suggested First PR

1. Decide and record the 1.0 contract: CLI/deprecation policy, `open`/`import` disposition, public Rust API boundary, MSRV, and supported Discourse range.
2. Fix the global dry-run breach across every mutator, beginning with `update`, then add no-write regression coverage.
3. Correct README first-run and `harden` wording in the same safety PR.

## Suggested Second PR

1. Confirm third-party asset licences and add the required provenance/REUSE coverage.
2. Land crates.io Trusted Publishing, use full release validation, harden `s/version++`, and apply least-privilege release/CI/docs workflow permissions.
3. Enable branch protection and migrate release authority to the CI auto-tag cascade before the first public 1.0 tag.

## Launch Package After P1s

1. Use a fresh, clean, fully synchronised release worktree - not this checkout.
2. Rehearse `s/test-fmt-clippy`, docs build, `cargo audit`, `cargo publish --dry-run`, and the release workflow on a release candidate/tag.
3. Complete `R3` (30-second pull → edit → push → diff recording), `R5` (pre-circulate), and `R23` (docs reality pass).
4. Draft the Meta post around a concrete administrator workflow: who `dsc` is for, admin API/SSH prerequisites, dry-run-first safety, supported scope, installation, support route, and invitation for field-driven specs. Mention Koloki Ltd's Discourse Trusted Partner status factually, without suggesting Discourse endorses `dsc` itself.

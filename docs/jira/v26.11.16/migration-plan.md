# rocket-craft migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/rocket-craft
- Remote: https://github.com/seanchatmangpt/rocket-craft.git
- Branch: claude/pre-ue4-admission-unblock@9605a9d7 (tracks same-named origin branch, 0/0)
- Last commit: 2026-09-24 ("security(v26.9.24): rotate the branch-only ontology/.ggen/keys pair")
- Commits: 616; size 4548 MB
- Languages/build: Rust (Cargo.toml; rocket-sdk, blueprint-rs, unify-rs, nexus-engine),
  TypeScript PWA (pwa-staff/), Justfile, Makefile, UE4.24 projects, Python helper scripts
- Nested repos: none per census (chicago-tdd-tools/ dir here is a vendored copy, not a repo)

## Observed state
- Dirty: 1 -- M docs/security/signing-key-rotation-v26.9.24.md
- Untracked: 10 -- docs/jira/ (contents)
- Local master is ahead 44 / behind 26 of origin/master (diverged; the working branch is
  claude/pre-ue4-admission-unblock which is in sync)
- Local branches: 31; census says 28 unpushed. Seen without upstream:
  security/revoke-branch-exposed-keys-v26.9.24, security/rotate-signing-keys-v26.9.24,
  three subagent-Agents-Engineer-self-* at the same sha
- Stash: 0
- Large dirs: versions/ 1.7 GB (UE engine/project versions: 4.24-Shooter, 4.24-Survival,
  4.27.0, Brm427, Realistic, Template, v4_27_0), pwa-staff/ 436 MB (node_modules present,
  2810 entries); Build/ and build/ are empty; no target/
- Local data: barbarian-road-mashines-key.keystore (Android signing keystore, 2810 bytes)
  next to a .placeholder twin -- SECRET; ontology/.ggen/keys referenced by recent commits.

## What it is
Rocket Craft: a multi-game Unreal Engine 4.24 monorepo (six UE4 projects) orchestrated by a
Rust SDK/CLI (`./rocket`), with a Supabase backend and a TypeScript PWA, plus several
independent Rust workspaces.

## Classification
ACTIVE (census). Note: last commit 7 weeks ago; the two most recent commits are key rotations.

## Migration risk
- Android keystore file in the tree -- if it is the real key and not in git, losing it means
  losing the ability to update the Play Store app. Verify `git ls-files | grep keystore`.
- master diverged (44 ahead) and ~28 local branches unpushed.
- versions/ (1.7 GB) may contain engine builds not reproducible from git (check
  `git check-ignore versions`).
- Rate: HIGH (keystore + 1.7 GB of possibly-untracked engine assets)

## Recommended action
COMMIT-PUSH

1. `git add docs/jira docs/security/signing-key-rotation-v26.9.24.md && git commit -m
   "docs: v26.9.24 key-rotation notes + jira (pre-migration checkpoint)" && git push`.
2. `git push origin master` (DECISION: master is 44 ahead / 26 behind -- Sean must decide
   whether to merge or force-with-lease; do not push blindly) and
   `git push -u origin security/revoke-branch-exposed-keys-v26.9.24
   security/rotate-signing-keys-v26.9.24`; prune subagent-* branches.
3. Keystore: `git ls-files barbarian-road-mashines-key.keystore` -- if NOT tracked (correct),
   copy it to the password manager / encrypted volume by hand. DECISION: if it IS tracked,
   that is a committed secret; rotate it.
4. `git check-ignore -v versions pwa-staff/node_modules` -- untracked-and-ignored parts of
   versions/ must be rsynced separately: `rsync -a --progress versions/ /Volumes/ARCHIVE/
   rocket-craft-versions/` (1.7 GB). DECISION: or re-download UE 4.24/4.27 sources.
5. Cleanup before any copy: `rm -rf pwa-staff/node_modules Build build __pycache__`.
6. Bundle: `git bundle create /Volumes/ARCHIVE/rocket-craft.bundle --all`.
7. Clone fresh on the new machine; restore keystore and versions/ out-of-band.

## First commands on the new machine
- `git clone https://github.com/seanchatmangpt/rocket-craft.git && cd rocket-craft`
- `cargo check --workspace` ; `cd pwa-staff && npm ci` ; `./rocket --help` (stated, not run)

## See Also
- ../v26.9.19/ (previous jira dir)
- /Users/sac/docs/jira/v26.11.16/README.md (master index)

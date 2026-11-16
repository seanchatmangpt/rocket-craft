# rocket-craft: land or delete remote branch `chore/remove-ds-store`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `chore/remove-ds-store` — not merged into `master`, no open PR
- Evidence: `git branch -r --no-merged origin/master` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/rocket-craft --head chore/remove-ds-store`) or delete (`git push origin --delete chore/remove-ds-store`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/master` no longer lists `chore/remove-ds-store`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | chore/remove-ds-store | decision pending

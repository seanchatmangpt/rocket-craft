# rocket-craft: land or delete remote branch `claude/eight-hour-update`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `claude/eight-hour-update` — not merged into `master`, no open PR
- Evidence: `git branch -r --no-merged origin/master` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/rocket-craft --head claude/eight-hour-update`) or delete (`git push origin --delete claude/eight-hour-update`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/master` no longer lists `claude/eight-hour-update`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | claude/eight-hour-update | decision pending

# rocket-craft: land or delete remote branch `claude/laughing-fermat-at82v3`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `claude/laughing-fermat-at82v3` — not merged into `master`, no open PR
- Evidence: `git branch -r --no-merged origin/master` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/rocket-craft --head claude/laughing-fermat-at82v3`) or delete (`git push origin --delete claude/laughing-fermat-at82v3`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/master` no longer lists `claude/laughing-fermat-at82v3`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | claude/laughing-fermat-at82v3 | decision pending

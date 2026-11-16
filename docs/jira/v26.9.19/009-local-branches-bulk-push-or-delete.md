# rocket-craft: triage 27 local-only branches

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: local branches with commits not on `origin/master` and no upstream
- Evidence: `git rev-list --count origin/master..<branch>` > 0 for each: claude/pre-ue4-admission-unblock:68 subagent-Agents-Engineer-self-0be1141e:55 subagent-Agents-Engineer-self-59a70811:55 subagent-Agents-Engineer-self-6a4f147b:55 subagent-Agents-Engineer-self-dd38d492:55 subagent-Ledger-DfLSS-Engineer-self-8dd7cb0f:56 subagent-Ontology-DfLSS-Engineer-self-626a67b5:56 subagent-Ontology-Tera-Engineer-self-136b2e7b:55 subagent-Ontology-Tera-Engineer-self-7d0269f2:55 subagent-Ontology-Tera-Engineer-self-b1414e11:55 subagent-Ontology-Tera-Engineer-self-e45ddd05:55 subagent-Playwright-DfLSS-Engineer-self-3c042c8c:56 subagent-Playwright-DfLSS-Engineer-self-575204f7:57 subagent-Process-DfLSS-Engineer-self-c04f6a03:56 subagent-Root-Engineer-self-35faa360:55 subagent-Root-Engineer-self-7148b8b9:56 subagent-Root-Engineer-self-77fe3122:55 subagent-Root-Engineer-self-f4d9b448:55 subagent-Rust-DfLSS-Engineer-self-8a7e428b:56 subagent-Rust-Ggen-Engineer-self-33dcf8e2:55 subagent-Rust-Ggen-Engineer-self-3de31b46:55 subagent-Rust-Ggen-Engineer-self-d9b3fcec:55 subagent-Rust-Ggen-Engineer-self-ed8e0697:55 subagent-Scripts-Engineer-self-22d4a18b:55 subagent-Scripts-Engineer-self-29f0318e:55 subagent-Scripts-Engineer-self-7694c2d3:55 subagent-Scripts-Engineer-self-eb9bb78b:55

## Work to complete
- Triage each branch: push (`git push -u origin <branch>`) or delete after confirming the commits are obsolete/recoverable. Batch the work; record decisions in History.

## Acceptance
- Every listed branch is pushed or deleted; no local-only branch with unique commits remains.

## History
- 2026-09-19 | OPEN | survey found 27 local-only branches | full list above | triage pending

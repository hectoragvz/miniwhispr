# Workflow

Personal project: all work lands directly on `main`. No branches, no PRs.

## Per ticket

Tickets live in `.scratch/<feature>/issues/NN-<slug>.md`. Pick the lowest-numbered ticket whose `Blocked by` tickets are all `Status: resolved`.

1. `/clear`, then record the starting commit: `git rev-parse HEAD`.
2. `/implement .scratch/<feature>/issues/NN-<slug>.md` (builds test-first with `/tdd`).
3. When it reaches review, commit locally (do not push), then run `/code-review <starting commit>`.
4. Fix the review findings.
5. Run the local checks (below); all must pass.
6. Set `Status: resolved` in the ticket file, commit (amend or new commit), and push to `main`.
7. Confirm CI is green on GitHub. If it is red, fix it immediately.
8. If the ticket went sideways, run `/retro` before clearing.

## Local checks before every push

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `pnpm tsc`
- `pnpm lint`
- `pnpm test`

## Commits

- Summary line describing the change, then a pointer to the ticket file in the body.
- Never add an agent `Co-Authored-By` line.

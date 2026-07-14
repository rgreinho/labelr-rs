1. Feature #0001: migrate-hubcaps-to-octocrab — requirements

2. WHAT (Functional Requirements)
3. 1. Replace the `hubcaps` GitHub client with `octocrab` v0.54.0 across the codebase.
4. 2. Preserve existing CLI behavior and flags (--file, --token, --org, --sync, --update-existing, --verbose, repository/owner inference).
5. 3. Support listing organization repositories with pagination and process all repos.
6. 4. Implement label create, update, delete flows identical in effect to current behavior.
7. 5. Provide bounded concurrency (default 8) while processing repositories.
8. 6. Implement robust retry & rate-limit handling (retry on 429 and 5xx) with exponential backoff and jitter.
9. 7. Ensure secret tokens are not logged or leaked.
10. 8. Add unit tests where practical and integration/mock tests for API interactions.

11. Non-functional Requirements
12. - Build and unit tests must pass in CI.
13. - CLI runtime for N repos should be reasonably performant; default concurrency = 8.
14. - Retry/backoff should avoid thundering herd and respect transient errors.
15. - Changes should remove all references to `hubcaps` in Cargo.toml and source.

16. Acceptance Criteria
17. - `cargo build` and `cargo test` succeed.
18. - No `hubcaps` entries remain in Cargo.toml or source imports.
19. - Manual verification: run against a test repo/org — labels created/updated/deleted as expected in both sync and non-sync modes.
20. - PR with migration includes testing steps and a rollback plan.

21. Glossary
22. - sync mode: delete existing labels then create configured labels.
23. - update_existing: update labels if present instead of skipping.

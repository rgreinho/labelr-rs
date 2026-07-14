1. Feature #0001: migrate-hubcaps-to-octocrab — tasks

2. Overview: step-by-step implementation checklist. Mark tasks done as implemented.

3. Pre-work
4. - [x] Create a feature branch: migrate/hubcaps-to-octocrab
5. - [x] Commit current state (safe rollback point)

6. Dependency updates
7. - [x] Add `octocrab = "0.54.0"` to Cargo.toml
8. - [x] Add `serde_json = "1.0"` and `rand = "0.8"`
9. - [x] Remove `hubcaps` dependency

10. Code changes (file-level)
11. - [x] src/main.rs
12.   - [x] Initialize Octocrab client with token
13.   - [x] Replace hubcaps repo listing with octocrab orgs().list_repos() + all_pages
14.   - [x] Build repo list as Vec<(owner, repo)>
15.   - [x] Implement bounded concurrency (futures::stream::buffer_unordered, concurrency=8)
16.   - [x] Integrate retry_octocrab helper for GET/POST/PATCH calls
17.   - [x] Add jitter to backoff
18.   - [x] Remove unused imports and clean warnings

19. - [x] src/label.rs
20.   - [x] Remove hubcaps types usage
21.   - [x] Add Label::to_label_body -> serde_json payload
22.   - [x] Replace delete_labels signature to accept &Octocrab, owner, repo, Vec<octocrab::models::Label>

23. - [x] src/git.rs
24.   - [x] Fix remote URL parsing errors and extract owner/repo from git-url-parse path

25. - [ ] tests/
26.   - [ ] Add unit tests for Label conversion (to_label_body)
27.   - [ ] Add mock server tests for retry/backoff on 429 and 5xx responses (httpmock or similar)
28.   - [ ] Add tests for pagination behavior using mocked paginated responses

29. Validation
30. - [x] Build project locally (used temporary CARGO_HOME for fetch)
31. - [x] Run unit tests (existing tests updated) — 5 passed
32. - [ ] Run integration/manual smoke test against a test org/repo (verify sync and non-sync flows)

33. CI & docs
34. - [ ] Update CI workflow to ensure cache works with new deps and set secure env var for GITHUB_TOKEN
35. - [ ] Update README to reflect octocrab usage and any CLI changes

36. PR
37. - [ ] Open PR: describe rationale, testing steps, manual verification, rollback plan
38. - [ ] Include benchmarks or notes about concurrency and rate-limit behavior

39. Estimated effort & notes
40. - Remaining work: tests and CI changes — ~1-3 hours depending on test depth.
41. - Rollback plan: revert feature branch to previous commit and re-run CI.

42. Contact
43. - For questions about API differences or choosing between typed endpoints vs HTTP endpoints, consult octocrab docs (https://docs.rs/octocrab/0.54.0).

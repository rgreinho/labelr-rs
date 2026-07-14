1. Feature #0001: migrate-hubcaps-to-octocrab — design

2. Overview
3. Replace hubcaps usage with octocrab. Use octocrab's semantic API where convenient (models) and fallback to octocrab HTTP methods (get/post/patch/delete) for endpoints not covered. Keep Tokio runtime and eyre/color-eyre for error handling.

4. Components
5. - CLI layer (src/main.rs): parse args, build Octocrab client with OctocrabBuilder::personal_token(token), collect repo list (single or org), drive concurrent processing.
6. - Label module (src/label.rs): keep Label and Labels data model for YAML, provide conversion to octocrab request body (JSON) and delete_labels that accepts octocrab, owner, repo.
7. - Git discovery (src/git.rs): infer owner/repo using git2 + git-url-parse path parsing.

8. Key Implementation Details
9. - Octocrab init: OctocrabBuilder::default().personal_token(token).build() -> use Arc to share client between tasks.
10. - Org repo listing: octo.orgs(&owner).list_repos().per_page(100).send() then octo.all_pages::<models::Repository>(page) to collect all pages.
11. - Label ops: use octo.get/post/patch/_delete or typed API when available. Convert local Label -> JSON body: { name, color (no #), description }.
12. - Pagination: use octocrab.all_pages for endpoints that return Page<T>.
13. - Concurrency: futures::stream::iter(repos).buffer_unordered(concurrency) where concurrency default = 8 and configurable.
14. - Retry/backoff: central retry_octocrab helper that retries on octocrab::Error::GitHub with 429 or >=500 status codes and transport errors (Hyper/Http/Service). Exponential backoff (2^n seconds) with jitter (random 0..base_secs*1000 ms). Respect Retry-After header as future improvement.

15. Error handling
16. - Surface Octocrab errors as eyre::Report for the CLI. For per-repo tasks, aggregate errors and fail with the first unrecoverable error.
17. - Retries limited to configurable attempts (default 5). For delete_labels use a smaller retry count (3).

18. Testing
19. - Unit tests: Label YAML parsing, to_label_body conversion.
20. - Integration / Mock tests: use httpmock or similar to simulate 429 and verify retry/jitter behavior, and to simulate paginated org repo responses.

21. Security
22. - Token only passed to OctocrabBuilder::personal_token and never logged. Avoid dbg! of token.
23. - Document required environment variables for CI and testing securely.

24. Files touched
25. - Cargo.toml (remove hubcaps, add octocrab, serde_json, rand)
26. - src/main.rs (client init, pagination, concurrency, retries)
27. - src/label.rs (label conversion, delete_labels)
28. - src/git.rs (minor parsing fixes)
29. - tests/* (new integration/mock tests)

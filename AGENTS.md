# AGENTS.md

## Scope

This repository contains a Rust workspace for a Yew frontend application. The active Rust workspace root is `flashcards/`.

## Working Rules

- Keep generated output out of version control. Do not commit `target/` or `flashcards/frontend/dist/`.
- Prefer small, responsibility-focused Rust modules over expanding `main.rs`.
- Keep app behavior stable unless the task explicitly calls for a product change.
- Preserve the current storage format unless a migration is part of the task.

## Project Layout

- `flashcards/Cargo.toml`: workspace manifest.
- `flashcards/frontend`: Yew frontend crate.
- `flashcards/frontend/index.html`: document shell that loads the shared fonts and stylesheet.
- `flashcards/frontend/styles.css`: shared application stylesheet for layout, components, tokens, and animations.
- `flashcards/frontend/src/app.rs`: main application component and orchestration.
- `flashcards/frontend/src/model.rs`: serializable domain types.
- `flashcards/frontend/src/storage.rs`: browser local-storage helpers.
- `flashcards/frontend/src/csv_io.rs`: CSV parsing and export helpers.

## Validation Commands

Use these commands from the repository root:

```powershell
cargo fmt --manifest-path .\flashcards\Cargo.toml
cargo check --manifest-path .\flashcards\Cargo.toml
```

Run the frontend locally:

```powershell
cd .\flashcards\frontend
trunk serve
```

Build the production frontend bundle:

```powershell
cd .\flashcards\frontend
trunk build --release
```

## Deployment Guidance

- GitHub Pages assets must be built with `flashcards/frontend/Trunk.toml` using `public_url = "/myRustDemoProject/"`.
- Preferred path: use `.github/workflows/deploy-pages.yml` to publish `flashcards/frontend/dist` to the `gh-pages` branch.
- The deploy workflow must run `scripts/strip-trunk-autoreload.ps1` after `trunk build --release` to remove Trunk's injected live-reload websocket script before publishing.
- Keep `.nojekyll` in the published `gh-pages` output.
- If Pages must be updated manually, build from `flashcards/frontend`, run `scripts/strip-trunk-autoreload.ps1`, then copy only the contents of `flashcards/frontend/dist/` into the root of the `gh-pages` branch and push that branch.
- Do not commit `flashcards/frontend/dist/` on development branches; it is a publish artifact only.

## Refactor Guidance

- If UI code grows, split presentational sections into additional modules under `flashcards/frontend/src/`.
- Keep styling centralized in `flashcards/frontend/styles.css`; prefer reusable class-based styling over inline styles or per-component CSS files.
- Extend the existing CSS custom properties in `:root` for colors, surfaces, and shadows before introducing new one-off values.
- Follow the existing dash-separated class naming and shared utility patterns such as `panel`, `btn`, and `*-actions` when adding UI.
- Keep pure data and transformation logic outside Yew callbacks when practical so it remains easy to test.
- Prefer fixing structure and ownership boundaries before adding abstractions.
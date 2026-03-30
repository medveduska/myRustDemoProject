# Language Flashcards

Language Flashcards is a Rust and Yew web application for studying vocabulary from CSV datasets.

## Features

- Import UTF-8 CSV flashcards with word, optional pinyin, translation, and known state.
- Switch between normal and reverse study directions.
- Shuffle unknown cards and progress through three reveal stages.
- Manage multiple datasets stored in browser local storage.
- Export the current dataset back to CSV with progress preserved.

## Project Structure

The Rust workspace lives under `flashcards/`.

```text
flashcards/
|- Cargo.toml
|- Cargo.lock
|- frontend/
|  |- Cargo.toml
|  |- Trunk.toml
|  |- index.html
|  '- src/
|     |- app.rs
|     |- csv_io.rs
|     |- main.rs
|     |- model.rs
|     '- storage.rs
```

## Development

Format the workspace:

```powershell
cargo fmt --manifest-path .\flashcards\Cargo.toml
```

Check the workspace:

```powershell
cargo check --manifest-path .\flashcards\Cargo.toml
```

Run the frontend locally with Trunk:

```powershell
cd .\flashcards\frontend
trunk serve
```

For GitHub Pages deployments from this repository, Trunk is configured with `public_url = "/myRustDemoProject/"` so generated asset URLs resolve under the repository site path.

## Deployment

GitHub Pages publishing is automated by [.github/workflows/deploy-pages.yml](.github/workflows/deploy-pages.yml). Pushing changes to `master` rebuilds `flashcards/frontend` with Trunk and publishes `flashcards/frontend/dist` to the `gh-pages` branch.

You can also run the workflow manually with `workflow_dispatch` if you need to republish without a new commit.

Because Trunk 0.21.14 still injects a live-reload websocket client into the generated HTML for this app, the deploy workflow runs [scripts/strip-trunk-autoreload.ps1](scripts/strip-trunk-autoreload.ps1) after `trunk build --release` to remove that dev-only snippet before publishing.

## Notes

- Generated output such as `target/` and `frontend/dist/` is intentionally ignored.
- The frontend crate is organized by responsibility: UI in `app.rs`, domain types in `model.rs`, persistence in `storage.rs`, and CSV handling in `csv_io.rs`.


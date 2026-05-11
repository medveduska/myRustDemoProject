# Language Flashcards

Language Flashcards is a Rust and Yew web application for studying vocabulary from CSV datasets.

## Features

- **Multi-wordset management**: Create and organize unlimited named wordsets (e.g., "HSK 1", "Week 3 vocabulary") all stored locally in your browser.
- **CSV import/export**: Import UTF-8 CSV files with word, optional pinyin, translation, and known state into an empty wordset. Export any wordset back to CSV with progress preserved.
- **Full progress backup**: Download your complete app state (all wordsets and study position) as a JSON backup file, or restore from a previous backup to switch devices or recover from data loss.
- **Flexible study modes**: Switch between normal (character → pinyin → translation) and reverse (translation → pinyin → character) directions. Shuffle card order and randomize study sessions.
- **Three-stage reveal system**: Click a flashcard to progress through three stages (e.g., character, pinyin, meaning) at your own pace.
- **Word Review tracking**: Mark cards as known to move them to the Word Review table. Restore or delete mastered words, and view all known and unknown words in one place.
- **Automatic persistence**: All progress is saved automatically to browser local storage after every action. Works entirely offline after the page loads.

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

For GitHub Pages deployments from this repository, Trunk is configured with `public_url = "/"` because the site is served from the custom domain `wordcards.chinesewithbaiba.eu`.

## Deployment

GitHub Pages publishing is automated by [.github/workflows/deploy-pages.yml](.github/workflows/deploy-pages.yml). Pushing changes to `master` rebuilds `flashcards/frontend` with Trunk and publishes `flashcards/frontend/dist` to the `gh-pages` branch.

You can also run the workflow manually with `workflow_dispatch` if you need to republish without a new commit.

Because Trunk 0.21.14 still injects a live-reload websocket client into the generated HTML for this app, the deploy workflow runs [scripts/strip-trunk-autoreload.ps1](scripts/strip-trunk-autoreload.ps1) after `trunk build --release` to remove that dev-only snippet before publishing.

## Notes

- Generated output such as `target/` and `frontend/dist/` is intentionally ignored.
- The frontend crate is organized by responsibility: UI in `app.rs`, domain types in `model.rs`, persistence in `storage.rs`, and CSV handling in `csv_io.rs`.
- Import is available only when a wordset contains no cards, preventing silent overwrites; Export is always available for the active wordset.


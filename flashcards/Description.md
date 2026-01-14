This is an app named Language Flashcards 🈶

A modern web-based flashcard app for learning languages, built with Rust and Yew.

## Features

### Multi-Column Learning
Import flashcards from CSV files with up to 3 columns in UTF-8 format. Perfect for languages like Chinese with character, pinyin, and translation:
```csv
阿姨,āyí,aunt
啊,a,ah
矮,ǎi,short
```

### Flexible Study Modes
- **Study Directions**: Switch between normal (Character → Pinyin → Translation) and reverse (Translation → Pinyin → Character) modes
- **Randomize**: Shuffle cards to avoid memorizing order
- **Progressive Learning**: Cards progress through 3 study stages

### Dataset Management
- **Multiple Datasets**: Create and manage separate datasets (e.g., HSK 1, Business Terms)
- **Independent Progress**: Each dataset tracks its own known/unknown cards
- **Auto-Save**: Changes are automatically saved to browser storage

### Card Operations
- **Mark as Known**: Move cards from the unknown pile to track progress
- **Track Progress**: See progress counter showing known vs. total cards
- **Restore Cards**: Move previously known cards back to study if needed
- **Delete Cards**: Remove unwanted flashcards

### File Management
- **Import CSV**: Upload flashcard data from CSV files
- **Download Progress**: Export current dataset with progress tracking
- **Local Storage**: All data persists in browser local storage

## Technology Stack

- **Frontend**: Rust with Yew framework
- **Build Tools**: Cargo and Trunk
- **Storage**: Browser LocalStorage for data persistence
- **Data Format**: CSV for import/export

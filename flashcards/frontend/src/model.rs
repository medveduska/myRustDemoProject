use serde::{Deserialize, Serialize};

pub const APP_BACKUP_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Flashcard {
    pub word: String,
    pub pinyin: Option<String>,
    pub translation: String,
    #[serde(default)]
    pub known: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Dataset {
    pub name: String,
    pub flashcards: Vec<Flashcard>,
    pub known_cards: Vec<Flashcard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlashcardStage {
    First,
    Second,
    Third,
}

impl Default for FlashcardStage {
    fn default() -> Self {
        Self::First
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StudyDirection {
    Normal,
    Reverse,
}

impl Default for StudyDirection {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyMode {
    Reveal,
    Exercise,
}

impl Default for StudyMode {
    fn default() -> Self {
        Self::Reveal
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PersistedState {
    pub flashcards: Vec<Flashcard>,
    pub known_cards: Vec<Flashcard>,
    pub current_index: usize,
    pub stage: FlashcardStage,
    pub direction: StudyDirection,
    #[serde(default)]
    pub current_dataset: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppBackupSnapshot {
    pub version: u32,
    pub exported_at: String,
    pub persisted_state: PersistedState,
    pub datasets: Vec<Dataset>,
}

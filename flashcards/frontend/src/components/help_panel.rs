use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HelpPanelProps {
    pub on_close: Callback<MouseEvent>,
}

#[function_component(HelpPanel)]
pub fn help_panel(props: &HelpPanelProps) -> Html {
    html! {
        <div class="help-backdrop">
            <section class="help-modal panel">
                <div class="help-modal-header">
                    <h2 class="panel-title help-modal-title">{"Help & Information"}</h2>
                    <button class="btn btn-secondary btn-small help-close-btn" onclick={props.on_close.clone()}>
                        {"✕ Close"}
                    </button>
                </div>

                <div class="help-body">
                    <div class="help-section">
                        <h3 class="help-section-title">{"About"}</h3>
                        <p class="help-text">
                            {"Language Flashcards is a browser-based vocabulary study tool. \
                            All your data is stored locally in your browser — nothing is sent to a server. \
                            You can organise cards into named datasets, study them in normal or reverse order, \
                            and track which words you have already mastered."}
                        </p>
                        <ul class="help-list">
                            <li>{"Works entirely offline after the page loads."}</li>
                            <li>{"Progress is saved automatically between sessions."}</li>
                            <li>{"Supports Chinese characters, pinyin, and a translation field."}</li>
                            <li>{"Export your cards at any time as a CSV file from the Wordsets panel."}</li>
                        </ul>
                    </div>

                    <hr class="help-divider" />

                    <div class="help-section">
                        <h3 class="help-section-title">{"How to use"}</h3>

                        <div class="help-step">
                            <span class="help-step-number">{"1"}</span>
                            <div>
                                <strong>{"Create or select a wordset"}</strong>
                                <p class="help-text">
                                    {"Use the Wordsets panel to create a named collection of flashcards \
                                    (e.g., \"HSK 1\" or \"Week 3 vocabulary\"). \
                                    Click an existing wordset button to switch to it."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"2"}</span>
                            <div>
                                <strong>{"Import a CSV file"}</strong>
                                <p class="help-text">
                                    {"After creating a new empty wordset, an Import option appears inside the Wordsets panel. Click "}
                                    <em>{"Choose File"}</em>
                                    {" and select a CSV file. \
                                    Each row should have up to four columns:"}
                                </p>
                                <div class="csv-format-block">
                                    <code>{"word, pinyin, translation, known"}</code>
                                </div>
                                <ul class="help-list">
                                    <li><strong>{"word"}</strong>{" — the term to study (e.g., a Chinese character)."}</li>
                                    <li><strong>{"pinyin"}</strong>{" — pronunciation hint, optional."}</li>
                                    <li><strong>{"translation"}</strong>{" — the meaning in your language."}</li>
                                    <li><strong>{"known"}</strong>{" — write "}<code>{"true"}</code>{" if already mastered, otherwise leave blank or write "}<code>{"false"}</code>{"."}</li>
                                </ul>
                                <p class="help-text">{"Example row:"}</p>
                                <div class="csv-format-block">
                                    <code>{"你好,nǐ hǎo,Hello,false"}</code>
                                </div>
                                <p class="help-text help-text-muted">
                                    {"The file does not need a header row. \
                                    Import is only available while the wordset is empty, \
                                    so it cannot overwrite cards you have already added."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"3"}</span>
                            <div>
                                <strong>{"Add cards manually"}</strong>
                                <p class="help-text">
                                    {"Click "}
                                    <em>{"Add New Flashcard"}</em>
                                    {" in the Study Controls panel to add a single card without a CSV file."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"4"}</span>
                            <div>
                                <strong>{"Study"}</strong>
                                <p class="help-text">
                                    {"Click the flashcard to reveal the next stage: \
                                    character → pinyin → translation (or reversed). \
                                    Mark a card as "}
                                    <em>{"Known"}</em>
                                    {" to move it to the Word Review table, \
                                    or use "}
                                    <em>{"Randomize"}</em>
                                    {" to shuffle the order. You can also switch to "}
                                    <em>{"Exercise Mode"}</em>
                                    {" from Study Controls to type an answer instead of flipping stages. \
                                    After pressing Enter or "}
                                    <em>{"Check"}</em>
                                    {", the card highlights green or red, shows the correct answer if needed, and reveals pinyin when available."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"5"}</span>
                            <div>
                                <strong>{"Word Review table"}</strong>
                                <p class="help-text">
                                    {"The Word Review section lists all words you have marked as known. \
                                    Use "}
                                    <em>{"Restore"}</em>
                                    {" to move a card back to the Flashcards section for further practice, \
                                    or "}
                                    <em>{"Delete"}</em>
                                    {" to remove it permanently."}
                                </p>
                                <p class="help-text">
                                    {"Click "}
                                    <em>{"Show Unknown Words"}</em>
                                    {" to also show cards still in the Flashcards section. \
                                    Unknown cards appear with an \"Unknown\" badge and let you \
                                    mark them as known or delete them directly from the table — \
                                    useful when you want to review all words in one place without \
                                    navigating card by card."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"6"}</span>
                            <div>
                                <strong>{"Rename a wordset"}</strong>
                                <p class="help-text">
                                    {"Click "}
                                    <em>{"Rename"}</em>
                                    {" below any wordset button to edit its name inline. \
                                    Confirm with "}
                                    <em>{"Save"}</em>
                                    {" or discard with "}
                                    <em>{"Cancel"}</em>
                                    {". Renaming is blocked if the new name is empty or already taken."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"7"}</span>
                            <div>
                                <strong>{"Export the active wordset"}</strong>
                                <p class="help-text">
                                    {"Click "}
                                    <em>{"Export Flashcards"}</em>
                                    {" in the Wordsets panel to save the currently selected wordset's cards \
                                    (including known/unknown status) as a CSV file for backup or sharing. \
                                    The export button is available whenever a wordset is selected."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"8"}</span>
                            <div>
                                <strong>{"Save and restore your entire progress"}</strong>
                                <p class="help-text">
                                    {"The "}
                                    <em>{"Full Progress Backup"}</em>
                                    {" section in the Wordsets panel lets you download and restore your complete app state:"}
                                </p>
                                <ul class="help-list">
                                    <li><strong>{"Save All"}</strong>{" — downloads every wordset and your current study position (which card you're on, normal vs. reverse mode, etc.) as a single JSON backup file."}</li>
                                    <li><strong>{"Load"}</strong>{" — restores your entire app state from a previously saved backup file. This replaces all current wordsets and study progress."}</li>
                                </ul>
                                <p class="help-text help-text-muted">
                                    {"Use this feature to switch devices, back up your learning history, or recover after clearing browser data. \
                                    Unlike Export (which saves only the active wordset as CSV), Save All captures everything in a portable format."}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}

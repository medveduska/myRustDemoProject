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
                            <li>{"Export your cards at any time as a CSV file."}</li>
                        </ul>
                    </div>

                    <hr class="help-divider" />

                    <div class="help-section">
                        <h3 class="help-section-title">{"How to use"}</h3>

                        <div class="help-step">
                            <span class="help-step-number">{"1"}</span>
                            <div>
                                <strong>{"Create or select a dataset"}</strong>
                                <p class="help-text">
                                    {"Use the Datasets panel to create a named collection of flashcards \
                                    (e.g., \"HSK 1\" or \"Week 3 vocabulary\"). \
                                    Click an existing dataset button to switch to it."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"2"}</span>
                            <div>
                                <strong>{"Import a CSV file"}</strong>
                                <p class="help-text">
                                    {"In the Import &amp; Export panel, click "}
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
                                    Importing replaces the currently loaded cards — \
                                    export first if you want to keep your progress."}
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
                                    {" to move it to the Known Words table, \
                                    or use "}
                                    <em>{"Randomize"}</em>
                                    {" to shuffle the order."}
                                </p>
                            </div>
                        </div>

                        <div class="help-step">
                            <span class="help-step-number">{"5"}</span>
                            <div>
                                <strong>{"Export"}</strong>
                                <p class="help-text">
                                    {"Click "}
                                    <em>{"Download CSV"}</em>
                                    {" in the Import &amp; Export panel to save all cards \
                                    (including known/unknown status) as a CSV file for backup or sharing."}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}

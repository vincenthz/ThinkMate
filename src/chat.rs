use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use iced::{
    widget::{column, container, row, scrollable, text, text_editor, Container},
    Color, Element, Length, Padding,
};
use ulid::Ulid;

use crate::{api, helper::button_icon, Message};

pub struct Chat {
    pub ulid: Ulid,
    pub model: api::LocalModel,
    pub state: ChatState,
}

pub enum ChatState {
    Prompting(iced::widget::text_editor::Content),
    Generate {
        start: SystemTime,
        prompt: String,
        output: ChatOutput,
        ended_at: Option<SystemTime>,
    },
}

impl Chat {
    pub fn new(model: api::LocalModel) -> Self {
        Self {
            ulid: Ulid::new(),
            model,
            state: ChatState::Prompting(iced::widget::text_editor::Content::new()),
        }
    }

    pub fn name(&self) -> String {
        let time = self.ulid.datetime();
        let date: DateTime<Local> = time.clone().into();

        format!("Chat {}", date.format("%Y-%m-%d %H:%M:%S"))
    }

    pub fn set_generating(&mut self) -> &str {
        match &mut self.state {
            ChatState::Prompting(content) => {
                let prompt = content.text();
                self.state = ChatState::Generate {
                    start: SystemTime::now(),
                    prompt,
                    output: ChatOutput::new(),
                    ended_at: None,
                };
                match &self.state {
                    ChatState::Prompting(_) => unreachable!(),
                    ChatState::Generate {
                        prompt,
                        output: _,
                        start: _,
                        ended_at: _,
                    } => prompt.as_str(),
                }
            }
            ChatState::Generate {
                prompt: _,
                output: _,
                ended_at: _,
                start: _,
            } => {
                tracing::error!("chat set generating in already generating mode");
                ""
            }
        }
    }

    pub fn view(&self) -> Container<Message> {
        match &self.state {
            ChatState::Prompting(content) => container(
                column![].push(
                    row![]
                        .push(
                            text_editor(&content)
                                .placeholder("Type something here...")
                                .on_action(Message::ChatEditPrompt),
                        )
                        .push(button_icon(iced_fonts::Bootstrap::Send).on_press_maybe(
                            (!content.text().is_empty()).then_some(Message::ChatSend),
                        ))
                        .spacing(5.0),
                ),
            ),
            ChatState::Generate {
                prompt,
                output,
                start,
                ended_at,
            } => {
                let end_element = if let Some(ended_at) = ended_at {
                    let duration = ended_at.duration_since(*start).unwrap_or(Duration::ZERO);
                    Element::from(
                        text(format!("generated in {} seconds", duration.as_secs()))
                            .size(13.0)
                            .color(Color::from_rgb8(0xa0, 0xa0, 0xa0)),
                    )
                } else {
                    Element::from(iced_aw::Spinner::new())
                };
                container(scrollable(
                    container(
                        column![]
                            .push(
                                container(
                                    container(text(prompt))
                                        .padding(Padding::default().left(5.0).right(5.0)),
                                )
                                .style(container::bordered_box)
                                .center_x(Length::Fill)
                                .padding(
                                    Padding::default()
                                        .top(5.0)
                                        .bottom(5.0)
                                        .left(30.0)
                                        .right(30.0),
                                ),
                            )
                            .push(output.view())
                            .push(
                                container(end_element)
                                    .padding(Padding::default().top(5.0))
                                    .center_x(Length::Fill),
                            )
                            .spacing(15.0),
                    )
                    .padding(Padding::from(5.0)),
                ))
            }
        }
        .padding(Padding::from(5.0))
    }

    pub fn add_content(&mut self, response: api::ChatMessageResponse) {
        match &mut self.state {
            ChatState::Prompting(_) => {
                tracing::error!("chat message appended in prompt mode")
            }
            ChatState::Generate {
                prompt: _,
                output,
                start: _,
                ended_at: _,
            } => output.add_content(response),
        }
    }
}

pub struct ChatOutput {
    output: String,
}

impl ChatOutput {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn view(&self) -> Container<Message> {
        container(text(&self.output))
    }

    pub fn add_content(&mut self, response: api::ChatMessageResponse) {
        self.output.push_str(&response.message.content)
    }
}

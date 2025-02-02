use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

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

pub enum OutputMode {
    Text(Vec<iced::widget::markdown::Item>),
    Code(String, iced::widget::text_editor::Content),
}

pub struct ChatOutput {
    stream: MarkdownIncremental,
    output: Vec<Chunk>,
}

impl ChatOutput {
    pub fn new() -> Self {
        Self {
            stream: MarkdownIncremental::new(),
            output: vec![],
        }
    }

    fn unparsed(&self) -> &str {
        &self.stream.buf[self.stream.pos..]
    }

    pub fn view(&self) -> Container<Message> {
        let rem = std::iter::once(text(self.unparsed()).into());
        container(column(self.output.iter().map(|c| c.view()).chain(rem)).spacing(20.0))
    }

    pub fn add_content(&mut self, response: api::ChatMessageResponse) {
        let msg = &response.message;
        //println!("adding content: {:?} \"{}\"", msg.role, msg.content);

        self.stream.add_content(&msg.content);
        match self.stream.process_content() {
            None => {}
            Some(Content::Code(s)) => self.output.push(Chunk::new_code(s)),
            Some(Content::Normal(s)) => self.output.push(Chunk::new(s)),
        }
    }
}

pub struct Chunk {
    raw_content: Arc<String>,
    output_mode: OutputMode,
}

impl Chunk {
    pub fn new(raw_content: String) -> Self {
        let items = iced::widget::markdown::parse(&raw_content).collect();
        Self {
            raw_content: Arc::new(raw_content),
            output_mode: OutputMode::Text(items),
        }
    }

    pub fn new_code(raw_content: String) -> Self {
        if let Some((code_type, content)) = raw_content.split_once("\n") {
            Self {
                raw_content: Arc::new(content.to_string()),
                output_mode: OutputMode::Code(
                    code_type.to_string(),
                    iced::widget::text_editor::Content::with_text(content),
                ),
            }
        } else {
            let content = iced::widget::text_editor::Content::with_text(&raw_content);
            Self {
                raw_content: Arc::new(raw_content),
                output_mode: OutputMode::Code(String::new(), content),
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match &self.output_mode {
            OutputMode::Text(items) =>
            //rich_text([span(self.raw_content.as_str())]).into(),
            {
                iced::widget::markdown(
                    items,
                    iced::widget::markdown::Settings::default(),
                    iced::widget::markdown::Style::from_palette(
                        iced::Theme::TokyoNightStorm.palette(),
                    ),
                )
                .map(Message::LinkClicked)
                .into()
            }
            OutputMode::Code(_code_type, content) => row![]
                .push(
                    button_icon(iced_fonts::Bootstrap::Clipboard)
                        .on_press(Message::CopyClipboard(self.raw_content.clone())),
                )
                .push(
                    iced::widget::TextEditor::new(content)
                        .style(|theme, style| {
                            let mut style = iced::widget::text_editor::default(theme, style);
                            style.background =
                                iced::Background::Color(iced::Color::from_rgb8(0, 0, 0));
                            style
                        })
                        .highlight(_code_type, iced::highlighter::Theme::InspiredGitHub)
                        .font(iced::Font::MONOSPACE),
                )
                .spacing(10.0)
                .into(),
        }
    }
}

pub struct MarkdownIncremental {
    context: MarkdownContext,
    buf: String,
    pos: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MarkdownContext {
    Normal,
    Code,
}

enum Content {
    Code(String),
    Normal(String),
}

enum ContentFound {
    NewParagraph(usize),
    CodeSyntax(usize),
}

impl MarkdownIncremental {
    pub fn new() -> Self {
        Self {
            context: MarkdownContext::Normal,
            buf: String::new(),
            pos: 0,
        }
    }

    pub fn add_content(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn process_content(&mut self) -> Option<Content> {
        let remaining = &self.buf[self.pos..];
        match self.context {
            MarkdownContext::Normal => match next_chunk(remaining) {
                None => None,
                Some(ContentFound::NewParagraph(idx)) => {
                    let s = &self.buf[self.pos..self.pos + idx];
                    self.pos += idx + 2;
                    Some(Content::Normal(s.to_string()))
                }
                Some(ContentFound::CodeSyntax(idx)) => {
                    let s = &self.buf[self.pos..self.pos + idx];
                    self.pos += idx + 3;
                    self.context = MarkdownContext::Code;
                    Some(Content::Normal(s.to_string()))
                }
            },
            MarkdownContext::Code => match remaining.find("```") {
                None => None,
                Some(idx) => {
                    let s = &self.buf[self.pos..self.pos + idx];
                    self.pos += idx + 3;
                    self.context = MarkdownContext::Normal;
                    Some(Content::Code(s.to_string()))
                }
            },
        }
    }
}

fn next_chunk(s: &str) -> Option<ContentFound> {
    let z1 = s.find("```");
    let z2 = s.find("\n\n");
    match (z1, z2) {
        (Some(z1), Some(z2)) => {
            if z1 < z2 {
                Some(ContentFound::CodeSyntax(z1))
            } else {
                Some(ContentFound::NewParagraph(z2))
            }
        }
        (Some(z1), None) => Some(ContentFound::CodeSyntax(z1)),
        (None, Some(z2)) => Some(ContentFound::NewParagraph(z2)),
        (None, None) => None,
    }
}

use std::{sync::Arc, time::SystemTime};

use chrono::{DateTime, Local};
use iced::{
    widget::{column, container, row, scrollable, text, text_editor, Container},
    Element, Length, Padding,
};
use ulid::Ulid;

use crate::{
    api,
    helper::button_icon,
    history::{Party, SavedChat},
    Message,
};

pub struct Chat {
    pub previous: SavedChat<ChatOutput>,
    pub state: ChatState,
}

pub enum ChatState {
    Prompting(iced::widget::text_editor::Content),
    Generating(ChatGenerating),
}

impl Default for ChatState {
    fn default() -> Self {
        Self::Prompting(iced::widget::text_editor::Content::new())
    }
}

pub struct ChatGenerating {
    prompt: String,
    start: SystemTime,
    output: ChatOutput,
}

impl ChatGenerating {
    fn new(prompt: String) -> Self {
        Self {
            prompt,
            start: SystemTime::now(),
            output: ChatOutput::new(),
        }
    }
}

impl Chat {
    pub fn new(model: api::LocalModel) -> Self {
        Self {
            previous: SavedChat {
                ulid: Ulid::new(),
                model: model.name().clone(),
                content: vec![],
            },
            state: ChatState::default(),
        }
    }

    pub fn ulid(&self) -> Ulid {
        self.previous.ulid
    }

    pub fn model(&self) -> String {
        self.previous.model.clone()
    }

    pub fn from_saved(chat: SavedChat<String>) -> Self {
        let previous = chat.to_chat_output();
        Self {
            previous,
            state: ChatState::default(),
        }
    }

    pub fn to_saved(&self) -> SavedChat<String> {
        self.previous.clone().flatten_output()
    }

    pub fn name(&self) -> String {
        let time = self.previous.ulid.datetime();
        let date: DateTime<Local> = time.clone().into();

        format!("Chat {}", date.format("%Y-%m-%d %H:%M:%S"))
    }

    pub fn set_generating(&mut self) -> String {
        match &mut self.state {
            ChatState::Prompting(prompt) => {
                let prompt = prompt.text();
                self.state = ChatState::Generating(ChatGenerating::new(prompt.clone()));
                prompt
            }
            ChatState::Generating(_) => {
                tracing::error!("set generating in already generating mode");
                String::new()
            }
        }
    }

    pub fn set_finish(&mut self) {
        let mut prev_state = ChatState::default();
        std::mem::swap(&mut prev_state, &mut self.state);
        match prev_state {
            ChatState::Prompting(content) => {
                tracing::error!("set finish in already prompting mode");
                // put back the previous state
                self.state = ChatState::Prompting(content);
            }
            ChatState::Generating(generating) => {
                self.previous.content.push(Party::Query(generating.prompt));
                self.previous.content.push(Party::Reply(generating.output));
            }
        }
    }

    pub fn view(&self) -> Container<Message> {
        let previous_chunks = self.previous.content.iter().map(|p| match p {
            Party::Query(q) => Self::view_prompt(q).into(),
            Party::Reply(o) => Self::view_output(o).into(),
        });

        let chunks: Box<dyn Iterator<Item = Element<'_, Message>> + '_> = match &self.state {
            ChatState::Prompting(content) => Box::new(
                previous_chunks.chain(std::iter::once(Self::view_prompt_editor(&content).into())),
            ),
            ChatState::Generating(chat_generating) => Box::new(
                previous_chunks
                    .chain(std::iter::once(
                        Self::view_prompt(&chat_generating.prompt).into(),
                    ))
                    .chain(std::iter::once(
                        Self::view_output(&chat_generating.output).into(),
                    )),
            ),
        };
        container(
            scrollable(
                container(column(chunks).spacing(15.0))
                    .padding(Padding::default().left(10.0).right(20.0)),
            )
            .anchor_bottom(),
        )
        .padding(Padding::from(5.0))
    }

    fn view_prompt_editor<'a>(
        content: &'a iced::widget::text_editor::Content,
    ) -> Container<'a, Message> {
        container(
            row![]
                .push(
                    text_editor(&content)
                        .placeholder("Type something here...")
                        .on_action(Message::ChatEditPrompt)
                        .key_binding(|key_press| match key_press.key.as_ref() {
                            iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                                if key_press.modifiers.command() =>
                            {
                                Some(iced::widget::text_editor::Binding::Custom(
                                    Message::ChatSend,
                                ))
                            }
                            _ => text_editor::Binding::from_key_press(key_press),
                        }),
                )
                .push(
                    button_icon(iced_fonts::Bootstrap::Send)
                        .on_press_maybe((!content.text().is_empty()).then_some(Message::ChatSend)),
                )
                .spacing(5.0),
        )
    }

    fn view_prompt<'a>(prompt: &'a str) -> Container<'a, Message> {
        container(container(text(prompt)).padding(Padding::default().left(5.0).right(5.0)))
            .style(container::bordered_box)
            .center_x(Length::Fill)
            .padding(
                Padding::default()
                    .top(5.0)
                    .bottom(5.0)
                    .left(30.0)
                    .right(30.0),
            )
    }

    fn view_output<'a>(output: &'a ChatOutput) -> Container<'a, Message> {
        output.view()
    }

    pub fn add_content(&mut self, response: api::ChatMessageResponse) {
        match &mut self.state {
            ChatState::Prompting(_) => {
                tracing::error!("chat message appended in prompt mode")
            }
            ChatState::Generating(generating) => {
                generating.output.add_content(&response.message.content)
            }
        }
    }
}

#[derive(Clone)]
pub enum OutputMode {
    Text(Vec<iced::widget::markdown::Item>),
    Code(String, Arc<iced::widget::text_editor::Content>),
}

#[derive(Clone)]
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

    pub fn raw(&self) -> String {
        self.stream.buf.clone()
    }

    fn unparsed(&self) -> &str {
        &self.stream.buf[self.stream.pos..]
    }

    pub fn view<'a>(&'a self) -> Container<'a, Message> {
        let rem = std::iter::once(text(self.unparsed()).into());
        container(column(self.output.iter().map(|c| c.view()).chain(rem)).spacing(20.0))
    }

    pub fn add_content(&mut self, message: &str) {
        self.stream.add_content(message);
        loop {
            match self.stream.process_content() {
                None => {
                    break;
                }
                Some(Content::Code(s)) => self.output.push(Chunk::new_code(s)),
                Some(Content::Normal(s)) => self.output.push(Chunk::new(s)),
            }
        }
    }
}

#[derive(Clone)]
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
                    Arc::new(iced::widget::text_editor::Content::with_text(content)),
                ),
            }
        } else {
            let content = iced::widget::text_editor::Content::with_text(&raw_content);
            Self {
                raw_content: Arc::new(raw_content),
                output_mode: OutputMode::Code(String::new(), Arc::new(content)),
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match &self.output_mode {
            OutputMode::Text(items) => iced::widget::markdown(
                items,
                iced::widget::markdown::Settings::default(),
                iced::widget::markdown::Style::from_palette(iced::Theme::TokyoNightStorm.palette()),
            )
            .map(Message::LinkClicked)
            .into(),
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

#[derive(Clone)]
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
            MarkdownContext::Normal => match normal_next_chunk(remaining) {
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

// find either a double newline or a triple backquote, whichever comes first
fn normal_next_chunk(s: &str) -> Option<ContentFound> {
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

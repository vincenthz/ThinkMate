use std::time::{Duration, SystemTime};

use helper::{button_icon_small, button_icon_text};
use iced::{
    font::{Family, Weight},
    widget::{
        button, column, combo_box, container, horizontal_rule, horizontal_space, row, text,
        vertical_space, Container,
    },
    Alignment, Color, Element, Font, Length, Padding, Size, Subscription, Task, Theme,
};
use indicator::Indicator;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream;
use sidebar::Sidebar;
use std::sync::Arc;
use ulid::Ulid;
use url::Url;

mod api;
mod chat;
mod helper;
mod indicator;
mod sidebar;
mod utils;

use chat::{Chat, ChatState};

#[derive(Clone, Debug)]
pub enum Message {
    ModelSelected(api::LocalModel),
    WorkerReady(mpsc::Sender<WorkerInput>),
    Connected,
    ModelsChanged(Vec<api::LocalModel>),
    Disconnected,
    NewChat(api::LocalModel),
    SidebarVisibilityToggle,
    ChatClosed(usize),
    ChatSelected(usize),
    ChatEditPrompt(iced::widget::text_editor::Action),
    ChatSend,
    ChatStreamStart(Ulid, api::ChatMessageResponseStream),
    ChatStream(Ulid, api::ChatMessageResponse),
    ChatStreamFinished(Ulid),
    CopyClipboard(Arc<String>),
    LinkClicked(Url),
}

fn main() -> iced::Result {
    let mut font = Font::with_name("Fira Sans");
    font.weight = Weight::Semibold;
    font.family = Family::SansSerif;
    let app = iced::application(ThinkMate::title, ThinkMate::update, ThinkMate::view)
        .theme(ThinkMate::theme)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .default_font(font)
        .centered()
        .window_size(Size {
            width: 1280.0,
            height: 1024.0,
        })
        .antialiasing(true)
        .subscription(ThinkMate::subscription);
    app.run_with(|| ThinkMate::new())
}

pub struct ThinkMate {
    ollama_config: api::OllamaConfig,
    menubar: Menubar,
    main: Main,
    worker: Option<mpsc::Sender<WorkerInput>>,
}

pub enum WorkerInput {
    Monitor(api::OllamaConfig),
}

impl ThinkMate {
    fn new() -> (Self, Task<Message>) {
        let me = Self {
            ollama_config: api::OllamaConfig::localhost(api::DEFAULT_PORT),
            menubar: Menubar::new(),
            main: Main::new(),
            worker: None,
        };
        (me, Task::none())
    }

    fn set_models(&mut self, models: Vec<api::LocalModel>) {
        self.menubar.set_models(models);
    }

    fn set_connected(&mut self, connected: bool) {
        self.menubar.connected = connected;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ModelSelected(m) => {
                self.menubar.selected = Some(m);
                Task::none()
            }
            Message::WorkerReady(sender) => {
                let mut sender2 = sender.clone();
                let config = self.ollama_config.clone();
                let to_send = async move {
                    sender2
                        .send(WorkerInput::Monitor(config))
                        .await
                        .unwrap_or(());
                };
                self.worker = Some(sender);
                Task::future(to_send).then(|_| Task::none())
            }
            Message::Connected => {
                self.set_connected(true);
                Task::none()
            }
            Message::ModelsChanged(models) => {
                self.set_models(models);
                Task::none()
            }
            Message::Disconnected => {
                self.set_models(vec![]);
                self.set_connected(false);
                Task::none()
            }
            Message::NewChat(local_model) => {
                self.main.add_new(local_model);
                Task::none()
            }
            Message::ChatClosed(chat_closing) => {
                self.main.tabs.remove(chat_closing);
                Task::none()
            }
            Message::ChatEditPrompt(text_action) => {
                let chat = &mut self.main.tabs[self.main.chat_view];
                match &mut chat.state {
                    ChatState::Prompting(content) => content.perform(text_action),
                    ChatState::Generate {
                        start: _,
                        prompt: _,
                        output: _,
                        ended_at: _,
                    } => {}
                };
                Task::none()
            }
            Message::ChatSelected(chat_selected) => {
                println!("chat selected {}", chat_selected);
                self.main.chat_view = chat_selected;
                Task::none()
            }
            Message::ChatSend => {
                let chat = &mut self.main.tabs[self.main.chat_view];
                let ulid = chat.ulid.clone();
                let model = chat.model.name().clone();
                let prompt = chat.set_generating().to_string();
                let config = &self.ollama_config.clone();
                let api = config.instance();
                Task::perform(api::chat_stream(api, model, prompt), move |stream| {
                    Message::ChatStreamStart(ulid, stream)
                })
            }
            Message::ChatStreamStart(ulid, chat_message_response_stream) => {
                println!("chat stream start");
                let ulid = ulid.clone();
                Task::run(chat_message_response_stream.0, move |x| {
                    Message::ChatStream(ulid, x.unwrap())
                })
                .chain(Task::done(Message::ChatStreamFinished(ulid)))
            }
            Message::ChatStream(ulid, chat_message_response) => {
                if let Some(chat) = self.main.find_chat_mut(ulid) {
                    chat.add_content(chat_message_response);
                    Task::none()
                } else {
                    Task::none()
                }
            }
            Message::ChatStreamFinished(ulid) => {
                if let Some(chat) = self.main.find_chat_mut(ulid) {
                    match &mut chat.state {
                        ChatState::Prompting(_) => {}
                        ChatState::Generate {
                            start: _,
                            prompt: _,
                            output: _,
                            ended_at,
                        } => {
                            let time = SystemTime::now();
                            *ended_at = Some(time);
                        }
                    }
                }
                Task::none()
            }
            Message::SidebarVisibilityToggle => {
                self.main.sidebar_visibility = self.main.sidebar_visibility.toggle();
                Task::none()
            }
            Message::CopyClipboard(s) => iced::clipboard::write(s.as_str().to_string()),
            Message::LinkClicked(_) => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(background_worker)
    }

    fn title(&self) -> String {
        "ThinkMate".to_string()
    }

    fn theme(&self) -> Theme {
        // to not use darklight crate directly, rely on the default theme being Dark or Light.
        let _system_use_dark = Theme::default() == Theme::Dark;
        Theme::default()
    }

    fn view(&self) -> Container<Message> {
        container(
            column![]
                .push(self.menubar.view().height(Length::Fixed(40.0)))
                .push(
                    row![]
                        .push(self.main.view().width(Length::Fill))
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .padding(Padding::default().top(5.0).top(5.0)),
                ),
        )
        .center(Length::Fill)
        .padding(3)
    }
}

fn background_worker() -> impl Stream<Item = Message> {
    stream::channel(10, |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);

        output.send(Message::WorkerReady(sender)).await.unwrap();

        loop {
            let input = receiver.select_next_some().await;
            match input {
                WorkerInput::Monitor(config) => {
                    let output = output.clone();
                    tokio::spawn(async move { monitor(output, config).await });
                }
            }
        }
    })
}

async fn monitor(mut output: mpsc::Sender<Message>, config: api::OllamaConfig) {
    let mut previous_models = Vec::new();
    let api = config.instance();
    loop {
        match api::get_model_lists(&api).await {
            Err(_) => {
                output.send(Message::Disconnected).await.unwrap();
            }
            Ok(models) => {
                output.send(Message::Connected).await.unwrap();
                if previous_models != models {
                    previous_models = models.clone();
                    output.send(Message::ModelsChanged(models)).await.unwrap();
                } else {
                }
            }
        }
        tokio::time::sleep(Duration::new(10, 0)).await
    }
}

pub struct Menubar {
    connected: bool,
    model: combo_box::State<api::LocalModel>,
    selected: Option<api::LocalModel>,
}

impl Menubar {
    pub fn new() -> Self {
        Self {
            connected: false,
            model: combo_box::State::new(vec![]),
            selected: None,
        }
    }

    pub fn view(&self) -> Container<Message> {
        let indicator_color = if self.connected {
            Color::from_rgb8(0, 0x9f, 0)
        } else {
            Color::from_rgb8(0x9f, 0, 0)
        };
        container(
            row![]
                .push(
                    text("ThinkMate")
                        .color(Color::from_rgb8(0x60, 0x0, 0x12))
                        .size(20.0),
                )
                .push(horizontal_space())
                .push(
                    combo_box(
                        &self.model,
                        "Select Model",
                        self.selected.as_ref(),
                        Message::ModelSelected,
                    )
                    .width(Length::Fixed(180.0)),
                )
                .push(
                    button_icon_text(iced_fonts::Bootstrap::Plus, "New Chat").on_press_maybe(
                        self.selected.as_ref().map(|s| Message::NewChat(s.clone())),
                    ),
                )
                .push(Indicator::new().circle_radius(8.0).color(indicator_color))
                .spacing(10.0)
                .align_y(Alignment::Center),
        )
        .center(Length::Fill)
        .padding(Padding::default().left(5.0).right(5.0))
        .style(container::bordered_box)
    }

    pub fn set_models(&mut self, models: Vec<api::LocalModel>) {
        if models.is_empty() {
            self.selected = None;
        }
        self.model = combo_box::State::with_selection(models, self.selected.as_ref());
    }
}

pub struct Main {
    home: EmptyChats,
    chat_view: usize,
    tabs: Vec<Chat>,
    sidebar: Sidebar,
    sidebar_visibility: SidebarVisibility,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SidebarVisibility {
    #[default]
    Expanded,
    Collapsed,
}

impl SidebarVisibility {
    pub fn toggle(self) -> Self {
        match self {
            SidebarVisibility::Expanded => SidebarVisibility::Collapsed,
            SidebarVisibility::Collapsed => SidebarVisibility::Expanded,
        }
    }
}

impl Main {
    pub fn new() -> Self {
        Self {
            home: EmptyChats::new(),
            chat_view: 0,
            tabs: vec![],
            sidebar: Sidebar::new(),
            sidebar_visibility: SidebarVisibility::default(),
        }
    }

    pub fn view(&self) -> Container<Message> {
        let main = if self.tabs.is_empty() {
            container(self.home.view())
        } else {
            let view = self.chat_view;
            let tab_bar_elements = self
                .tabs
                .iter()
                .enumerate()
                .map(|(i, chat)| {
                    let selected = i == view;
                    let label = text(chat.name());
                    let close = button_icon_small(iced_fonts::Bootstrap::X)
                        .padding(1.0)
                        .style(|theme, status| button::danger(theme, status))
                        .on_press(Message::ChatClosed(i));
                    button(
                        row![]
                            .push(label)
                            .push(close)
                            .spacing(10.0)
                            .align_y(Alignment::Center),
                    )
                    .on_press(Message::ChatSelected(i))
                    .style(move |theme, status| {
                        if selected {
                            button::primary(theme, status)
                        } else {
                            button::secondary(theme, status)
                        }
                    })
                })
                .map(|b| Element::from(b));
            let tab_bar = row(tab_bar_elements).width(Length::Fill).spacing(5.0);
            if let Some(chat) = self.tabs.get(view) {
                container(
                    column![]
                        .push(tab_bar)
                        .push(horizontal_rule(1.0))
                        .push(vertical_space().height(5.0))
                        .push(chat.view()),
                )
            } else {
                container(column![].push(tab_bar))
            }
        };

        let sidebar = match self.sidebar_visibility {
            SidebarVisibility::Expanded => self.sidebar.view().width(Length::FillPortion(9)),
            SidebarVisibility::Collapsed => {
                self.sidebar.view_collapse().width(Length::FillPortion(1))
            }
        };
        container(
            row![]
                .push(main.width(Length::FillPortion(32)))
                .push(sidebar),
        )
    }

    pub fn add_new(&mut self, model: api::LocalModel) {
        self.tabs.push(Chat::new(model))
    }

    pub fn find_chat(&self, ulid: Ulid) -> Option<&Chat> {
        self.tabs.iter().find(|chat| chat.ulid == ulid)
    }

    pub fn find_chat_mut(&mut self, ulid: Ulid) -> Option<&mut Chat> {
        self.tabs.iter_mut().find(|chat| chat.ulid == ulid)
    }
}

#[derive(Clone)]
pub struct EmptyChats {}

impl EmptyChats {
    pub fn new() -> Self {
        EmptyChats {}
    }

    pub fn view(&self) -> Container<Message> {
        container(
            column![]
                .push(
                    text(
                        "To get started create a new chat or open a previous chat from the sidebar",
                    )
                    .style(|theme| text::secondary(theme)),
                )
                .spacing(10.0),
        )
        .center(Length::Fill)
    }
}

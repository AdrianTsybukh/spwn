mod plugins;
mod plugin;

use crate::plugin::Plugin;
use crate::plugins::apps::{get_all_apps, App};
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::{self, key::Named};
use iced::widget::{column, container, scrollable, text, text_input};
use iced::{event, stream, window};
use iced::{Color, Element, Event, Font, Length, Size, Subscription, Task, Theme};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

const WINDOW_WIDTH: f32 = 600.0;
const ITEM_HEIGHT: f32 = 50.0;
const INPUT_HEADER_HEIGHT: f32 = 76.0; 
const MAX_WINDOW_HEIGHT: f32 = 522.0;

#[derive(Debug, Clone)]
enum Message {
    Init(iced::window::Id),
    Run,
    Content(String),
    Completed(Result<String, String>),
    Close,
    NextResult,
    PrevResult,
    Toggle,
}

struct Launcher {
    prompt: String,
    output: String,
    input_id: iced::widget::Id,
    window_id: iced::window::Id,
    plugins: Vec<Box<dyn Plugin>>,
    apps: Vec<App>,
    filtered_apps: Vec<App>,
    selected_index: usize,
    hotkey_manager: GlobalHotKeyManager,
    is_visible: bool,
}

impl Launcher {
    fn new() -> (Self, Task<Message>) {
        let input_id = iced::widget::Id::unique();
        let window_id = iced::window::Id::unique();

        let manager = GlobalHotKeyManager::new().expect("error while creating hotkey manager.");
        let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Space);
        manager.register(hotkey).expect("failed to register hotkey.");

        (
            Self {
                prompt: String::new(),
                output: String::new(),
                input_id: input_id.clone(),
                window_id,
                plugins: vec![
                    Box::new(plugins::shell::Shell),
                    Box::new(plugins::app::AppPlugin),
                ],
                apps: get_all_apps(),
                filtered_apps: vec![],
                selected_index: 0,
                hotkey_manager: manager,
                is_visible: true,
            },
            Task::batch(vec![
                iced::widget::operation::focus(input_id),
                window::latest().map(|id| Message::Init(id.expect("Window should exist at startup"))),
            ]),
        )
    }

    fn adjust_window_size(&self) -> Task<Message> {
        let mut content_height = 0.0;

        if !self.output.is_empty() {
             return window::resize(self.window_id, Size::new(WINDOW_WIDTH, MAX_WINDOW_HEIGHT));
        }

        if !self.filtered_apps.is_empty() {
            let list_height = self.filtered_apps.len() as f32 * ITEM_HEIGHT;
            content_height += list_height;
        }

        let total_required = INPUT_HEADER_HEIGHT + content_height;

        let final_height = total_required.clamp(INPUT_HEADER_HEIGHT, MAX_WINDOW_HEIGHT);

        window::resize(self.window_id, Size::new(WINDOW_WIDTH, final_height))
    }

    fn subscription(&self) -> Subscription<Message> {
        let hotkey_subscription = Subscription::run(|| {
            stream::channel(1, |mut output: Sender<Message>| async move {
                loop {
                    if let Ok(event) = tokio::task::spawn_blocking(|| {
                        GlobalHotKeyEvent::receiver().recv()
                    })
                    .await
                    .unwrap()
                    {
                        if event.state == HotKeyState::Pressed {
                            let _ = output.send(Message::Toggle).await;
                        }
                    }
                }
            })
        });

        Subscription::batch(vec![
            hotkey_subscription,
            event::listen().filter_map(|event| {
                if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                    match key {
                        keyboard::Key::Named(Named::Escape) => Some(Message::Close),
                        keyboard::Key::Named(Named::ArrowDown) => Some(Message::NextResult),
                        keyboard::Key::Named(Named::ArrowUp) => Some(Message::PrevResult),
                        _ => None,
                    }
                } else {
                    None
                }
            }),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Init(id) => {
                self.window_id = id;
                return Task::none();
            }

            Message::Content(prompt) => {
                self.prompt = prompt;
                self.selected_index = 0;
                self.output.clear();

                if self.prompt.is_empty() {
                    self.filtered_apps.clear();
                } else if !self.prompt.starts_with('>') {
                    self.filtered_apps = self.apps
                        .iter()
                        .filter(|app| app.name.to_lowercase().contains(&self.prompt.to_lowercase()))
                        .cloned()
                        .collect();
                } else {
                    self.filtered_apps.clear();
                }

                return self.adjust_window_size();
            }

            Message::NextResult => {
                if !self.filtered_apps.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.filtered_apps.len();
                }
                return Task::none();
            }

            Message::PrevResult => {
                if !self.filtered_apps.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.filtered_apps.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
                return Task::none();
            }

            Message::Run => {
                for plugin in &self.plugins {
                    if plugin.can_handle(&self.prompt) {
                        return plugin.execute(&self.prompt).map(Message::Completed);
                    }
                }
                
                if let Some(app) = self.filtered_apps.get(self.selected_index) {
                    println!("Launching: {}", app.name);
                    self.is_visible = false;
                    
                    let exec_path = app.exec_path.clone();
                    let plugin_run = self.plugins[1].execute(&exec_path).map(Message::Completed);
                    
                    return Task::batch(vec![
                        plugin_run,
                        window::set_mode(self.window_id, window::Mode::Hidden),
                    ]);
                }
                return Task::none();
            }

            Message::Completed(result) => {
                match result {
                    Ok(out) => self.output = out,
                    Err(e) => self.output = e,
                }
                return self.adjust_window_size();
            }

            Message::Close => {
                self.is_visible = false;
                return window::set_mode(self.window_id, window::Mode::Hidden)
            }

            Message::Toggle => {
                if !self.is_visible {
                    self.is_visible = true;
                    self.prompt.clear();
                    self.output.clear();
                    self.filtered_apps.clear();

                    return Task::batch(vec![
                        window::resize(self.window_id, Size::new(WINDOW_WIDTH, INPUT_HEADER_HEIGHT)),
                        window::set_mode(self.window_id, window::Mode::Windowed),
                        window::gain_focus(self.window_id),
                        iced::widget::operation::focus(self.input_id.clone()),
                    ]);
                } else {
                    self.is_visible = false;
                    return window::set_mode(self.window_id, window::Mode::Hidden);
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search apps or use > for commands...", &self.prompt)
            .id(self.input_id.clone())
            .on_input(Message::Content)
            .on_submit(Message::Run)
            .padding(15)
            .size(20)
            .font(Font::MONOSPACE);

        let mut content = column![container(input).padding(10)].spacing(0);

        if !self.filtered_apps.is_empty() {
            let app_list = column(
                self.filtered_apps.iter().enumerate().map(|(i, app)| {
                    let is_selected = i == self.selected_index;

                    container(
                        text(&app.name)
                            .size(16)
                            .font(Font::MONOSPACE)
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(ITEM_HEIGHT))
                    .padding(15)
                    .center_y(Length::Fill)
                    .style(move |_theme| {
                        if is_selected {
                            container::Style::default().background(Color::from_rgb(0.2, 0.4, 0.8)).color(Color::WHITE)
                        } else {
                            container::Style::default()
                        }
                    })
                    .into()
                }).collect::<Vec<Element<'_, Message>>>()
            );

            content = content.push(scrollable(app_list).height(Length::Fill));
        }

        if !self.output.is_empty() {
            content = content.push(
                scrollable(
                    container(text(&self.output).font(Font::MONOSPACE))
                        .padding(15)
                        .width(Length::Fill)
                ).height(Length::Fill)
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn main() -> iced::Result {
    let settings = window::Settings {
        decorations: false,
        resizable: false,
        size: Size { width: WINDOW_WIDTH, height: INPUT_HEADER_HEIGHT },
        level: window::Level::AlwaysOnTop,
        ..Default::default()
    };

    iced::application(Launcher::new, Launcher::update, Launcher::view)
        .title("spwn")
        .window(settings)
        .subscription(Launcher::subscription)
        .centered()
        .exit_on_close_request(false)
        .theme(Theme::Dark)
        .run()
}

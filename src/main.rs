mod plugins;
mod plugin;

use crate::plugin::Plugin;
use crate::plugins::apps::{App, get_all_apps};
use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Color, Element, Font, Length, Size, Task, Theme, window};
use iced::{Event, Subscription, event};
use iced::keyboard::{self, key::Named};

#[derive(Debug, Clone)]
enum Message {
    Run,
    Content(String),
    Completed(Result<String, String>),
    Close,
    NextResult,
    PrevResult,
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
}

impl Launcher {
    fn new() -> (Self, Task<Message>) {
        let input_id = iced::widget::Id::unique();
        let window_id = iced::window::Id::unique();
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
            },
            iced::widget::operation::focus(input_id)
        )
    }

    fn subscription(&self) -> Subscription<Message> {
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
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Content(prompt) => {
                self.prompt = prompt;
                self.selected_index = 0;
                self.output.clear();

                if self.prompt.is_empty() {
                    self.filtered_apps.clear();
                    return window::resize(self.window_id, Size { width: 500.0, height: 60.0 });
                }

                if !self.prompt.starts_with('>') {
                    self.filtered_apps = self.apps
                        .iter()
                        .filter(|app| app.name.to_lowercase().contains(&self.prompt.to_lowercase()))
                        .cloned()
                        .collect();
                } else {
                    self.filtered_apps.clear();
                }

                let height = if self.filtered_apps.is_empty() { 60.0 } else { 400.0 };
                return window::resize(self.window_id, Size { width: 500.0, height });
            }

            Message::NextResult => {
                if !self.filtered_apps.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.filtered_apps.len();
                }
            }

            Message::PrevResult => {
                if !self.filtered_apps.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.filtered_apps.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
            }

            Message::Run => {
                for plugin in &self.plugins {
                    if plugin.can_handle(&self.prompt) {
                        return plugin.execute(&self.prompt).map(Message::Completed);
                    } else {
                        if let Some(app) = self.filtered_apps.get(self.selected_index) {
                            println!("Launching: {}", app.name);
                            let res = plugin.execute(app.exec_path.as_str()).map(Message::Completed);
                            return res;
                        }
                    }
                }
            }

            Message::Completed(result) => {
                match result {
                    Ok(out) => self.output = out,
                    Err(e) => self.output = e,
                }
                return window::resize(self.window_id, Size { width: 500.0, height: 500.0 });
            }

            Message::Close => std::process::exit(0),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("Search apps or use > for commands...", &self.prompt)
            .id(self.input_id.clone())
            .on_input(Message::Content)
            .on_submit(Message::Run)
            .padding(15)
            .size(20);

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
                    .padding(10)
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

            content = content.push(scrollable(app_list).height(Length::Fixed(340.0)));
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
        size: Size { width: 500.0, height: 500.0 },
        level: window::Level::AlwaysOnTop,
        ..Default::default()
    };

    iced::application(Launcher::new, Launcher::update, Launcher::view)
        .title("spwn")
        .window(settings)
        .subscription(Launcher::subscription)
        .centered()
        .theme(Theme::Dark)
        .run()
}

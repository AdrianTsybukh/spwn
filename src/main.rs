mod plugins;
mod plugin;

use crate::plugin::Plugin;
use crate::plugins::apps::{App, get_all_apps};
use iced::widget::{column, text, text_input};
use iced::{Element, Size, Task, window};
use iced::{Event, Subscription, event};
use iced::keyboard::{self, key::Named};

// повідомлення програми. можна сказати тригери для самого функціоналу.
#[derive(Debug, Clone)]
enum Message {
    Run,
    Content(String),
    Completed(Result<String, String>),
    Close,
}

// статус програми. збереження основної інформації
struct Launcher {
    prompt: String,
    output: String,
    input_id: iced::widget::Id,
    plugins: Vec<Box<dyn Plugin>>,
    apps: Vec<App>,
}

impl Launcher {
    fn new() -> (Self, Task<Message>) {
        let input_id = iced::widget::Id::unique();
        let apps = get_all_apps();

        (
            Self {
                prompt: String::new(),
                output: String::new(),
                input_id: input_id.clone(),
                plugins: vec![ // список плагінів. поки тільки shell для виконування команд. будемо
                               // додавати інші(калькулятор, запуск програм, пошук веб)
                    Box::new(plugins::shell::Shell),
                ],
                apps: apps,
            },
            iced::widget::operation::focus(input_id)
        )
    }
}

impl Launcher {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().filter_map(|event| {
            match event {
                Event::Keyboard(keyboard::Event::KeyReleased {
                    key: keyboard::Key::Named(Named::Escape), // вихід при натисканні escape
                    ..
                }) => Some(Message::Close),

                _ => None,
            }
        }) 
    }
}

impl Launcher {
    fn view(&self) -> Element<'_, Message> { // метод view для малювання інтерфейсу.
        column![
            text_input("enter", &self.prompt).id(self.input_id.clone()).on_input(Message::Content).on_submit(Message::Run),
            text(&self.output),
        ].into()
    }
}

impl Launcher {
    fn update(&mut self, message: Message) -> Task<Message> { // метод update для оновлення та
                                                              // обробки повідомлень
        match message {
            Message::Run => {
                let prompt = self.prompt.clone();
                for plugin in &self.plugins {
                    if plugin.can_handle(&prompt) {
                        return plugin.execute(&prompt).map(Message::Completed);
                    }
                }
                self.output = "No plugin found.".to_string();
            }
            Message::Completed(result) => {
                match result {
                    Ok(out) => self.output = out,
                    Err(e) => self.output = e.to_string(),
                }
            }
            Message::Content(prompt) => {
                self.prompt = prompt;
            }
            Message::Close => {
                std::process::exit(0);
            }
        }
        Task::none()
    }
}


fn main() -> iced::Result {
    let settings = iced::window::Settings {
        decorations: false,
        resizable: false,
        size: Size {width: 500_f32, height: 500_f32},
        level: window::Level::AlwaysOnTop,
        ..Default::default()
    };
    iced::application(Launcher::new, Launcher::update, Launcher::view)
        .title("spwn")
        .window(settings)
        .subscription(Launcher::subscription)
        .centered()
        .run()
}

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::column;
use iced::{Element, Fill, Subscription, Task, Theme};

use crate::canvas::PianoRollGrid;
use crate::status_bar;

pub struct App {
    mode: Mode,
    grid: PianoRollGrid,
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(Key, Modifiers),
    Tick,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                mode: Mode::Normal,
                grid: PianoRollGrid::new(),
            },
            Task::none(),
        )
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPressed(_key, _modifiers) => {}
            Message::Tick => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let status = status_bar::view(&self.mode);
        let canvas = self.grid.view();

        column![canvas, status].height(Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Some(Message::KeyPressed(key, modifiers))
            }
            _ => None,
        })
    }
}

pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Motif")
        .theme(App::theme)
        .subscription(App::subscription)
        .centered()
        .run()
}

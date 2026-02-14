use std::cell::RefCell;
use std::collections::HashSet;

use iced::keyboard::{self, Key, Modifiers, key::Named};
use iced::widget::column;
use iced::{Element, Fill, Subscription, Task, Theme};
use motif_core::id::TrackId;
use motif_engine::control::PlaybackControl;
use motif_engine::events::MidiEvent;
use wmidi::{Note, Velocity};

use crate::canvas::PianoRollGrid;
use crate::status_bar;

pub struct App {
    mode: Mode,
    grid: PianoRollGrid,
    control: PlaybackControl,
    /// Tracks currently-held notes so key repeat does not flood NoteOn
    /// and mode exits can reliably silence everything.
    active_notes: HashSet<Note>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Play,
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(Key, Modifiers),
    KeyReleased(Key, Modifiers),
    Tick,
}

impl App {
    fn new(control: PlaybackControl) -> (Self, Task<Message>) {
        (
            Self {
                mode: Mode::Normal,
                grid: PianoRollGrid::new(),
                control,
                active_notes: HashSet::new(),
            },
            Task::none(),
        )
    }

    fn key_to_note(key: &Key) -> Option<Note> {
        let Key::Character(chars) = key.as_ref() else {
            return None;
        };

        let mut chars = chars.chars();
        let char = chars.next()?;

        if chars.next().is_some() {
            return None;
        }

        // One-octave "typing keyboard piano": white keys on home row,
        // black keys on top row.
        match char.to_ascii_lowercase() {
            // White keys
            'a' => Some(Note::C4),
            's' => Some(Note::D4),
            'd' => Some(Note::E4),
            'f' => Some(Note::F4),
            'g' => Some(Note::G4),
            'h' => Some(Note::A4),
            'j' => Some(Note::B4),
            'k' => Some(Note::C5),
            'l' => Some(Note::D5),
            ';' => Some(Note::E5),
            // Black keys
            'q' => Some(Note::CSharp4),
            'w' => Some(Note::DSharp4),
            'e' => Some(Note::FSharp4),
            'r' => Some(Note::GSharp4),
            't' => Some(Note::ASharp4),
            'y' => Some(Note::CSharp5),
            'u' => Some(Note::DSharp5),
            'o' => Some(Note::FSharp5),
            'p' => Some(Note::GSharp5),
            _ => None,
        }
    }

    fn note_on(&mut self, note: Note) {
        if self.active_notes.insert(note) {
            let _ = self.control.send_midi(
                TrackId(0),
                MidiEvent::NoteOn {
                    note,
                    velocity: Velocity::MAX,
                },
            );
        }
    }

    fn note_off(&mut self, note: Note) {
        if self.active_notes.remove(&note) {
            let _ = self
                .control
                .send_midi(TrackId(0), MidiEvent::NoteOff { note });
        }
    }

    /// Send note-offs for all held keys when leaving play mode so no
    /// voices are left hanging.
    fn all_notes_off(&mut self) {
        for note in self.active_notes.drain() {
            let _ = self
                .control
                .send_midi(TrackId(0), MidiEvent::NoteOff { note });
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPressed(key, _modifiers) => {
                match key.as_ref() {
                    Key::Character("n") | Key::Character("N") => {
                        self.mode = Mode::Play;
                        return Task::none();
                    }
                    Key::Named(Named::Escape) => {
                        self.mode = Mode::Normal;
                        self.all_notes_off();
                        return Task::none();
                    }
                    _ => {}
                }

                if self.mode == Mode::Play
                    && let Some(note) = Self::key_to_note(&key)
                {
                    self.note_on(note);
                }
            }
            Message::KeyReleased(key, _modifiers) => {
                if self.mode == Mode::Play
                    && let Some(note) = Self::key_to_note(&key)
                {
                    self.note_off(note);
                }
            }
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
            keyboard::Event::KeyReleased { key, modifiers, .. } => {
                Some(Message::KeyReleased(key, modifiers))
            }
            _ => None,
        })
    }
}

pub fn run(control: PlaybackControl) -> iced::Result {
    let control = RefCell::new(Some(control));

    iced::application(
        move || {
            let control = control
                .borrow_mut()
                .take()
                .expect("application boot called more than once");

            App::new(control)
        },
        App::update,
        App::view,
    )
    .title("Motif")
    .theme(App::theme)
    .subscription(App::subscription)
    .centered()
    .run()
}

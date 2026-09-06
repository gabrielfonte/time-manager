use iced::widget::{button, center, column, row, text, text_input};
use iced::{time, Element, Subscription};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum State {
    Idle,
    Ticking {
        last_tick: Instant,
    },
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ProjectNameChanged(String),
    Start,
    Stop,
    Reset,
    Tick(Instant),
}

#[derive(Debug, Default)]
pub struct ProjectTimer {
    duration: Duration,
    state: State,
    project_name: String,
}

impl ProjectTimer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ProjectNameChanged(name) => {
                self.project_name = name;
            }
            Message::Start => {
                if let State::Idle = self.state {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                }
            }
            Message::Stop => {
                if let State::Ticking { .. } = self.state {
                    self.state = State::Idle;
                }
            }
            Message::Reset => {
                self.duration = Duration::ZERO;
                self.state = State::Idle;
            }
            Message::Tick(now) => {
                if let State::Ticking { last_tick } = &mut self.state {
                    self.duration += now.saturating_duration_since(*last_tick);
                    *last_tick = now;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => time::every(Duration::from_secs(1)).map(Message::Tick),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let total_seconds = self.duration.as_secs();
        let hours = total_seconds / 3_600;
        let minutes = (total_seconds % 3_600) / 60;
        let seconds = total_seconds % 60;
        let duration = text(format!("{hours:02}:{minutes:02}:{seconds:02}")).size(40);
        let project_name = if self.project_name.is_empty() {
            "Time Manager"
        } else {
            &self.project_name
        };
        let project_input = text_input("Project name", &self.project_name)
            .on_input(Message::ProjectNameChanged)
            .padding(10)
            .width(250);
        let start_stop = match self.state {
            State::Idle => button("Start").on_press(Message::Start),
            State::Ticking { .. } => button("Stop").on_press(Message::Stop),
        };
        let controls = row![start_stop, button("Reset").on_press(Message::Reset)].spacing(16);

        center(column![project_input, text(project_name).size(20), duration, controls]
            .spacing(20)
            .align_x(iced::Alignment::Center))
        .into()
    }
}
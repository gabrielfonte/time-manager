use chrono::Local;
use iced::{time, Element, Subscription};
use std::time::{Duration, Instant};

use crate::gui::time_entry::{daily_totals_for_week, project_totals_for_day, project_totals_for_month, project_totals_for_year, ProjectTimeEntries, TimeEntry};

#[derive(Debug, Clone, Copy)]
pub enum State {
    Idle,
    Ticking { last_tick: Instant },
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Today,
    Reports,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportRange {
    Week,
    Month,
    Year,
}

impl std::fmt::Display for ReportRange {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Week => "Week",
            Self::Month => "Month",
            Self::Year => "Year",
        })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TrayPoll,
    ScreenSelected(Screen),
    ReportRangeSelected(ReportRange),
    ProjectNameChanged(String),
    ProjectSelected(String),
    Start,
    Stop,
    Reset,
    Tick(Instant),
}

#[derive(Debug)]
pub struct ProjectTimer {
    pub(crate) duration: Duration,
    pub(crate) state: State,
    pub(crate) project_name: String,
    pub(crate) screen: Screen,
    pub(crate) report_range: ReportRange,
    pub(crate) entries: ProjectTimeEntries,
}

impl Default for ProjectTimer {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
            state: State::Idle,
            project_name: "Focus work".to_string(),
            screen: Screen::Today,
            report_range: ReportRange::Week,
            entries: ProjectTimeEntries::new(),
        }
    }
}

impl ProjectTimer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TrayPoll => {}
            Message::ScreenSelected(screen) => self.screen = screen,
            Message::ReportRangeSelected(range) => self.report_range = range,
            Message::ProjectNameChanged(name) => self.project_name = name,
            Message::ProjectSelected(name) => self.project_name = name,
            Message::Start => {
                if matches!(self.state, State::Idle) {
                    self.duration = Duration::ZERO;
                    self.state = State::Ticking { last_tick: Instant::now() };
                }
            }
            Message::Stop => {
                if matches!(self.state, State::Ticking { .. }) {
                    self.record_current_session();
                    self.duration = Duration::ZERO;
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
        let timer = match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => time::every(Duration::from_secs(1)).map(Message::Tick),
        };
        let tray = time::every(Duration::from_millis(100)).map(|_| Message::TrayPoll);
        Subscription::batch([timer, tray])
    }

    pub fn view(&self) -> Element<'_, Message> {
        crate::gui::views::view(self)
    }

    pub(crate) fn project_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.entries.keys().cloned().collect();
        if !self.project_name.is_empty() && !names.contains(&self.project_name) {
            names.push(self.project_name.clone());
        }
        names.sort();
        names
    }

    pub(crate) fn today_projects(&self) -> Vec<(String, u64)> {
        project_totals_for_day(&self.entries, crate::gui::time_entry::today())
    }

    pub(crate) fn today_seconds(&self) -> u64 {
        crate::gui::time_entry::total_for_day(&self.entries, crate::gui::time_entry::today())
    }

    pub(crate) fn week_seconds(&self) -> u64 {
        crate::gui::time_entry::total_for_week(&self.entries, crate::gui::time_entry::today())
    }

    pub(crate) fn focus_streak(&self) -> u32 {
        crate::gui::time_entry::focus_streak(&self.entries, crate::gui::time_entry::today())
    }

    pub(crate) fn weekly_daily_totals(&self) -> Vec<(chrono::NaiveDate, u64)> {
        daily_totals_for_week(&self.entries, crate::gui::time_entry::today())
    }

    pub(crate) fn report_project_totals(&self) -> Vec<(String, u64)> {
        let date = crate::gui::time_entry::today();
        match self.report_range {
            ReportRange::Week => crate::gui::time_entry::project_totals_for_week(&self.entries, date),
            ReportRange::Month => project_totals_for_month(&self.entries, date),
            ReportRange::Year => project_totals_for_year(&self.entries, date),
        }
    }

    fn record_current_session(&mut self) {
        let seconds = self.duration.as_secs().min(u64::from(u32::MAX)) as u32;
        if seconds == 0 || self.project_name.trim().is_empty() {
            return;
        }

        let project = self.project_name.trim().to_string();
        self.entries
            .entry(project.clone())
            .or_default()
            .push(TimeEntry { project, date: Local::now(), seconds });
    }
}

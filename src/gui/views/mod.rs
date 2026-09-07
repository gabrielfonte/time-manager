mod reports;
mod today;

use iced::widget::{button, column, container, rule, text};
use iced::{Alignment, Background, Border, Color, Element, Shadow};

use super::timer::{Message, ProjectTimer, Screen};

pub(crate) const INK: Color = Color::from_rgb(0.12, 0.15, 0.18);
pub(crate) const MUTED: Color = Color::from_rgb(0.38, 0.43, 0.46);
pub(crate) const PAPER: Color = Color::from_rgb(0.96, 0.95, 0.92);
pub(crate) const PANEL: Color = Color::from_rgb(1.0, 0.99, 0.97);
pub(crate) const TEAL: Color = Color::from_rgb(0.08, 0.48, 0.45);
pub(crate) const CORAL: Color = Color::from_rgb(0.88, 0.32, 0.25);

pub(crate) fn view(timer: &ProjectTimer) -> Element<'_, Message> {
    let sidebar = sidebar(timer.screen);
    let content = match timer.screen {
        Screen::Today => today::view(timer),
        Screen::Reports => reports::view(timer),
    };

    container(iced::widget::row![sidebar, content].height(iced::Fill))
        .style(|_| container::Style {
            background: Some(Background::Color(PAPER)),
            ..Default::default()
        })
        .into()
}

fn sidebar(screen: Screen) -> Element<'static, Message> {
    let today = nav_button("Today", screen == Screen::Today, Message::ScreenSelected(Screen::Today));
    let reports = nav_button("Reports", screen == Screen::Reports, Message::ScreenSelected(Screen::Reports));
    let brand = column![text("TIME").size(14).color(TEAL), text("keeper").size(27).color(INK)].spacing(0);

    container(column![brand, rule::horizontal(1), today, reports].spacing(16).padding(28))
        .width(220)
        .height(iced::Fill)
        .style(|_| panel_style())
        .into()
}

fn nav_button(label: &'static str, selected: bool, message: Message) -> Element<'static, Message> {
    button(text(label).size(15))
        .width(iced::Fill)
        .padding(12)
        .style(move |_, _| if selected {
            button::Style {
                background: Some(Background::Color(TEAL)),
                text_color: Color::WHITE,
                ..Default::default()
            }
        } else {
            button::Style {
                background: None,
                text_color: INK,
                ..Default::default()
            }
        })
        .on_press(message)
        .into()
}

pub(crate) fn panel_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(PANEL)),
        text_color: Some(INK),
        border: Border {
            color: Color::from_rgb(0.87, 0.87, 0.83),
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: Shadow {
            color: Color { a: 0.08, ..INK },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

pub(crate) fn metric_card(label: &'static str, value: String, note: &'static str, accent: Color) -> Element<'static, Message> {
    container(column![
        text(label).size(11).color(MUTED),
        text(value).size(25).color(INK),
        text(note).size(12).color(accent),
    ].spacing(7))
        .padding(20)
        .width(iced::Fill)
        .style(|_| panel_style())
        .into()
}

pub(crate) fn format_seconds(seconds: u64) -> String {
    format!("{}h {:02}m", seconds / 3600, (seconds % 3600) / 60)
}

pub(crate) fn format_duration(timer: &ProjectTimer) -> String {
    let seconds = timer.duration.as_secs();
    format!("{:02}:{:02}:{:02}", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
}

pub(crate) fn active(timer: &ProjectTimer) -> bool {
    matches!(timer.state, super::timer::State::Ticking { .. })
}

pub(crate) fn _alignment() -> Alignment {
    Alignment::Center
}

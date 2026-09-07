use iced::widget::{button, column, container, pick_list, row, rule, scrollable, text};
use iced::{Alignment, Background, Color, Element};

use super::{active, format_duration, format_seconds, metric_card, panel_style, CORAL, INK, MUTED, TEAL};
use crate::gui::timer::{Message, ProjectTimer};

pub(crate) fn view(timer: &ProjectTimer) -> Element<'_, Message> {
    let project_options = timer.project_names();

    let project_picker = pick_list(project_options, Some(timer.project_name.clone()), Message::ProjectSelected)
        .placeholder("Choose a project")
        .width(220);

    let project_input = iced::widget::text_input("Name this project", &timer.project_name)
        .on_input(Message::ProjectNameChanged)
        .padding(10)
        .width(220);

    let is_active = active(timer);
    let start_stop = if is_active {
        button(text("Stop session").size(15)).style(button::danger).on_press(Message::Stop)
    } else {
        button(text("Start session").size(15)).style(button::primary).on_press(Message::Start)
    };

    let timer_card = container(column![
        row![
            text("CURRENT SESSION").size(12).color(MUTED),
            text(if is_active { "TRACKING" } else { "READY" }).size(12).color(if is_active { CORAL } else { TEAL }),
        ].spacing(12).width(iced::Fill),
        text(format_duration(timer)).size(56).color(INK),
        row![project_input, project_picker, start_stop, button("Reset").on_press(Message::Reset)]
            .spacing(12)
            .align_y(Alignment::Center),
    ].spacing(20))
        .padding(28)
        .style(|_| panel_style());

    let header = column![
        text("Good morning.").size(34).color(INK),
        text("Make the next block of time count.").size(16).color(MUTED),
    ].spacing(6);

    let summary = row![
        metric_card("TODAY", format_seconds(timer.today_seconds()), "across recorded entries", TEAL),
        metric_card("THIS WEEK", format_seconds(timer.week_seconds()), "recorded this week", Color::from_rgb(0.32, 0.46, 0.72)),
        metric_card("FOCUS STREAK", format!("{} days", timer.focus_streak()), "consecutive days", Color::from_rgb(0.95, 0.61, 0.22)),
    ].spacing(14);

    container(scrollable(column![header, timer_card, summary, project_overview(timer)].spacing(22).padding(34)))
        .width(iced::Fill)
        .height(iced::Fill)
        .into()
}

pub(crate) fn project_overview(timer: &ProjectTimer) -> Element<'_, Message> {
    let projects = timer.today_projects();
    
    let rows: Vec<Element<'static, Message>> = projects.into_iter().enumerate().map(|(index, (name, seconds))| {
        let percentage = (seconds as f32 / 28_800.0).min(1.0);
        let color = [TEAL, Color::from_rgb(0.95, 0.61, 0.22), Color::from_rgb(0.32, 0.46, 0.72)][index % 3];
        let bar = container(rule::horizontal(8.0))
            .width(iced::Length::FillPortion((percentage * 100.0) as u16 + 1))
            .style(move |_| container::Style { background: Some(Background::Color(color)), ..Default::default() });
        let remainder = container(rule::horizontal(8.0))
            .width(iced::Length::FillPortion(100 - (percentage * 100.0) as u16))
            .style(|_| container::Style { background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.87))), ..Default::default() });
        row![
            text(name).width(130),
            row![bar, remainder].width(iced::Fill),
            text(format_seconds(seconds)).width(70),
        ].spacing(12).align_y(Alignment::Center).into()
    }).collect();

    container(column![
        row![text("PROJECT OVERVIEW").size(12).color(MUTED), text("Today").size(12).color(MUTED)]
            .spacing(12)
            .width(iced::Fill),
        column(rows).spacing(16),
    ].spacing(20))
        .padding(24)
        .style(|_| panel_style())
        .into()
}

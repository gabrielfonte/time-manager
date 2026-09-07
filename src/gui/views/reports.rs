use iced::widget::{column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Background, Color, Element};

use super::{format_seconds, panel_style, INK, MUTED};
use crate::gui::timer::{Message, ProjectTimer, ReportRange};

pub(crate) fn view(timer: &ProjectTimer) -> Element<'_, Message> {
    let daily_totals = timer.weekly_daily_totals();
    let max_seconds = daily_totals.iter().map(|(_, seconds)| *seconds).max().unwrap_or(0);

    let chart: Vec<Element<'_, Message>> = daily_totals.iter().map(|(date, seconds)| {
        let height = if max_seconds == 0 {
            4.0
        } else {
            24.0 + (*seconds as f32 / max_seconds as f32) * 96.0
        };
        let bar_color = if *seconds == 0 {
            Color::from_rgb(0.86, 0.87, 0.84)
        } else {
            super::TEAL
        };
        column![
            container(text("")).height(height).width(28).style(move |_| container::Style {
                background: Some(Background::Color(bar_color)),
                ..Default::default()
            }),
            text(date.format("%a").to_string()).size(12).color(MUTED),
        ].align_x(Alignment::Center).spacing(8).into()
    }).collect();

    let ranges = vec![ReportRange::Week, ReportRange::Month, ReportRange::Year];
    let range_picker = pick_list(ranges, Some(timer.report_range), Message::ReportRangeSelected)
        .width(130);
    let project_totals = timer.report_project_totals();
    let project_rows: Vec<Element<'static, Message>> = project_totals.into_iter().map(|(project, seconds)| {
        row![text(project).width(iced::Fill), text(format_seconds(seconds)).width(80)]
            .spacing(16)
            .into()
    }).collect();
    
    let range_label = match timer.report_range {
        ReportRange::Week => "This week",
        ReportRange::Month => "This month",
        ReportRange::Year => "This year",
    };

    let project_body: Element<'static, Message> = if project_rows.is_empty() {
        column![text("No recorded time for this period.").color(MUTED)].into()
    } else {
        column(project_rows).spacing(14).into()
    };

    let project_report = container(column![
        row![text("PROJECT TOTALS").size(12).color(MUTED), range_picker].spacing(12).width(iced::Fill),
        text(range_label).size(14).color(INK),
        project_body,
    ].spacing(16)).padding(28).style(|_| panel_style());

    container(scrollable(column![
        text("Reports").size(34).color(INK),
        text("A quieter view of where your week went.").size(16).color(MUTED),
        container(column![
            row![text("WEEKLY FOCUS").size(12).color(MUTED), text(format_seconds(timer.week_seconds())).size(25).color(INK)]
                .spacing(12)
                .width(iced::Fill),
            row(chart).align_y(Alignment::End).spacing(24).height(160),
        ].spacing(20)).padding(28).style(|_| panel_style()),
        project_report,
    ].spacing(22).padding(34)))
        .width(iced::Fill)
        .height(iced::Fill)
        .into()
}

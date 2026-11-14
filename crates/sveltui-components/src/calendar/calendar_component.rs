use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::prelude::*;

use super::calendar_types::*;
use crate::controllable::use_controllable;
use crate::date::{self, Date, Weekday};
use crate::inline_button::{Press, inline_button};

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub(crate) fn month_name(date: Date) -> &'static str {
    MONTHS[usize::try_from(date.month() - 1).unwrap_or(0)]
}

pub(crate) fn weekday_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}

/// "Friday, February 13, 2026": what assistive tech reads for a day.
pub(crate) fn long_date(date: Date) -> String {
    format!(
        "{}, {} {}, {}",
        weekday_name(date.weekday()),
        month_name(date),
        date.day(),
        date.year()
    )
}

/// Seven two-cell days with a cell between each, and a cell either side.
const WIDTH: u16 = 22;

/// A month of days to pick a date from. The grid is one tab stop: arrows
/// move a day or a week, Page Up/Down a month (with Shift, a year),
/// Home/End to the ends of the week, and Enter or Space picks.
#[component]
pub fn Calendar(props: CalendarProps) -> VNode {
    let own_id = use_hook(FocusId::new);
    let id = props.id.unwrap_or(own_id);
    let heading_id = use_hook(FocusId::new);
    let prev_id = use_hook(FocusId::new);
    let next_id = use_hook(FocusId::new);
    let focused = use_focus(id).is_focused();

    let value = use_controllable(
        props.value.bound(),
        props.value.value(),
        || props.default_value,
        props.on_value_change.clone(),
    );
    let cursor = state(None::<Date>);
    // Once focus leaves the calendar (the grid and its month buttons), the
    // cursor is forgotten, so coming back starts on the picked date again.
    let within = [id, prev_id, next_id]
        .into_iter()
        .any(|part| use_focus(part).is_focused());
    if !within && cursor.get().is_some() {
        cursor.set(None);
    }
    let (min, max) = (props.min, props.max);
    let today = date::today();
    // The cursor starts on the picked date, else today, within range.
    let at = date::clamp(cursor.get().or(value.get()).unwrap_or(today), min, max);
    let unavailable = {
        let check = props.is_date_unavailable.clone();
        move |day: Date| {
            min.is_some_and(|m| day < m)
                || max.is_some_and(|m| day > m)
                || check.as_ref().is_some_and(|check| check(day))
        }
    };
    let go = move |day: Date| cursor.set(Some(date::clamp(day, min, max)));
    let pick = {
        let value = value.clone();
        let unavailable = unavailable.clone();
        let on_select = props.on_select.clone();
        move |day: Date| {
            go(day);
            if unavailable(day) {
                return;
            }
            value.set(Some(day));
            if let Some(on_select) = &on_select {
                on_select(day);
            }
        }
    };

    let first = props.week_starts_on;
    let heading = format!("{} {}", month_name(at), at.year());
    let month_button = |button: FocusId, name: &str, glyph: &str, months: i64| {
        let press = Press {
            by_key: Rc::new(move |_| go(date::add_months(at, months))),
            by_click: Rc::new(move || go(date::add_months(at, months))),
        };
        inline_button(button, name.to_owned(), glyph.to_owned(), None, press)
    };
    let header = view! {
        <View flex_direction=FlexDirection::Row width=WIDTH>
            {month_button(prev_id, "Previous month", "‹", -1)}
            <Pressable focus_id=heading_id
                semantics={Semantics::new(Role::Heading, heading.clone()).live(Live::Polite).atomic(true)}>
                <View flex_grow=1.0 flex_direction=FlexDirection::Row
                    justify_content=JustifyContent::Center>
                    <TextSpan bold=true>{heading.clone()}</TextSpan>
                </View>
            </Pressable>
            {month_button(next_id, "Next month", "›", 1)}
        </View>
    };

    let chosen = value.get();
    let mut cells: Vec<VNode> = Vec::with_capacity(49);
    for (column, day) in date::weekdays(first).into_iter().enumerate() {
        let mut semantics = Semantics::new(Role::ColumnHeader, weekday_name(day));
        semantics.column = Some(column);
        let short: String = weekday_name(day).chars().take(2).collect();
        cells.push(
            view! {
                <Pressable semantics=semantics>
                    <TextSpan foreground_color=Color::DarkGrey>{short}</TextSpan>
                </Pressable>
            }
            .into_node(),
        );
    }
    for (row, week) in date::month_grid(at, first).into_iter().enumerate() {
        for (column, day) in week.into_iter().enumerate() {
            let active = focused && day == at;
            let picked = chosen == Some(day);
            let blocked = unavailable(day);
            let outside = !date::same_month(day, at);
            let (fg, bg) = if active {
                (Some(Color::Black), Some(Color::White))
            } else if picked {
                (Some(Color::White), Some(Color::DarkGrey))
            } else if outside || blocked {
                (Some(Color::DarkGrey), None)
            } else {
                (None, None)
            };
            let mut semantics = Semantics::new(Role::Cell, long_date(day))
                .selected(picked)
                .current(day == today);
            semantics.row = Some(row + 1);
            semantics.column = Some(column);
            let pick = pick.clone();
            cells.push(
                view! {
                    <Pressable semantics=semantics disabled=blocked>
                        // Clicks carry the grid's id, so they focus the grid.
                        <Pressable focus_id=id on_click={move || pick(day)}>
                            <View background_color=bg foreground_color=fg>
                                <TextSpan bold=picked underline={day == today}>
                                    {format!("{:>2}", day.day())}
                                </TextSpan>
                            </View>
                        </Pressable>
                    </Pressable>
                }
                .into_node(),
            );
        }
    }

    let on_key = move |key: KeyEvent, _: &mut EventContext| -> EventResult {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let next = match key.code {
            KeyCode::Left => date::add_days(at, -1),
            KeyCode::Right => date::add_days(at, 1),
            KeyCode::Up => date::add_days(at, -7),
            KeyCode::Down => date::add_days(at, 7),
            KeyCode::PageUp if shift => date::add_years(at, -1),
            KeyCode::PageDown if shift => date::add_years(at, 1),
            KeyCode::PageUp => date::add_months(at, -1),
            KeyCode::PageDown => date::add_months(at, 1),
            KeyCode::Home => date::start_of_week(at, first),
            KeyCode::End => date::end_of_week(at, first),
            KeyCode::Enter | KeyCode::Char(' ') => {
                pick(at);
                return EventResult::Handled;
            }
            _ => return EventResult::Ignored,
        };
        go(next);
        EventResult::Handled
    };

    let grid_semantics = Semantics::new(Role::Grid, props.label).labelled_by(heading_id);
    view! {
        <View width=WIDTH>
            {header}
            <Pressable focus_id=id tab_stop=true semantics=grid_semantics on_key=on_key>
                <View display=Display::Grid grid_template_columns={vec![length(2); 7]}
                    grid_auto_rows={length(1)} column_gap=1 padding={Padding::only(0, 1, 0, 1)}>
                    {cells}
                </View>
            </Pressable>
        </View>
    }
    .into_node()
}

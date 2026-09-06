//! Calendar, date and color pickers keep popup, focus and provisional selection native.
use super::popups::PopupAnchor;
use super::{ControlSize, primitives::Color};
use crate::native::{
    ControlledBinding, Deserialize, Event, EventDefinition, NativeChildren, NativeView, Serialize,
    TS, ViewCommand,
};
use chrono::{Datelike, NaiveDate, Weekday};
use gpui::{AppContext, Context, Entity, Focusable, IntoElement, Render, Subscription, Window};
use gpui_component::calendar::{CalendarEvent, CalendarState, Date, Matcher};
use gpui_component::{Disableable, Sizable};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct CivilDate(String);
impl<'de> Deserialize<'de> for CivilDate {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?;
        if s.len() != 10 || date.format("%Y-%m-%d").to_string() != s || date.year() < 1 {
            return Err(serde::de::Error::custom(
                "date must be YYYY-MM-DD, year 0001..9999",
            ));
        }
        Ok(Self(s))
    }
}
impl CivilDate {
    fn native(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.0, "%Y-%m-%d").expect("validated civil date")
    }
    fn from_native(v: NaiveDate) -> Self {
        Self(v.format("%Y-%m-%d").to_string())
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DateValue {
    Single {
        date: Option<CivilDate>,
    },
    Range {
        start: Option<CivilDate>,
        end: Option<CivilDate>,
    },
}
impl Default for DateValue {
    fn default() -> Self {
        Self::Single { date: None }
    }
}
impl DateValue {
    fn native(&self) -> Date {
        match self {
            Self::Single { date } => Date::Single(date.as_ref().map(CivilDate::native)),
            Self::Range { start, end } => Date::Range(
                start.as_ref().map(CivilDate::native),
                end.as_ref().map(CivilDate::native),
            ),
        }
    }
    fn from_native(v: Date) -> Self {
        match v {
            Date::Single(date) => Self::Single {
                date: date.map(CivilDate::from_native),
            },
            Date::Range(start, end) => Self::Range {
                start: start.map(CivilDate::from_native),
                end: end.map(CivilDate::from_native),
            },
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct YearRange {
    pub start: i32,
    pub end_exclusive: i32,
}
#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct DisabledDates {
    pub weekdays: Vec<u8>,
    pub before: Option<CivilDate>,
    pub after: Option<CivilDate>,
    pub dates: Vec<CivilDate>,
    pub ranges: Vec<DateInterval>,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct DateInterval {
    pub from: Option<CivilDate>,
    pub to: Option<CivilDate>,
}
impl DisabledDates {
    fn matcher(&self) -> Option<Rc<Matcher>> {
        if self == &Self::default() {
            return None;
        }
        let weekdays = self.weekdays.clone();
        let before = self.before.as_ref().map(CivilDate::native);
        let after = self.after.as_ref().map(CivilDate::native);
        let dates: std::collections::HashSet<_> =
            self.dates.iter().map(CivilDate::native).collect();
        let ranges: Vec<_> = self
            .ranges
            .iter()
            .map(|r| {
                (
                    r.from.as_ref().map(CivilDate::native),
                    r.to.as_ref().map(CivilDate::native),
                )
            })
            .collect();
        Some(Rc::new(Matcher::custom(move |date| {
            weekdays.contains(&(date.weekday().num_days_from_sunday() as u8))
                || before.is_some_and(|v| *date < v)
                || after.is_some_and(|v| *date > v)
                || dates.contains(date)
                || ranges
                    .iter()
                    .any(|(a, b)| a.is_none_or(|a| *date >= a) && b.is_none_or(|b| *date <= b))
        })))
    }
}
macro_rules! date_props {
    ($name:ident { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        #[crate::native_type]
        #[derive(Clone, Debug)]
        #[serde(default, rename_all = "camelCase")]
        pub struct $name { pub value: Option<DateValue>, pub default_value: DateValue, pub ack_edit_seq: u32, pub number_of_months: usize, pub first_day_of_week: u8, pub year_range: Option<YearRange>, pub disabled_dates: DisabledDates, $($field:$ty,)* }
        impl Default for $name { fn default() -> Self { Self { value: None, default_value: DateValue::default(), ack_edit_seq: 0, number_of_months: 1, first_day_of_week: 0, year_range: None, disabled_dates: DisabledDates::default(), $($field:$default,)* } } }
    }
}
date_props!(CalendarProps {});
date_props!(DatePickerProps { placeholder: String = String::new(), cleanable: bool = false, format: String = "%Y/%m/%d".into(), disabled: bool = false, appearance: bool = true, size: ControlSize = ControlSize::Medium, presets: Vec<DatePreset> = Vec::new() });
#[crate::native_type]
#[derive(Clone, Debug)]
pub struct DatePreset {
    pub label: String,
    pub value: DateValue,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DateChange {
    pub value: DateValue,
    pub edit_seq: u32,
}
fn weekday(value: u8) -> Weekday {
    [
        Weekday::Sun,
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
    ][value as usize]
}
fn validate_date(value: &DateValue, rules: &DisabledDates) -> Result<(), String> {
    let date = value.native();
    if let Date::Range(start, end) = date
        && (start.is_none() && end.is_some() || start.zip(end).is_some_and(|(a, b)| a > b))
    {
        return Err("date range must start before it ends".into());
    }
    if rules.matcher().is_some_and(|m| m.is_match(&date)) {
        return Err("selected date is disabled".into());
    }
    Ok(())
}
fn validate_calendar(
    months: usize,
    first: u8,
    years: &Option<YearRange>,
    rules: &DisabledDates,
    value: &DateValue,
) -> Result<(), String> {
    if !(1..=12).contains(&months)
        || first > 6
        || years
            .as_ref()
            .is_some_and(|r| r.start < 1 || r.end_exclusive > 10000 || r.end_exclusive <= r.start)
    {
        return Err("calendar requires 1..12 months, firstDayOfWeek 0..6, and ordered years within 1..10000".into());
    }
    if rules.weekdays.iter().any(|v| *v > 6)
        || rules.dates.len() > 10000
        || rules.ranges.len() > 1000
        || rules.ranges.iter().any(|r| {
            r.from
                .as_ref()
                .zip(r.to.as_ref())
                .is_some_and(|(a, b)| a.native() > b.native())
        })
    {
        return Err("invalid or excessive disabled-date rules".into());
    }
    validate_date(value, rules)
}
fn binding() -> Option<ControlledBinding> {
    Some(ControlledBinding {
        value_prop: "value",
        event_id: 1,
        sequence_field: "editSeq",
        ack_prop: "ackEditSeq",
    })
}
pub struct Calendar {
    state: Entity<CalendarState>,
    props: CalendarProps,
    event: Event<DateChange>,
    edit_seq: u32,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for Calendar {
    type Props = CalendarProps;
    type Event = DateChange;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        validate_calendar(
            p.number_of_months,
            p.first_day_of_week,
            &p.year_range,
            &p.disabled_dates,
            p.value.as_ref().unwrap_or(&p.default_value),
        )
    }
    fn mount(
        props: Self::Props,
        event: Event<DateChange>,
        _: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut s = CalendarState::new(window, cx);
            s.set_disabled_matcher_shared(props.disabled_dates.matcher());
            s.set_date(
                props
                    .value
                    .as_ref()
                    .unwrap_or(&props.default_value)
                    .native(),
                window,
                cx,
            );
            if let Some(r) = &props.year_range {
                s.set_year_range((r.start, r.end_exclusive), cx);
            }
            s
        });
        let subscription = cx.subscribe(&state, |this, _, event, cx| {
            match event {
                CalendarEvent::Change(date) => {
                    this.edit_seq = this
                        .edit_seq
                        .checked_add(1)
                        .expect("calendar edit sequence exhausted");
                    this.event.emit(DateChange {
                        value: DateValue::from_native(*date),
                        edit_seq: this.edit_seq,
                    });
                }
                CalendarEvent::Selected(date) => this
                    .event
                    .related("complete")
                    .emit(DateValue::from_native(*date)),
            }
            cx.notify();
        });
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let entering = self.props.value.is_none() && p.value.is_some();
        self.state.update(cx, |s, cx| {
            if p.disabled_dates != self.props.disabled_dates {
                s.set_disabled_matcher_shared(p.disabled_dates.matcher());
            }
            if p.year_range != self.props.year_range {
                let r = p.year_range.clone().unwrap_or_else(|| YearRange {
                    start: s.today().year() - 50,
                    end_exclusive: s.today().year() + 50,
                });
                s.set_year_range((r.start, r.end_exclusive), cx);
            }
            if let Some(value) = &p.value
                && (entering || p.ack_edit_seq >= self.edit_seq)
                && s.date() != value.native()
            {
                s.set_date(value.native(), window, cx);
            }
        });
        self.props = p;
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        binding()
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![EventDefinition::new::<DateValue>("complete")]
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![ViewCommand::new(
            "showDate",
            |this, date: CivilDate, window, cx| {
                this.state.update(cx, |s, cx| {
                    s.show_date(date.native(), cx);
                    s.set_number_of_months(this.props.number_of_months, window, cx);
                });
                Ok(())
            },
        )]
    }
}
impl Render for Calendar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_component::calendar::Calendar::new(&self.state)
            .number_of_months(self.props.number_of_months)
            .first_day_of_week(weekday(self.props.first_day_of_week))
    }
}
pub struct DatePicker {
    state: Entity<gpui_component::date_picker::DatePickerState>,
    props: DatePickerProps,
    event: Event<DateChange>,
    edit_seq: u32,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for DatePicker {
    type Props = DatePickerProps;
    type Event = DateChange;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        validate_calendar(
            p.number_of_months,
            p.first_day_of_week,
            &p.year_range,
            &p.disabled_dates,
            p.value.as_ref().unwrap_or(&p.default_value),
        )?;
        if p.format.len() > 256
            || chrono::format::StrftimeItems::new(&p.format)
                .any(|i| matches!(i, chrono::format::Item::Error))
            || p.presets.len() > 100
        {
            return Err("invalid date format or too many presets".into());
        }
        for preset in &p.presets {
            validate_date(&preset.value, &p.disabled_dates)?;
            if !preset.value.native().is_complete() {
                return Err("date presets must be complete".into());
            }
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<DateChange>,
        _: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut s = gpui_component::date_picker::DatePickerState::new(window, cx)
                .date_format(props.format.clone())
                .first_day_of_week(weekday(props.first_day_of_week));
            s.set_disabled_matcher_shared(props.disabled_dates.matcher(), cx);
            s.set_date(
                props
                    .value
                    .as_ref()
                    .unwrap_or(&props.default_value)
                    .native(),
                window,
                cx,
            );
            if let Some(r) = &props.year_range {
                s.set_year_range((r.start, r.end_exclusive), cx);
            }
            s
        });
        let subscription = cx.subscribe(&state, |this, _, event, _| {
            let gpui_component::date_picker::DatePickerEvent::Change(date) = event;
            this.edit_seq = this
                .edit_seq
                .checked_add(1)
                .expect("date picker edit sequence exhausted");
            this.event.emit(DateChange {
                value: DateValue::from_native(*date),
                edit_seq: this.edit_seq,
            });
        });
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let entering = self.props.value.is_none() && p.value.is_some();
        self.state.update(cx, |s, cx| {
            if p.format != self.props.format {
                s.set_date_format(p.format.clone(), cx);
            }
            if p.first_day_of_week != self.props.first_day_of_week {
                s.set_first_day_of_week(weekday(p.first_day_of_week), cx);
            }
            if p.disabled_dates != self.props.disabled_dates {
                s.set_disabled_matcher_shared(p.disabled_dates.matcher(), cx);
            }
            if p.year_range != self.props.year_range {
                let year = chrono::Local::now().year();
                let r = p.year_range.clone().unwrap_or(YearRange {
                    start: year - 50,
                    end_exclusive: year + 50,
                });
                s.set_year_range((r.start, r.end_exclusive), cx);
            }
            if p.disabled && !self.props.disabled {
                s.set_open(false, cx);
            }
            if let Some(value) = &p.value
                && (entering || p.ack_edit_seq >= self.edit_seq)
                && s.date() != value.native()
            {
                s.set_date(value.native(), window, cx);
            }
        });
        self.props = p;
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        binding()
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("setOpen", |this, open: bool, _, cx| {
                if open && this.props.disabled {
                    return Err("disabled date picker cannot be opened".into());
                }
                this.state.update(cx, |s, cx| s.set_open(open, cx));
                Ok(())
            }),
            ViewCommand::new("focus", |this, (): (), window, cx| {
                if this.props.disabled {
                    return Err("disabled date picker cannot be focused".into());
                }
                this.state.read(cx).focus_handle(cx).focus(window, cx);
                Ok(())
            }),
        ]
    }
}
impl Render for DatePicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let presets = self
            .props
            .presets
            .iter()
            .map(|p| match p.value.native() {
                Date::Single(Some(date)) => {
                    gpui_component::date_picker::DateRangePreset::single(p.label.clone(), date)
                }
                Date::Range(Some(start), Some(end)) => {
                    gpui_component::date_picker::DateRangePreset::range(p.label.clone(), start, end)
                }
                _ => unreachable!("validated complete preset"),
            })
            .collect();
        gpui_component::date_picker::DatePicker::new(&self.state)
            .placeholder(self.props.placeholder.clone())
            .cleanable(self.props.cleanable)
            .appearance(self.props.appearance)
            .disabled(self.props.disabled)
            .with_size(self.props.size)
            .number_of_months(self.props.number_of_months)
            .presets(presets)
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ColorPickerProps {
    pub value: Option<Color>,
    pub default_value: Option<Color>,
    pub ack_edit_seq: u32,
    pub label: Option<String>,
    pub accessibility_label: Option<String>,
    pub featured_colors: Option<Vec<Color>>,
    pub anchor: PopupAnchor,
    pub size: ControlSize,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ColorChange {
    pub value: Option<Color>,
    pub edit_seq: u32,
}
pub struct ColorPicker {
    state: Entity<gpui_component::color_picker::ColorPickerState>,
    props: ColorPickerProps,
    event: Event<ColorChange>,
    edit_seq: u32,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for ColorPicker {
    type Props = ColorPickerProps;
    type Event = ColorChange;
    fn validate_props(p: &Self::Props) -> Result<(), String> {
        if p.featured_colors.as_ref().is_some_and(|c| c.len() > 256) {
            return Err("featuredColors is limited to 256 colors".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<ColorChange>,
        _: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut s = gpui_component::color_picker::ColorPickerState::new(window, cx);
            if let Some(color) = props.value.as_ref().or(props.default_value.as_ref()) {
                s.set_value(color.native(), window, cx);
            }
            s
        });
        let subscription = cx.subscribe(&state, |this, _, event, _| {
            let gpui_component::color_picker::ColorPickerEvent::Change(color) = event;
            this.edit_seq = this
                .edit_seq
                .checked_add(1)
                .expect("color edit sequence exhausted");
            this.event.emit(ColorChange {
                value: color.map(Color::from_native),
                edit_seq: this.edit_seq,
            });
        });
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        if (p.value != self.props.value || p.ack_edit_seq != self.props.ack_edit_seq)
            && (self.props.value.is_none() && p.value.is_some() || p.ack_edit_seq >= self.edit_seq)
        {
            self.state.update(cx, |s, cx| {
                let color = p.value.as_ref().map(Color::native);
                if color != s.value() {
                    match color {
                        Some(c) => s.set_value(c, window, cx),
                        None => s.clear_value(window, cx),
                    }
                }
            });
        }
        self.props = p;
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        binding()
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![ViewCommand::new("setOpen", |this, open: bool, _, cx| {
            this.state.update(cx, |s, cx| s.set_open(open, cx));
            Ok(())
        })]
    }
}
impl Render for ColorPicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut v = gpui_component::color_picker::ColorPicker::new(&self.state)
            .anchor(self.props.anchor.into())
            .with_size(self.props.size);
        if let Some(label) = &self.props.label {
            v = v.label(label.clone());
        }
        if let Some(label) = &self.props.accessibility_label {
            v = v.accessibility_label(label.clone());
        }
        if let Some(colors) = &self.props.featured_colors {
            v = v.featured_colors(colors.iter().map(Color::native).collect());
        }
        v
    }
}
pub(crate) fn definitions() -> Vec<crate::native::ComponentDefinition> {
    vec![
        __native_component_Calendar(),
        __native_component_DatePicker(),
        __native_component_ColorPicker(),
    ]
}

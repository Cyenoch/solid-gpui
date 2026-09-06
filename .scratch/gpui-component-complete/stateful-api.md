# Stateful gpui-component adapters: pinned source and implementation contract

Source baseline: `928c3eb776a3d733d9b771f7dea27a6a79242ced`, read from `/Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb`. This is source research, not implemented or compiled adapter code. `C/` below means `crates/component/src/`; `B/` means `crates/base/src/` at that exact baseline. Many public state types in C are re-exports of B types. The snippets below use the real public API at this revision; DTO constructors/conversions such as `into_native`/`TableDto::sort` are local adapter functions to implement from the displayed DTO shapes, not upstream symbols. Imports, identical `NativeView` glue and conversion boilerplate are omitted. Explicitly named **native additions** do not exist yet and must not be reported as supported.

## 1. Shared bridge: retained native entity, data-only DTOs

The real bridge files are [native/component.rs](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:97) and [components/input.rs](/Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/input.rs:1). `crates/solid-gpui/src/native/input.rs` does not exist. `NativeView::mount(props, Event<E>, window, cx)` creates one entity per mounted Host Node; `update` mutates it; `ViewCommand::new(name, fn)` exposes typed asynchronous commands. `ComponentDefinition::view` accepts **no JS children** and currently declares one named event per retained view. Use a typed event union or extend this seam for named event channels/slots; do not pretend JS render closures can execute inside `render_item`. [bridge]

The immutable DTO bytes cross JS/runtime threads. GPUI `Entity`, `Window`, `App`, subscriptions, delegates, render closures, `Rc`, `RefCell`, scroll/focus handles and IME state remain on the foreground GPUI thread. JS sends item data, stable keys, controlled values and command arguments. Native emits keys plus the data revision that resolved the index. A deferred callback must capture the key from the model generation producing the callback, rather than resolve the old index against newly supplied items. Background work receives owned immutable data and a generation; apply results through `WeakEntity::update`/`cx.update` only if owner, epoch and generation still match. [domain][bridge][command-state]

Common owner shape (component-specific mount/render bodies follow):

```rust,ignore
struct Adapter<S: 'static, P, E> {
    state: Entity<S>,
    props: P,
    events: Event<E>,
    subscriptions: Vec<Subscription>, // owner drops these on unmount
    tasks: Vec<Task<()>>,             // retained cancellation, never detached global work
}
// In NativeView::mount:
let state = cx.new(|cx| /* real state constructor listed below */);
let subscription = cx.subscribe_in(&state, window, |this, state, event, window, cx| {
    // Read typed state/event here, convert into owned DTO, this.events.emit(dto).
});
// In update: compare old/new fields, call mutable setters only for changed fields.
// In Render: build the styled C component around &self.state, never cx.new here.
```

`NativeView` currently accepts already decoded props and cannot return update errors. Validate finite numbers, unique keys, bounded collection sizes, date formats and semantic combinations before publishing a commit, by a validated DTO deserializer or an explicit validation seam. Do not let native assertions/panics become the JS validation mechanism. [`Event::emit` encodes DTOs into the one typed bytes field; no listener means no encoding.][bridge]

## 2. Input, Textarea, Editor, NumberInput

### Shared text engine and controlled ownership

Public state types are distinct aliases: `InputState = InputBaseState<InputMode>`, `TextareaState = InputBaseState<TextareaMode>`, `EditorState = InputBaseState<EditorMode>`. Use their distinct constructors; the old `.multi_line()` / `.code_editor()` design is gone. They share `InputEvent::{Change, PressEnter { secondary, shift }, Focus, Blur}`. `Change` carries no text: read `state.value()` inside the subscription. [input-kinds][input-state]

Reusable DTO shape, with mode-specific structs rather than irrelevant options on every input:

```ts
interface TextValueProps {
  value?: string; defaultValue?: string; ackEditSeq: number;
  placeholder?: string; disabled?: boolean; readonly?: boolean;
  appearance?: boolean; bordered?: boolean; ariaLabel?: string;
}
interface TextChange { value: string; editSeq: number }
interface TextSelection { startByte: number; endByte: number }
interface TextPosition { line: number; character: number } // 0-based Unicode scalar columns, not JS UTF-16
interface ScrollOffset { x: number; y: number }
// Input adds masked, cleanable, maskToggle, contentType, pattern/mask DTO.
// Textarea adds rows or autoGrow:{minRows,maxRows}, softWrap, searchable.
// Editor adds language, softWrap, lineNumber, folding, indentGuides,
// tabSize:{tabSize,hardTabs}, showWhitespaces, cursorSurroundingLines,
// scrollBeyondLastLine, diagnostics and explicit decoration layers.
// NumberInput adds step/min/max while retaining a STRING value.
```

Keep NumberInput `value` a string: empty input, `-`, decimal editing and mask formatting cannot be represented by `number`. An optional parsed finite numeric value can accompany `Change`, but never replace text ownership. `InputBaseState` actually defaults `number_step` to `Some(Fixed(1.))`; some method comments still describe an older event-only default. [input-defaults][number]

The existing input adapter contains a necessary protocol, not incidental retry scaffolding: normalize only single-line `\r/\n`, preserve selection/undo on equal value echoes, ignore stale `ackEditSeq`, defer an acknowledged replacement while marked text exists, detect unmark-without-Change, and cancel pending work on a newer native edit/unmount. `replaceValue` explicitly calls `unmark_text` then `replace_all` and is different from an ordinary controlled echo. Refactor this reconciliation into a mode-generic native owner and reuse it for Input/Textarea/Editor/NumberInput; do not implement these as `set_value(props.value)` on every update. Multiline values must not strip newlines. [existing-input]

Exact common operations:

- Constructor: `InputState::new(window, cx).default_value(initial).placeholder(text)`; corresponding `TextareaState::new` and `EditorState::new` have the same constructor arguments.
- Runtime: `set_value(value, window, cx)` is a programmatic replacement that does not emit `Change`; `replace_all(value, window, cx)` records a user-like replacement; `insert(text, window, cx)`, `replace(text, window, cx)` replaces the current native selection. For an explicit byte range, validate the range, call `set_selected_range(range,cx)`, then `replace(text,window,cx)`; `replace` has no range argument. Check the exact position/range units: `selected_range`/`set_selected_range(Range<usize>, cx)` use native byte offsets; `EntityInputHandler` replacement ranges use UTF-16. Do not directly pass JS string indices as native byte offsets. Despite using `lsp_types::Position`, the current RopeExt `position_to_offset`/`offset_to_position` count Unicode scalar characters within the line, not UTF-16 code units. Use an explicitly named scalar-column DTO or convert against the current text. [input-rope] [input-state]
- `focus(window,cx)`, `set_placeholder(text,window,cx)`, `set_disabled(bool,cx)`, `set_readonly(bool,cx)`, `set_submit_on_enter(bool,cx)`, `set_text_align(TextAlign,cx)`, `set_context_menu_enabled(bool)`.
- Cursor/scroll commands: `cursor_position() -> Position`, `set_cursor_position(position,window,cx)`, `selected_range()`, `set_selected_range(range,cx)`, `scroll_offset()`, `set_scroll_offset(point,cx)`, `visible_row_range()`; validate offsets on the current value and tag asynchronous requests by edit sequence. [input-state]

Minimal native construction/render bodies:

```rust,ignore
// Input
let state = cx.new(|cx| InputState::new(window, cx)
    .default_value(initial).placeholder(placeholder).masked(masked));
// update changed presentation/policy fields, plus shared controlled reconciliation:
state.update(cx, |s, cx| {
    s.set_masked(masked, window, cx);
    s.set_placeholder(placeholder, window, cx);
});
let mut input = GpuiInput::new(&self.state).disabled(p.disabled).readonly(p.readonly)
    .appearance(p.appearance).bordered(p.bordered).cleanable(p.cleanable);
if p.mask_toggle { input = input.mask_toggle(); } // this builder takes no bool
input

// Textarea
let state = cx.new(|cx| TextareaState::new(window, cx)
    .default_value(initial).placeholder(placeholder).rows(rows)
    .soft_wrap(soft_wrap).searchable(searchable));
state.update(cx, |s, cx| {
    s.set_rows(rows, cx); // fixed rows mode, when that prop changes
    // auto-grow alternative: s.set_auto_grow(min_rows, max_rows, cx)
    s.set_soft_wrap(soft_wrap, window, cx);
    s.set_searchable(searchable, cx);
});
Textarea::new(&self.state).disabled(p.disabled).readonly(p.readonly)
    .appearance(p.appearance).bordered(p.bordered)

// Editor
let state = cx.new(|cx| EditorState::new(window, cx)
    .default_value(initial).language(language).soft_wrap(soft_wrap)
    .line_number(line_numbers).folding(folding).indent_guides(indent_guides));
state.update(cx, |s, cx| {
    s.set_highlighter(language, cx); // actual runtime language setter
    s.set_soft_wrap(soft_wrap, window, cx);
    s.set_line_number(line_numbers, window, cx);
    s.set_folding(folding, window, cx);
    s.set_indent_guides(indent_guides, window, cx);
    s.set_tab_size(TabSize { tab_size, hard_tabs }, cx);
});
Editor::new(&self.state).disabled(p.disabled).readonly(p.readonly)
    .appearance(p.appearance).bordered(p.bordered)

// NumberInput
let state = cx.new(|cx| InputState::new(window, cx)
    .default_value(initial).step(step));
state.update(cx, |s, cx| {
    s.set_step(Some(NumberStep::Fixed(step)), window, cx);
    s.set_min(min, window, cx);
    s.set_max(max, window, cx);
});
NumberInput::new(&self.state).disabled(p.disabled).appearance(p.appearance)
```

Input presentation supports `prefix`, `suffix`, `h`, `appearance`, `cleanable`, `mask_toggle`, `disabled`, `readonly`, `bordered`, tab/role/aria/content-type settings. Textarea/Editor presentation specifically supports `h`, appearance/bordered/disabled/readonly, tab/role/aria, context menu. NumberInput supports placeholder, prefix, suffix, appearance and Disableable/Sizable/FocusableExt. Retained view currently cannot accept JSX slot children: typed text/icon adornment DTOs are directly implementable; arbitrary JSX adornments require a native child-slot bridge. [input-widget][multiline-widgets][number-widget][bridge]

Textarea defaults ordinary text layout to 2 rows, soft-wrap true, search false. `set_rows` in AutoGrow mode changes its current/max rows but does not switch layout mode back to PlainText; a reactive autoGrow on→off prop needs an explicit native layout setter. Editor defaults search/line numbers/folding/indent guides true, 2-space soft tabs; all actual defaults must be explicitly mirrored in serde defaults instead of using `bool::default()` for everything. [input-modes]

Editor diagnostics DTO: `{editSeq, entries:[{range:{start:{line,character},end:{line,character}}, message, severity:"error"|"warning"|"information"|"hint",code?,source?}]}`. Apply only to the matching text generation. Read `text().clone()` before borrowing `diagnostics_mut()`, then `set.reset(&text)`, `set.extend(Diagnostic::new(range,message).with_severity(...))`, `cx.notify()`. Diagnostics are an `Option`; reject/report the unavailable capability rather than silently discard it. Decoration byte ranges use `TextDecoration::new(range, HighlightStyle)` and a retained `TextDecorationCollection` created via `create_decorations_collection(Vec<TextDecoration>,cx)`; update `collection.set(vec,cx)`. [editor-extra]

Native synchronous validation (`validate`, `set_validator`) and `NumberStep::ByValue` are Rust closures called inside input handling. A JS predicate cannot synchronously cross the process/thread boundary. Define a finite serializable rule grammar (regex/range/step bands) and compile Rust closures, or use explicit asynchronous validation events after editing. Editor `lsp()/lsp_mut()` expose native provider state, not a JavaScript LSP implementation; full completion/hover/actions require a separate request/result provider bridge with edit-generation cancellation. A plain Editor wrapper does not make those provider callbacks callable from JS. [input-modes][number][input-kinds]

## 3. OtpInput

`OtpState::new(length,window,cx)` owns value and focus; default value empty, masked false. `OtpEvent` is Change, Complete, Focus, Blur. Read `.value()` in subscription; Complete follows Change when keyboard entry reaches length. `set_value` and `default_value` deliberately do not validate digits or length, while keyboard input is numeric. `set_masked(bool,window,cx)` and `focus(window,cx)` are public; length has no setter. Styled `OtpInput::new(&state)` defaults groups=2, size Medium, focus ring true, disabled false. [otp]

```ts
interface OtpProps { length: number; value?: string; defaultValue?: string;
  masked?: boolean; groups?: number; disabled?: boolean }
type OtpEvent = {kind:"change"|"complete"|"focus"|"blur"; value:string};
```

```rust,ignore
let state = cx.new(|cx| OtpState::new(p.length, window, cx)
    .default_value(initial).masked(p.masked));
let sub = cx.subscribe_in(&state, window, |this, state, ev: &OtpEvent, _, cx| {
    this.events.emit(OtpDto { kind: (*ev).into(), value: state.read(cx).value().to_string() });
});
// Only on changed external value / masked flag:
state.update(cx, |s, cx| { s.set_value(value, window, cx); s.set_masked(masked, window, cx); });
OtpInput::new(&self.state).groups(p.groups).disabled(p.disabled)
```

For a numeric JS contract validate `length>=1`, bounded length, ASCII digit value of at most length before native publication. This is an explicit JS contract restriction, not upstream behavior. For fully reactive `length`, add a native `set_length` preserving the same focus/subscriptions and defining truncation; do not replace the entity on every length/value update. Clipboard paste and IME are not implemented by this OTP state: it handles key-down digits/backspace, so do not claim full text-input behavior. [otp]

## 4. Slider

`SliderState::new()` defaults min=0,max=100,step=1,value=`Single(0)`,scale=Linear. `SliderValue` is `Single(f32)` or `Range(f32,f32)`. `SliderEvent::Change(value)` is continuous and `Release(value)` is the completed press/drag. `set_value(value,window,cx)` emits no Change. Styled `Slider::new(&state)` supports horizontal/vertical, reverse, disabled and Styled. [slider]

```ts
type SliderValue = {kind:"single";value:number}|{kind:"range";start:number;end:number};
interface SliderProps { value:SliderValue; min?:number;max?:number;step?:number;
  scale?:"linear"|"logarithmic";vertical?:boolean;reverse?:boolean;disabled?:boolean }
type SliderEvent = {kind:"change"|"release";value:SliderValue};
```

```rust,ignore
let state = cx.new(|_| SliderState::new().min(p.min).max(p.max)
    .step(p.step).scale(p.scale.into()).default_value(p.value.into_native()));
let sub = cx.subscribe_in(&state, window, |this, _, ev: &SliderEvent, _, _| {
    this.events.emit(match ev {
        SliderEvent::Change(v) => SliderDto::change(*v),
        SliderEvent::Release(v) => SliderDto::release(*v),
    });
});
if value_changed { state.update(cx, |s,cx| s.set_value(value,window,cx)); }
let mut slider = Slider::new(&self.state).disabled(p.disabled);
if p.vertical { slider = slider.vertical(); }
if p.reverse { slider = slider.reverse(); }
slider
```

Validate finite min/max/value, min<max, step>0, ordered range, logarithmic min>0. `set_value` clamps thumb position but retains the original raw value; clamp/reject according to the declared DTO contract before setter use. **Native addition:** mutable min/max/step/scale configuration setters (ideally one atomic configuration setter) do not exist. Replacing `SliderState` inside the same Entity also resets private dragging/bounds; that is not a preservation solution. The adapter can expose constructor-only defaults honestly, or add setters before promising reactive range configuration. [slider]

## 5. Calendar and DatePicker

Use a tagged civil-date DTO, never UTC timestamps:

```ts
type DateValue = {kind:"single";date:string|null} |
  {kind:"range";start:string|null;end:string|null}; // ISO YYYY-MM-DD, chrono NaiveDate
interface DisabledDates { weekdays?:number[]; before?:string;after?:string;
  ranges?:{from:string|null;to:string|null}[];dates?:string[] }
interface CalendarProps { value:DateValue;numberOfMonths?:number;
  firstDayOfWeek?:0|1|2|3|4|5|6;yearRange?:{start:number;endExclusive:number};
  disabledDates?:DisabledDates }
interface DatePickerProps extends CalendarProps { placeholder?:string;cleanable?:boolean;
  format?:string;disabled?:boolean;appearance?:boolean;
  presets?:{label:string;value:DateValue}[] }
```

`Date::{Single(Option<NaiveDate>), Range(Option<NaiveDate>,Option<NaiveDate>)}` is the native value. `CalendarState::new(window,cx)` starts today-visible, empty single selection, one month, years `[today.year-50,today.year+50)`. `CalendarEvent::Selected(Date)` fires only once selection is complete; the first click of a range changes native state and notifies but emits no Selected. If JS requires every provisional range selection, observe the CalendarState and compare `.date()`; do not mislabel Selected as a per-click Change. [calendar]

```rust,ignore
let state = cx.new(|cx| {
    let mut s = CalendarState::new(window,cx);
    s.set_date(initial,window,cx);
    s.set_year_range((start_year,end_exclusive),cx);
    s
});
let sub = cx.subscribe_in(&state,window,|this,_,ev:&CalendarEvent,_,_| {
    let CalendarEvent::Selected(date) = ev;
    this.events.emit(DateDto::from(*date));
});
state.update(cx,|s,cx| {
    if date_changed { s.set_date(date,window,cx); }
    s.set_disabled_matcher_shared(matcher); // Option<Rc<Matcher>>, supports removing rule
    if years_changed { s.set_year_range((start,end_exclusive),cx); }
});
Calendar::new(&self.state).number_of_months(p.number_of_months)
    .first_day_of_week(p.first_day_of_week.into_chrono())
```

Matcher is `DayOfWeek(Vec<u32>)` with Sunday=0; `Matcher::interval(before,after)` disables dates strictly **before before OR after after**; `Matcher::range(from,to)` matches inclusive bounds; `Matcher::custom` takes `Fn(&NaiveDate)->bool + Send+Sync`. For multiple rules, compile one Rust closure over immutable parsed date sets/ranges using OR semantics. `Matcher::is_match(Date::Range)` only tests the two endpoints, not every day inside. Decide/document whether ranges spanning disabled interior days are allowed. [calendar]

```rust,ignore
let state = cx.new(|cx| {
    let mut s = (if initial.is_single() { DatePickerState::new(window,cx) }
                 else { DatePickerState::range(window,cx) })
        .date_format(format).first_day_of_week(first_day);
    s.set_date(initial,window,cx);
    s.set_year_range((start,end_exclusive),cx);
    s
});
let sub = cx.subscribe_in(&state,window,|this,_,ev:&DatePickerEvent,_,_| {
    let DatePickerEvent::Change(date) = ev;
    this.events.emit(DateDto::from(*date));
});
if date_changed { state.update(cx,|s,cx| s.set_date(date,window,cx)); }
DatePicker::new(&self.state).placeholder(p.placeholder.clone())
    .cleanable(p.cleanable).disabled(p.disabled).appearance(p.appearance)
    .number_of_months(p.number_of_months).presets(native_presets)
```

DatePicker defaults format `%Y/%m/%d`, first day Sunday, one month, cleanable false, appearance/focus ring true, disabled false. `set_date` unconditionally closes its popup and synchronizes its inner calendar; call only for an actual external value change, not every prop render/ack. Its private calendar may reject disabled `Date` while picker has already assigned that Date: validate value against rules before calling. Format, firstDayOfWeek and disabledMatcher only have consuming builders; open is private with no public setter. **Native additions:** mutable format/first-day/optional matcher/open setters are needed for their reactive/command API. Number of months is already available on presentation; year range already has a state setter. [date-picker]

## 6. ColorPicker

State lives in `B/color_picker.rs`, re-exported from C. Empty/closed, preview None, tab 0 are defaults. `ColorPickerEvent::Change(Option<Hsla>)` represents committed color only. Hover is a preview and no Change; sliders commit without closing; palette and valid hex Enter commit and close. Four native sliders and the hex InputState are owned by ColorPickerState. [color]

```ts
interface Hsla { h:number;s:number;l:number;a:number } // all normalized 0..1
interface ColorPickerProps { value:Hsla|null;label?:string;accessibilityLabel?:string;
  featuredColors?:Hsla[];anchor?:"topLeft"|"topRight"|"bottomLeft"|"bottomRight" }
type ColorPickerEvent = {kind:"change";value:Hsla|null};
```

```rust,ignore
let state = cx.new(|cx| ColorPickerState::new(window,cx));
state.update(cx,|s,cx| match initial {
    Some(color) => s.set_value(color,window,cx), None => s.clear_value(window,cx),
});
let sub = cx.subscribe_in(&state,window,|this,_,ev:&ColorPickerEvent,_,_| {
    let ColorPickerEvent::Change(value) = ev;
    this.events.emit(ColorDto::from(*value));
});
// Changed controlled value uses the same match. No events are emitted by these setters.
ColorPicker::new(&self.state).label(label).accessibility_label(aria_label)
    .featured_colors(colors).anchor(anchor)
```

Commands directly available: `set_open(bool,cx)`, `is_open()`, `set_active_tab(usize,cx)`, `preview_color`, `clear_preview`, `preview_hex(&str,window,cx)->bool`, `commit_hex(&str,window,cx)->Option<Hsla>`, `select_color`, `update_color`, `clear_value`. A command that calls commit/select has user-like Change semantics; controlled props use quiet setters. Validate normalized finite channel values. Defaults for rendering are size Medium, anchor TopLeft, themed featured palette if not supplied. `ColorPicker` has no public Disableable implementation in this source; do not expose a `disabled` prop as if it were already enforced. [color]

## 7. Select and Combobox: data delegate with stable values

Use the modern `SearchableListItem`, `SearchableListDelegate`, `SearchableVec`, `SearchableGroup` names; Select's older trait names are aliases. No JS closures are needed for ordinary rows:

```ts
interface Choice { key:string;label:string;keywords?:string[];disabled?:boolean;
  description?:string;icon?:string }
interface ChoiceGroup { key:string;label?:string;items:Choice[] }
interface ChoiceProps { items:ChoiceGroup[];dataRevision:number;disabled?:boolean;
  placeholder?:string;searchPlaceholder?:string;cleanable?:boolean;
  searchable?:boolean;menuMaxHeight?:number;menuWidth?:number }
interface SelectProps extends ChoiceProps { value:string|null }
interface ComboboxProps extends ChoiceProps { values:string[];multiple?:boolean }
```

`Choice.key` must be unique across groups and is `SearchableListItem::Value=String`; labels may repeat. All cursor `IndexPath` values are indices into the *current filtered view*. Never use row index as a durable item key. [searchable]

```rust,ignore
#[derive(Clone)]
struct Choice { key: String, label: String, keywords: Vec<String>, disabled: bool }
impl SearchableListItem for Choice {
    type Value = String;
    fn title(&self) -> SharedString { self.label.clone().into() }
    fn value(&self) -> &String { &self.key }
    fn disabled(&self) -> bool { self.disabled }
    fn matches(&self, query:&str) -> bool {
        let q = query.to_lowercase();
        self.label.to_lowercase().contains(&q) ||
            self.keywords.iter().any(|s| s.to_lowercase().contains(&q))
    }
}
type Choices = SearchableVec<Choice>; // or SearchableVec<SearchableGroup<Choice>>
let select = cx.new(|cx| SelectState::new(Choices::new(items), None, window,cx)
    .searchable(searchable));
select.update(cx,|s,cx| match value {
    Some(key) => s.set_selected_value(&key,window,cx),
    None => s.set_selected_index(None,window,cx),
});
let sub = cx.subscribe_in(&select,window,|this,_,ev:&SelectEvent<Choices>,_,_| {
    let SelectEvent::Confirm(key) = ev;
    this.events.emit(SelectionDto { key:key.clone() });
});
Select::new(&self.state).placeholder(label).cleanable(cleanable).disabled(disabled)

let combo = cx.new(|cx| ComboboxState::new(Choices::new(items), vec![],window,cx)
    .multiple(multiple).searchable(searchable));
combo.update(cx,|s,cx| s.set_selected_values(&values,window,cx));
let sub = cx.subscribe_in(&combo,window,|this,_,ev:&ComboboxEvent<Choices>,_,_| {
    this.events.emit(match ev {
        ComboboxEvent::Change(keys) => ChoicesDto::change(keys.clone()),
        ComboboxEvent::Confirm(keys) => ChoicesDto::confirm(keys.clone()),
    });
});
Combobox::new(&self.state).placeholder(label).cleanable(cleanable).disabled(disabled)
```

Both default searchable=false, appearance/focusRing=true, cleanable=false, disabled=false, menu width Auto, menu max height 20rem. Combobox defaults multiple=false; with multiple true each toggle emits Change and popup close emits Confirm. `set_selected_values` is quiet; `clear_selection` **emits** Change, so clearing a controlled prop should use `set_selected_values(&[])`. `add_selected_index` can accumulate values even when multiple=false; enforce a one-key DTO in single mode. [select][combobox]

Custom `SearchableListDelegate` required methods: `type Item`, `items_count(section)->usize`, `item(IndexPath)->Option<&Item>`, and generic `position<V>(&V)->Option<IndexPath> where Item: SearchableListItem<Value=V>, V:PartialEq`. Optional `sections_count`, `render_section_header`, `perform_search(&str,&mut Window,&mut App)->Task<()>`, `render_item`, `is_item_enabled`, `is_item_checked`, `on_will_change`, `on_confirm`. The synchronous `on_will_change` has **no App** and runs under the list's mutable lease; it cannot wait for JS or re-enter the entity. Encode veto/max-selection rules in Rust from DTOs. [searchable]

Important replacement facts: `SelectState::set_items` and `ComboboxState::set_items` only assign the inner delegate. They do not re-resolve selected item snapshots, call notify, or preserve/refilter the search. `set_selected_value(s)` clears the current query before resolving the key(s). For a basic data replacement, apply `set_items` then reapply keys, then notify; that resets search by design. A controlled value **echo** must be skipped or the search is cleared. Combobox has public `query(cx)` / `set_query(query,window,&mut App)`; Select does not expose the equivalent query methods at this revision. Neither exposes the inner list's scroll handle publicly. [select][combobox]

**Native addition for correct live data:** `replace_items_preserving_selection_and_query` must preserve selection by Value, rebuild filtered indices, update selection snapshots, and leave list offset untouched on append/label-only edits. Add mutable searchable/multiple policy setters and optional query/open accessors if those props/commands are promised. `SearchableVec::push` appends to the filtered view even if the item does not match; it is unsuitable for async append under an active filter without re-filtering. There is no `has_more/load_more` on SearchableListDelegate, so infinite loading Select/Combobox needs an explicit native delegate extension or a JS query/pagination command contract, not a fake existing hook. [searchable-vec][select][combobox]

## 8. Command

Command uses `Entity<CommandState>` plus the `Command` builder model, **not** ListDelegate. `Command::render` installs the model on every builder render; `CommandState` retains query, selection and scroll. Events are builder callbacks (`on_query`, `on_select`, `on_confirm`, `on_cancel`), not an EventEmitter enum. [command][command-state]

```ts
type CommandEntry = {kind:"item";key:string;label?:string;keywords?:string[];
  icon?:string;disabled?:boolean;checked?:boolean} |
  {kind:"group";key:string;label?:string;items:CommandItem[]} |
  {kind:"separator"};
interface CommandProps {entries:CommandEntry[];dataRevision:number;
  searchable?:boolean;filterable?:boolean;loading?:boolean;query?:string;
  selectedKey?:string|null;placeholder?:string;bordered?:boolean;maxHeight?:number}
type CommandEvent = {kind:"query";query:string;requestId:number} |
  {kind:"select"|"confirm";key:string;dataRevision:number}|{kind:"cancel"};
```

```rust,ignore
let state = cx.new(|cx| CommandState::new(window,cx));
// update changed query/loading; setters are real APIs:
state.update(cx,|s,cx| {
    if query_changed { s.set_query(query,window,cx); }
    if loading_changed { s.set_loading(loading,window,cx); }
});
// Render: model is owned data; key_by_index captures THIS installed generation.
let keys = self.key_by_index.clone(); // Rc<HashMap<IndexPath,String>>
let events = self.events.clone();
let revision = self.props.data_revision;
let command = Command::new(&self.state)
    .items(self.props.items.iter().map(|i| CommandItem::new()
        .label(i.label.clone()).keywords(i.keywords.clone())
        .checked(i.checked).disabled(i.disabled)))
    .filterable(self.props.filterable).searchable(self.props.searchable)
    .on_confirm(move |ix,_,_| {
        if let Some(key) = keys.get(&ix) {
            events.emit(CommandDto::confirm(key.clone(),revision));
        }
    });
// Add .on_select with its own cloned key map, .on_query, .on_cancel similarly.
// .group(CommandGroup::new().label(...).items(...)) and .separator() preserve hierarchy.
command
```

Ungrouped rows use section=0 and input row positions; groups use group positions, shifted by one section if any ungrouped items exist. Filtering does not alter these original model coordinates. With `filterable(false)` all provided entries stay visible; `on_query` still emits so JS can perform remote search. A query change resets selection to the first enabled item and scrolls; disable-all leaves no selection. `on_select` precedes `on_query` after state lease release; `on_confirm` runs after optional native Action dispatch; Cancel callback is synchronous before empty-query Cancel propagates. JS actions should use key events, not serialized arbitrary Rust `Action`. [command][command-state]

Native command items can use lazy `.child(|window,cx| ...)` renderers that may run for measurement and drawing, so they must be pure and native. This bridge needs a slot/template/data renderer contract before it can accept arbitrary custom JS rows. Default searchable/filterable=true, border=true, maxHeight=18.75rem. [command]

`install_model` preserves the highlight by original IndexPath and retains offset if that path survives, **not by stable key**. Reordering a key into another index can therefore silently highlight another command. Current public `set_selected_index` resolves the currently installed model and requests scrolling, so calling it in `NativeView::update` before the next Command render cannot safely restore a key across reordered entries. **Native addition:** key-based model identity or a public atomic model/selection update that optionally preserves scroll. Keep selections and custom row caches keyed by actual key and data revision, not only IndexPath. [command-state]

## 9. List

`ListState<D>::new(delegate,window,cx)` defaults searchable=false, selectable=true. List renders equal-height item rows; each section's headers and footers also have uniform heights. The implementation measures one representative row, so an adapter must bound/truncate text and use fixed row height or use a different native component for variable-height content. [list][list-delegate]

```ts
interface ListRow {key:string;label:string;description?:string;disabled?:boolean;icon?:string}
interface ListSection {key:string;header?:string;footer?:string;items:ListRow[]}
interface ListProps {sections:ListSection[];dataRevision:number;selectedKey?:string|null;
  query?:string;searchable?:boolean;selectable?:boolean;loading?:boolean;
  hasMore?:boolean;nextCursor?:string|null;requestId?:number;rowHeight:number}
type ListEvent = {kind:"select"|"confirm"|"rightClick";key:string|null;
  dataRevision:number;secondary?:boolean} | {kind:"query";query:string;requestId:number} |
  {kind:"loadMore";cursor:string|null;requestId:number;dataRevision:number}|{kind:"cancel"};
```

Required delegate methods and working minimal body:

```rust,ignore
struct Rows {
    sections: Vec<Vec<ListRow>>, selected: Option<IndexPath>,
    events: Event<ListDto>, loading: bool, has_more: bool,
    in_flight: bool, next_cursor: Option<String>, request_id: u32,
}
impl ListDelegate for Rows {
    type Item = ListItem;
    fn sections_count(&self, _: &App) -> usize { self.sections.len() }
    fn items_count(&self, section:usize, _: &App) -> usize {
        self.sections.get(section).map_or(0,Vec::len)
    }
    fn render_item(&mut self,ix:IndexPath,_:&mut Window,_:&mut Context<ListState<Self>>)
        -> Option<ListItem> {
        let row = self.sections.get(ix.section)?.get(ix.row)?;
        Some(ListItem::new(SharedString::from(row.key.clone()))
            .child(row.label.clone()).disabled(row.disabled))
    }
    fn set_selected_index(&mut self,ix:Option<IndexPath>,_:&mut Window,
        _:&mut Context<ListState<Self>>) { self.selected=ix; }
    fn loading(&self,_:&App)->bool { self.loading }
    fn has_more(&self,_:&App)->bool { self.has_more && !self.in_flight }
    fn load_more(&mut self,_:&mut Window,_:&mut Context<ListState<Self>>) {
        if !self.has_more || self.in_flight { return; }
        self.in_flight=true; self.request_id+=1;
        self.events.emit(ListDto::load_more(self.next_cursor.clone(),self.request_id));
    }
}
let state = cx.new(|cx| ListState::new(delegate,window,cx)
    .searchable(searchable).selectable(selectable));
let sub = cx.subscribe_in(&state,window,|this,state,event:&ListEvent,_,cx| {
    // Map Select/Confirm IndexPath to a key from state.read(cx).delegate(), emit once.
    // Cancel is a separate event. Delegate.confirm exposes secondary if needed.
});
List::new(&self.state).search_placeholder(placeholder).scrollbar_visible(show_scrollbar)
```

`ListEvent::{Select(IndexPath),Confirm(IndexPath),Cancel}` provides no secondary/right-click/query/load-more payload. Use `delegate.confirm(secondary,window,cx)` if secondary is required and do not also emit duplicate confirmation from the subscription. `set_right_clicked_index` is an optional delegate hook. `perform_search(&str,window,&mut Context<ListState<Self>>)->Task<()>` is the query callback; own full data + filtered index map and rebuild it outside render, returning Task::ready for local search. For remote search retain a pending task waiting for the response (generation token), rather than returning immediate completion while results remain outstanding; List starts its own search spinner and resets scroll after that Task resolves, then waits 100ms before hiding the spinner. [list-delegate][list]

Update data in `state.delegate_mut()` and `cx.notify()`, not by remounting ListState. Before replacement capture selected key; after rebuilding filtered view map key→IndexPath and call `set_selected_index` (the public method does **not scroll**). Preserve `scroll_handle().base_handle().offset()` for appends/label edits; the retained handle already remains stable. A new query intentionally resets to top; a reorder/filter change should preserve the top visible key plus intra-row offset if required, using your row-height/section-height model, then `base_handle().set_offset`. Selection restoration must not call `scroll_to_selected_item`, which requests Top. Public `scroll_to_item(IndexPath,ScrollStrategy,window,cx)` is the user-directed navigation command. [list]

Native load-more can fire repeatedly near bottom. Its built-in Task only schedules delegate invocation; it does not keep the network request pending. The delegate's in-flight gate must stay true until a matching response/error/cancel clears it. Responses carry `{requestId,dataRevision,cursor,rows,hasMore,nextCursor}`; ignore stale query generations and merge rows by stable keys. Default threshold=20 counts entities including headers/footers, not only data rows. [list-delegate][list]

## 10. Tree

`TreeState::new(&mut App).items(Vec<TreeItem>)`; `TreeItem::new(id,label).children(...).expanded(bool).disabled(bool)` stores stable id, labels/children and an `Rc<RefCell>` expanded/disabled state. Cloning a TreeItem preserves its expansion state. `Tree::new(&state, |ix,&TreeEntry,selected,window,cx| -> ListItem)` supplies styled rows. No delegate trait is required. [tree]

```ts
interface TreeNode {key:string;label:string;disabled?:boolean;children:TreeNode[];
  expanded?:boolean;icon?:string}
interface TreeProps {nodes:TreeNode[];dataRevision:number;selectedKey?:string|null}
type TreeEvent = {kind:"expanded"|"collapsed";key:string} |
  {kind:"select";key:string|null;dataRevision:number};
```

```rust,ignore
let state = cx.new(|cx| TreeState::new(cx).items(items));
let expand_sub = cx.subscribe_in(&state,window,|this,_,ev:&TreeEvent,_,_| {
    this.events.emit(match ev {
        TreeEvent::Expanded(id) => TreeDto::expanded(id.to_string()),
        TreeEvent::Collapsed(id) => TreeDto::collapsed(id.to_string()),
    });
});
let select_observer = cx.observe(&state,|this,state,cx| {
    let key = state.read(cx).selected_item().map(|item|item.id.to_string());
    if key != this.observed_selection {
        this.observed_selection=key.clone();
        this.events.emit(TreeDto::selected(key));
    }
});
Tree::new(&self.state, |_,entry,selected,_,_| {
    ListItem::new(entry.item().id.clone()).child(entry.item().label.clone())
        .selected(selected)
})
```

There is no Selected event in TreeEvent; observing and comparing selection is required for keyboard as well as pointer selection. Suppress prop-projection selection events by comparing desired and observed state or a projection generation, rather than a transient bool reset before deferred notifications. Commands: `set_selected_index(Option<usize>,cx)`, `set_selected_item(Option<&TreeItem>,cx)`, `focus(window,&mut App)`, `index_of(&SharedString)`, `reveal_item(&id,ScrollStrategy,cx)` expands ancestors, and `scroll_to_item(ix,strategy)`. [tree]

`set_items` resets selected/right-clicked indices. Retain a key→TreeItem map, clone existing items, mutate public label/children, and use `.disabled(...)`; reuse shared expansion state unless an explicit controlled expansion changed. Then `set_items`, re-resolve selected key, restore selection, retain the same scroll handle. `set_selected_item` may expand ancestors and emit Expanded; data refresh should not unexpectedly reveal hidden selections, so a visible-only `index_of` + `set_selected_index` is the correct ordinary refresh choice. Appends preserve pixel offset; structural changes need key+intra-row anchor if the same top row must remain visible. [tree]

**Native limitation:** `TreeItem::is_folder` is exactly `!children.is_empty()`. There is no `hasChildren`, `loading`, loader callback or independently expandable empty node. True lazy children cannot be exposed by a faithful wrapper at this revision. Add native `has_children`/load-state semantics and an expand-request event; use `{key,requestId,dataRevision}` response correlation. Avoid fake child rows solely to make an unloaded node appear expandable. The tree uses a uniform list, so native Tree rows must have equal heights. [tree]

## 11. DataTable

`TableState<D>::new(delegate,window,cx)` and `DataTable::new(&state)`. Defaults: sortable/resizable/movable/fixed/loopSelection/rowSelectable/colSelectable true, cellSelectable false, rowHeader true; table border and scrollbars true, stripe false. The flags are public fields and can be updated in place; style options belong to DataTable builder. There is no automatic data sorting: delegate must implement it. [table-state][table-ui]

```ts
type Cell = {text:string;sortValue?:string|number|boolean|null};
interface TableColumn {key:string;name:string;width?:number;minWidth?:number;maxWidth?:number;
  align?:"left"|"center"|"right";sortable?:boolean;sort?:"default"|"ascending"|"descending";
  fixed?:"left";resizable?:boolean;movable?:boolean;selectable?:boolean}
interface TableRow {key:string;cells:Record<string,Cell>}
interface TableSelection {kind:"row"|"column"|"cell";rowKey?:string;columnKey?:string}
interface TableProps {columns:TableColumn[];rows:TableRow[];dataRevision:number;
  selection?:TableSelection|null;loading?:boolean;hasMore?:boolean;nextCursor?:string|null;
  groupHeaders?:{label:string;span:number}[][];stripe?:boolean;bordered?:boolean;
  cellSelectable?:boolean;rowSelectable?:boolean;colSelectable?:boolean;rowHeader?:boolean}
// Selection events always include stable rowKey/columnKey plus dataRevision.
// Other events: sort, moveColumn, columnWidthsChanged, visibleRows/Columns, loadMore.
```

Required delegate methods are exactly `columns_count`, `rows_count`, `column(col_ix,&App)->Column`, and `render_td(row_ix,col_ix,window,&mut Context<TableState<Self>>)->impl IntoElement`. For a functional data table also implement `perform_sort`, `move_column`, `cell_text` and pagination/visible-range hooks. `Column::new(key,name)` defaults width=100px,min=20px,max=f32::MAX, sort=None, no fixed column, resizable/movable/selectable true. ColumnFixed only has Left. `group_headers` returns `Option<Vec<Vec<ColumnGroup>>>` where span counts logical leaf columns. [table-delegate][table-column]

```rust,ignore
struct Grid {
    columns: Vec<Column>, rows: Vec<TableRow>,
    original_order: Vec<String>, // stable source order for ColumnSort::Default
    events: Event<TableDto>, in_flight: bool, has_more: bool,
}
impl TableDelegate for Grid {
    fn columns_count(&self,_:&App)->usize { self.columns.len() }
    fn rows_count(&self,_:&App)->usize { self.rows.len() }
    fn column(&self,col:usize,_:&App)->Column { self.columns[col].clone() }
    fn render_td(&mut self,row:usize,col:usize,_:&mut Window,_:&mut Context<TableState<Self>>)
        -> impl IntoElement {
        self.rows[row].cells[&self.columns[col].key.to_string()].text.clone()
    }
    fn cell_text(&self,row:usize,col:usize,_:&App)->String {
        self.rows[row].cells[&self.columns[col].key.to_string()].text.clone()
    }
    fn render_tr(&mut self,row:usize,_:&mut Window,_:&mut Context<TableState<Self>>)
        -> Stateful<Div> { div().id(SharedString::from(self.rows[row].key.clone())) }
    fn move_column(&mut self,from:usize,to:usize,_:&mut Window,_:&mut Context<TableState<Self>>) {
        let column=self.columns.remove(from); self.columns.insert(to,column);
        // Emit here only if not subscribing to TableEvent::MoveColumn as well.
    }
    fn perform_sort(&mut self,col:usize,sort:ColumnSort,_:&mut Window,
        _:&mut Context<TableState<Self>>) {
        // EITHER sort the native index vector using validated typed sortValue,
        // stable ties resolved by original_order; Default restores original_order.
        // OR emit {columnKey,sort,requestId,dataRevision}; JS returns sorted rows.
        // No JS comparator call from this synchronous native function.
        self.events.emit(TableDto::sort(self.columns[col].key.to_string(),sort));
    }
    fn has_more(&self,_:&App)->bool { self.has_more && !self.in_flight }
    fn load_more(&mut self,_:&mut Window,_:&mut Context<TableState<Self>>) {
        if !self.has_more || self.in_flight {return}
        self.in_flight=true;
        self.events.emit(TableDto::load_more());
    }
}
let state = cx.new(|cx| TableState::new(delegate,window,cx));
let sub = cx.subscribe_in(&state,window,|this,state,ev:&TableEvent,_,cx| {
    // Resolve numeric indices with state.read(cx).delegate(); emit keys/revision.
    // On ColumnWidthsChanged(widths), first persist widths by current column key
    // in delegate.columns. This prevents later refresh from undoing the drag.
});
DataTable::new(&self.state).stripe(p.stripe).bordered(p.bordered)
    .scrollbar_visible(p.vertical_scrollbar,p.horizontal_scrollbar)
```

Sort UI cycles Default→Descending→Ascending→Default. Persist new sort state in delegate columns too: Table's internal col_groups sort indicator and delegate column DTOs are different copies. The native sorting hook cannot assume the state will reorder rows. For JS sorting include a request id and apply matching responses atomically; if rows are already locally sorted, avoid asking JS to sort them again. `move_column` must actually reorder delegate columns; native also moves its internal col_group. Resolve MoveColumn metadata from a captured old key order or from the new `to` position, not the old `from` position after the mutation. [table-state]

Exact TableEvent variants: SelectRow, DoubleClickedRow, SelectColumn, SelectCell, DoubleClickedCell, ColumnWidthsChanged(Vec<Pixels>), MoveColumn(from,to), RightClickedRow(Option<usize>), RightClickedCell, ClearSelection. No Sort event is emitted: use the delegate hook. Selection methods are not quiet: `set_selected_row` emits SelectRow and RightClickedRow(None) and scrolls; `set_selected_col` emits and scrolls; `set_selected_cell` emits and scrolls; `clear_selection` emits. A simple `projecting=true; setter; projecting=false` flag does not reliably suppress deferred subscriber delivery. [table-state]

**Native addition needed for controlled selection without refresh jumps:** a quiet setter with explicit scroll policy, preferably by stable key at delegate boundary, to restore selection after sort/filter/data replacement. Otherwise keep table selection native-owned and expose explicit navigation commands honestly; do not claim a stable controlled selection prop built from emitting scroll setters. [table-state]

`refresh(cx)` rebuilds col_groups from `delegate.column()` including widths and sort, and recomputes header layout; it does not itself call `cx.notify()`. For data-only row append/edit call `cx.notify()` while retaining TableState and both scroll handles; do not reset all column runtime widths by blindly refreshing on every props update. If column definitions changed, merge native retained widths/order/sort by column key, call refresh, then notify. If only group headers changed use `refresh_header_layout(cx)`. The two public handles are `vertical_scroll_handle: UniformListScrollHandle`, `horizontal_scroll_handle: VirtualListScrollHandle`; snapshot/restore offsets only for intentional structural anchoring. [table-state]

Pagination uses `has_more`, default `load_more_threshold=20` rows, `load_more`; the internal Task only invokes the delegate, so use the same explicit in-flight/request-generation gate as List. `visible_rows_changed(Range<usize>,window,cx)` and `visible_columns_changed` must be constant-time event/data scheduling, not render-time data fetch. Native visible range suppresses updates of len<=1 because the first row is used for measurement; do not promise it as exact one-row-viewport telemetry without a native fix. Use `dump_range(start..end,cx)` for bounded export, backed by `cell_text`; `dump` materializes the whole table. [table-delegate][table-state]

## 12. Minimal high-value verification targets

Research snippets have not been compiled. For implementation acceptance, one native scenario per ownership hazard is enough: (1) Input/Textarea/Editor IME+equal echo keeps selection/undo/entity and stale ack cannot overwrite; (2) Select/Combobox filtering then data reorder preserves selected keys/search; (3) Command filtered/reordered model confirms the key from its current generation; (4) List/Table append/loadMore produces one outstanding request and keeps scroll/column widths; (5) Tree data replacement preserves expansion/selection, true unloaded folder expansion works only after the explicit native addition; (6) date range first/second click, clear and disabled rules have specified events. These are correctness checks; calibrated CPU or empty/default render proves no native UX/performance acceptance. [existing-input][domain]

## Source index

All C/B files below are from the fixed source commit above; exact line anchors identify the owning implementation, including cases where comments disagree with defaults.

[domain]: /Users/jgbingzi/workspace/sp/solid-gpui/CONTEXT.md
[bridge]: /Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/native/component.rs:97
[existing-input]: /Users/jgbingzi/workspace/sp/solid-gpui/crates/solid-gpui/src/components/input.rs:1
[input-kinds]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/input/editor/mod.rs:1
[input-state]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/input/base/state.rs:113
[input-rope]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/input/base/rope_ext.rs:189
[input-defaults]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/input/base/state.rs:595
[input-modes]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/input/base/state.rs:4975
[input-widget]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/input/input.rs:174
[multiline-widgets]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/input/textarea.rs:14
[number-widget]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/input/number_input.rs:21
[number]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/number_input.rs:24
[editor-extra]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/input/editor/diagnostics.rs:98
[otp]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/otp_input.rs:10
[slider]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/slider.rs:14
[calendar]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/calendar.rs:12
[date-picker]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/time/date_picker.rs:73
[color]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/color_picker.rs:92
[select]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/select.rs:156
[combobox]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/combobox.rs:160
[searchable]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/searchable_list/delegate.rs:8
[searchable-vec]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/searchable_list/vec.rs:74
[command]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/command/command.rs:82
[command-state]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/command/state.rs:141
[list]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/list/list.rs:38
[list-delegate]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/list/delegate.rs:11
[tree]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/base/src/tree.rs:38
[table-delegate]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/table/delegate.rs:18
[table-state]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/table/state.rs:189
[table-ui]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/table/data_table.rs:104
[table-column]: /Users/jgbingzi/.cargo/git/checkouts/gpui-component-95ce574d8a0da8b8/928c3eb/crates/component/src/table/column.rs:9

use super::*;
use crate::{
    Root,
    setting::{SettingGroup, SettingItem},
};
use gpui::{
    Context, InteractiveElement as _, Render, TestAppContext, VisualTestContext, point, size,
};

struct SettingsHost {
    pages: Vec<SettingPage>,
    state: Entity<SettingsState>,
}

impl Render for SettingsHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Settings::new("search-test")
            .pages(self.pages.clone())
            .with_state(&self.state)
    }
}

fn item(keyword: &'static str) -> SettingItem {
    item_with_height(keyword, 80.)
}

fn item_with_height(keyword: &'static str, height: f32) -> SettingItem {
    SettingItem::render(keyword, move |options: &RenderOptions, _, _| {
        let selector = format!(
            "setting-{}-{}-{}",
            options.page_ix(),
            options.group_ix(),
            options.item_ix()
        );
        div()
            .h(px(height))
            .child("Setting")
            .debug_selector(move || selector.clone())
    })
    .keywords([keyword])
}

fn pages() -> Vec<SettingPage> {
    vec![
        SettingPage::new("general", "General")
            .group(SettingGroup::new("general").item(item("language"))),
        SettingPage::new("appearance", "Appearance")
            .default_open(true)
            .groups([
                SettingGroup::new("misc").item(item("unrelated")),
                SettingGroup::new("colors")
                    .title("Colors")
                    .item(item("theme colors")),
                SettingGroup::new("fonts")
                    .title("Fonts")
                    .items([item("unrelated"), item("theme font")]),
            ]),
        SettingPage::new("editor", "Editor")
            .group(SettingGroup::new("editor").item(item("theme editor"))),
    ]
}

fn setup(cx: &mut TestAppContext) -> (Entity<SettingsHost>, &mut VisualTestContext) {
    cx.update(|cx| {
        crate::init(cx);
        crate::Theme::global_mut(cx).font_size = px(16.);
    });
    let pages = pages();
    let mut host = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let state = cx.new(|cx| SettingsState::new(window, cx));
        // The fixture starts on the Appearance page, as the upstream index
        // fixture did through `default_selected_index`.
        state.update(cx, |state, cx| {
            state.select(
                SettingSelection {
                    page: "appearance".into(),
                    group: None,
                },
                cx,
            );
        });
        let view = cx.new(|_| SettingsHost { pages, state });
        host = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.simulate_resize(size(px(1000.), px(700.)));
    draw(cx);
    (host.unwrap(), cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn search(host: &Entity<SettingsHost>, query: &str, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let input = host.read(cx).state.read(cx).search_input.clone();
        input.update(cx, |input, cx| input.set_value(query, window, cx));
    });
    draw(cx);
}

/// The selection after the current frame, if anything is selected. An empty
/// filter leaves nothing selected until the query clears or changes.
fn selection(host: &Entity<SettingsHost>, cx: &mut VisualTestContext) -> Option<SettingSelection> {
    cx.update(|_, cx| host.read(cx).state.read(cx).selected().cloned())
}

fn click_nav(y: f32, cx: &mut VisualTestContext) {
    // Fixed-size fixture with a 16px rem; click the label area, away from carets.
    cx.simulate_click(point(px(50.), px(y)), Default::default());
    draw(cx);
}

#[gpui::test]
fn search_preserves_the_selected_page_and_clicks_select_by_key(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: None,
        })
    );

    // The Appearance page keeps the selection while it matches; hidden pages
    // draw nothing.
    search(&host, "theme", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: None,
        })
    );
    assert!(cx.debug_bounds("setting-0-1-0").is_some());
    assert!(cx.debug_bounds("setting-2-0-0").is_none());

    // Only the Fonts group matches "font"; its sole survivor draws at the
    // first filtered position of the only matching page.
    search(&host, "font", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: None,
        })
    );
    assert!(cx.debug_bounds("setting-0-0-0").is_some());

    // Select Editor from the compressed result list, then clear the query.
    search(&host, "theme", cx);
    click_nav(156., cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "editor".into(),
            group: None,
        })
    );
    search(&host, "", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "editor".into(),
            group: None,
        })
    );
    assert!(cx.debug_bounds("setting-2-0-0").is_some());

    // The current page disappears from the filter: the first matching page
    // shows instead, and nothing draws once nothing matches. Clearing the
    // query restores the page the user picked.
    search(&host, "font", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: None,
        })
    );
    assert!(cx.debug_bounds("setting-0-0-0").is_some());
    search(&host, "no matching setting", cx);
    assert_eq!(selection(&host, cx), None);
    assert!(cx.debug_bounds("setting-0-0-0").is_none());
    search(&host, "", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "editor".into(),
            group: None,
        })
    );
}

#[gpui::test]
fn search_preserves_group_and_item_identity(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    // Make the target group require scrolling, with an untitled group before it.
    cx.update(|_, cx| {
        host.update(cx, |host, cx| {
            host.pages[1].groups[1].items = vec![item_with_height("theme colors", 450.).into()];
            cx.notify();
        });
    });
    draw(cx);
    click_nav(156., cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: Some("fonts".into()),
        })
    );
    search(&host, "theme", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: Some("fonts".into()),
        })
    );
    // Both surviving groups draw their single matching item at compacted
    // positions: Colors at 0-0-0, Fonts at 0-1-0.
    assert!(cx.debug_bounds("setting-0-0-0").is_some());
    assert!(cx.debug_bounds("setting-0-1-0").is_some());
    // Click Fonts after the leading page/group/item have been filtered out.
    click_nav(120., cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: Some("fonts".into()),
        })
    );
    let target = cx.debug_bounds("setting-0-1-0").unwrap();
    assert!(target.top() >= px(0.) && target.bottom() <= px(700.));
    search(&host, "font", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: Some("fonts".into()),
        })
    );
    search(&host, "", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: Some("fonts".into()),
        })
    );
    search(&host, "colors", cx);
    assert_eq!(
        selection(&host, cx),
        Some(SettingSelection {
            page: "appearance".into(),
            group: None,
        })
    );
    assert!(cx.debug_bounds("setting-0-0-0").is_some());
}

#[gpui::test]
fn resetting_search_results_leaves_hidden_settings_unchanged(cx: &mut TestAppContext) {
    use std::{cell::Cell, rc::Rc};

    let (_, cx) = setup(cx);
    let visible = Rc::new(Cell::new(false));
    let hidden = Rc::new(Cell::new(true));
    let resettable_item = |keyword, dirty: &Rc<Cell<bool>>| {
        let read = dirty.clone();
        let reset = dirty.clone();
        item(keyword).on_reset(move |_| read.get(), move |_, _| reset.set(false))
    };
    let group = SettingGroup::new("reset").items([
        resettable_item("theme", &visible),
        resettable_item("hidden", &hidden),
    ]);
    cx.update(|window, cx| {
        assert!(!group.is_resettable("theme", cx));
        visible.set(true);
        assert!(group.is_resettable("theme", cx));
        group.reset("theme", window, cx);
    });
    assert!(!visible.get());
    assert!(hidden.get());
}

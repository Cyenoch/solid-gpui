use super::support::*;
use crate::renderer::ReactRoot;
use gpui::AppContext as _;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const PROPERTY_SEEDS: usize = 16;
const PROPERTY_STEPS: usize = 48;

#[derive(Clone, Copy)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9e37_79b9_7f4a_7c15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        (self.next_u64() % upper as u64) as usize
    }
}

#[derive(Clone)]
struct Surface {
    id: u32,
    epoch: u32,
    revision: u32,
    window: gpui::WindowHandle<ReactRoot>,
    root: gpui::Entity<ReactRoot>,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    OpenFresh,
    ReopenRetired,
    Close,
    ValidCommand,
    InvalidEpochCommand,
    WrongSurfaceCommand,
    Focus,
    Activate,
}

impl Operation {
    fn name(self) -> &'static str {
        match self {
            Self::OpenFresh => "openSurface(fresh)",
            Self::ReopenRetired => "openSurface(reused-id)",
            Self::Close => "closeSurface",
            Self::ValidCommand => "valid-command",
            Self::InvalidEpochCommand => "invalid-epoch-command",
            Self::WrongSurfaceCommand => "wrong-surface-command",
            Self::Focus => "focus-cross-surface",
            Self::Activate => "activate-cross-surface",
        }
    }
}

fn snapshot_for(surface_id: u32, epoch: u32) -> Snapshot {
    let mut focusable = Node::new(2, 1, 0, KIND_VIEW);
    focusable.listener_id = 2;
    focusable.focusable = true;
    Snapshot::new(
        surface_id,
        epoch,
        0,
        1,
        vec![Node::new(1, 0, 0, KIND_VIEW), focusable],
    )
}

fn draw_surface(surface: &Surface, cx: &mut gpui::TestAppContext) {
    cx.update_window(surface.window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .expect("draw surface");
    cx.run_until_parked();
}

fn drain_events(runtime: &InMemoryAdapter) {
    while runtime
        .take_event()
        .expect("read surface property event")
        .is_some()
    {}
}

fn open_surface(
    cx: &mut gpui::TestAppContext,
    runtime: &Arc<InMemoryAdapter>,
    id: u32,
    epoch: u32,
) -> Surface {
    let window = cx.open_window(gpui::size(gpui::px(320.0), gpui::px(240.0)), {
        let runtime = Arc::clone(runtime);
        move |_, _| ReactRoot::new(runtime)
    });
    let root = window.root(cx).expect("surface root");
    let snapshot = snapshot_for(id, epoch)
        .encode()
        .expect("encode surface property snapshot");
    root.update(cx, |root, cx| root.apply_payload(&snapshot, cx))
        .expect("apply surface property snapshot");
    let surface = Surface {
        id,
        epoch,
        revision: 1,
        window,
        root,
    };
    draw_surface(&surface, cx);
    drain_events(runtime);
    surface
}

fn command(
    surface_id: u32,
    epoch: u32,
    revision: u32,
    request_id: u32,
    node_id: u32,
    kind: u32,
) -> Command {
    Command {
        protocol: PROTOCOL_VERSION,
        message: COMMAND_MESSAGE,
        surface_id,
        epoch,
        after_revision: revision,
        request_id,
        node_id,
        kind,
        payload: None,
        scroll_offset: None,
        title: None,
        body: None,
        actions: None,
        menus: None,
        keybindings: None,
        window_options: None,
        image: None,
    }
}

fn command_result(runtime: &InMemoryAdapter, request_id: u32) -> CommandResult {
    loop {
        let event = runtime
            .take_event()
            .expect("read surface command result")
            .unwrap_or_else(|| panic!("missing command result for request {request_id}"));
        if event.event_type == EVENT_COMMAND_RESULT
            && let Some(EventPayload::CommandResult(result)) = event.payload
            && result.request_id == request_id
        {
            return result;
        }
    }
}

fn send_command(
    cx: &mut gpui::TestAppContext,
    runtime: &InMemoryAdapter,
    surface: &Surface,
    command: Command,
) -> CommandResult {
    let request_id = command.request_id;
    let payload = command.encode().expect("encode surface property command");
    surface
        .root
        .update(cx, |root, cx| root.apply_payload(&payload, cx))
        .expect("queue surface property command");
    draw_surface(surface, cx);
    command_result(runtime, request_id)
}

fn choose_live(live: &HashMap<u32, Surface>, rng: &mut Rng) -> Surface {
    let mut ids: Vec<u32> = live.keys().copied().collect();
    ids.sort_unstable();
    live[&ids[rng.below(ids.len())]].clone()
}

fn assert_live_surface_invariants(
    live: &HashMap<u32, Surface>,
    retired: &[Surface],
    cx: &gpui::TestAppContext,
) {
    let live_ids: HashSet<u32> = live.keys().copied().collect();
    assert!(
        live_ids
            .iter()
            .all(|id| !retired.iter().any(|old| old.id == *id)),
        "retired surface id was live without an explicit reopen"
    );
    for surface in live.values() {
        surface.root.read_with(cx, |root, _| {
            assert_eq!(root.store().surface_id(), surface.id, "surface id changed");
            assert_eq!(root.store().epoch(), surface.epoch, "surface epoch changed");
            for (name, ids) in root.test_side_map_ids() {
                assert!(
                    ids.iter().all(|id| root.store().get(*id).is_some()),
                    "surface {}/{} map {name} retained a dead node",
                    surface.id,
                    surface.epoch
                );
            }
        });
    }
}

fn run_property_sequences(cx: &mut gpui::TestAppContext, seeds: std::ops::Range<u64>) {
    let runtime = InMemoryAdapter::new();
    let mut next_id = 3u32;
    let mut next_request = 1u32;

    for seed in seeds {
        let main = open_surface(cx, &runtime, 1, 1);
        let child = open_surface(cx, &runtime, 2, 1);
        let mut live = HashMap::from([(main.id, main), (child.id, child)]);
        let mut retired: Vec<Surface> = Vec::new();
        let mut history = Vec::with_capacity(PROPERTY_STEPS);
        let mut rng = Rng::new(seed);

        for step in 0..PROPERTY_STEPS {
            let roll = rng.below(100);
            let operation = if roll < 14 {
                Operation::OpenFresh
            } else if roll < 24 && !retired.is_empty() {
                Operation::ReopenRetired
            } else if roll < 34 && live.len() > 1 {
                Operation::Close
            } else if roll < 54 {
                Operation::ValidCommand
            } else if roll < 69 {
                Operation::InvalidEpochCommand
            } else if roll < 80 {
                Operation::WrongSurfaceCommand
            } else if roll < 91 {
                Operation::Focus
            } else {
                Operation::Activate
            };
            history.push(format!("{step}: {}", operation.name()));

            match operation {
                Operation::OpenFresh => {
                    let surface = open_surface(cx, &runtime, next_id, 1);
                    assert!(live.insert(surface.id, surface).is_none());
                    next_id += 1;
                }
                Operation::ReopenRetired => {
                    let old = retired[rng.below(retired.len())].clone();
                    let reopened = open_surface(cx, &runtime, old.id, old.epoch + 1);
                    assert!(live.insert(reopened.id, reopened.clone()).is_none());
                    retired.retain(|surface| surface.id != reopened.id);
                    let result = send_command(
                        cx,
                        &runtime,
                        &reopened,
                        command(
                            reopened.id,
                            old.epoch,
                            reopened.revision,
                            next_request,
                            1,
                            COMMAND_ACTIVATE_WINDOW,
                        ),
                    );
                    next_request += 1;
                    assert!(
                        !result.success,
                        "old epoch command was accepted: {result:?}"
                    );
                    assert_eq!(result.error.as_deref(), Some("surface or epoch mismatch"));
                }
                Operation::Close => {
                    let surface = choose_live(&live, &mut rng);
                    let closed = live.remove(&surface.id).expect("live surface to close");
                    retired.push(closed);
                }
                Operation::ValidCommand => {
                    let surface = choose_live(&live, &mut rng);
                    let result = send_command(
                        cx,
                        &runtime,
                        &surface,
                        command(
                            surface.id,
                            surface.epoch,
                            surface.revision,
                            next_request,
                            1,
                            COMMAND_GET_WINDOW_SIZE,
                        ),
                    );
                    next_request += 1;
                    assert!(result.success, "valid command rejected: {result:?}");
                }
                Operation::InvalidEpochCommand => {
                    let surface = choose_live(&live, &mut rng);
                    let result = send_command(
                        cx,
                        &runtime,
                        &surface,
                        command(
                            surface.id,
                            surface.epoch.wrapping_add(1),
                            surface.revision,
                            next_request,
                            1,
                            COMMAND_GET_WINDOW_SIZE,
                        ),
                    );
                    next_request += 1;
                    assert!(
                        !result.success,
                        "invalid epoch command accepted: {result:?}"
                    );
                    assert_eq!(result.error.as_deref(), Some("surface or epoch mismatch"));
                }
                Operation::WrongSurfaceCommand => {
                    let surface = choose_live(&live, &mut rng);
                    let result = send_command(
                        cx,
                        &runtime,
                        &surface,
                        command(
                            surface.id.wrapping_add(0x1000),
                            surface.epoch,
                            surface.revision,
                            next_request,
                            1,
                            COMMAND_GET_WINDOW_SIZE,
                        ),
                    );
                    next_request += 1;
                    assert!(
                        !result.success,
                        "wrong surface command accepted: {result:?}"
                    );
                    assert_eq!(result.error.as_deref(), Some("surface or epoch mismatch"));
                }
                Operation::Focus => {
                    let surface = choose_live(&live, &mut rng);
                    let result = send_command(
                        cx,
                        &runtime,
                        &surface,
                        command(
                            surface.id,
                            surface.epoch,
                            surface.revision,
                            next_request,
                            2,
                            COMMAND_FOCUS,
                        ),
                    );
                    next_request += 1;
                    assert!(result.success, "focus command rejected: {result:?}");
                }
                Operation::Activate => {
                    let surface = choose_live(&live, &mut rng);
                    let result = send_command(
                        cx,
                        &runtime,
                        &surface,
                        command(
                            surface.id,
                            surface.epoch,
                            surface.revision,
                            next_request,
                            1,
                            COMMAND_ACTIVATE_WINDOW,
                        ),
                    );
                    next_request += 1;
                    assert!(result.success, "activation command rejected: {result:?}");
                }
            }

            assert_live_surface_invariants(&live, &retired, cx);
        }

        assert!(!live.is_empty(), "seed {seed} closed every surface");
        let _ = history;
    }
}

#[gpui::test]
fn seeded_surface_lifecycle_sequences_preserve_epoch_and_routing_invariants(
    cx: &mut gpui::TestAppContext,
) {
    run_property_sequences(cx, 0..PROPERTY_SEEDS as u64);
}

#[gpui::test]
fn seed_zero_surface_lifecycle_regression(cx: &mut gpui::TestAppContext) {
    run_property_sequences(cx, 0..1);
}

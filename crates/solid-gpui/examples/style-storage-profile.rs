//! Bounded allocation attribution on the real retained store and transaction.
//! Run serially; elapsed time is diagnostic and is not a presentation metric.
use solid_gpui::protocol::{BoxShadow, UPDATE_STYLE, UPDATE_TEXT};
use solid_gpui::{Node, NodeStore, Patch, PatchOperation, Snapshot, StoredNode, Style};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

struct Counting;
static ENABLED: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static RELEASED: AtomicUsize = AtomicUsize::new(0);
#[global_allocator]
static ALLOCATOR: Counting = Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && ENABLED.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if ENABLED.load(Relaxed) {
            RELEASED.fetch_add(layout.size(), Relaxed);
        }
        unsafe {
            System.dealloc(pointer, layout);
        }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() && ENABLED.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(size, Relaxed);
            RELEASED.fetch_add(layout.size(), Relaxed);
        }
        next
    }
}

fn measure<T>(name: &str, operation: impl FnOnce() -> T) -> T {
    ALLOCS.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    RELEASED.store(0, Relaxed);
    ENABLED.store(true, Relaxed);
    let start = Instant::now();
    let result = operation();
    let elapsed = start.elapsed();
    ENABLED.store(false, Relaxed);
    println!(
        "{{\"phase\":\"{name}\",\"allocations\":{},\"allocated_bytes\":{},\"released_bytes\":{},\"elapsed_us\":{}}}",
        ALLOCS.load(Relaxed),
        BYTES.load(Relaxed),
        RELEASED.load(Relaxed),
        elapsed.as_micros()
    );
    result
}

fn style(heap: bool, x: f32) -> Style {
    Style {
        width: Some(100.0),
        height: Some(24.0),
        left: Some(x),
        font_family: heap.then(|| "System UI".to_owned()),
        box_shadows: heap.then(|| {
            vec![BoxShadow {
                offset_x: 0.0,
                offset_y: 2.0,
                blur_radius: 4.0,
                spread_radius: 0.0,
                color_rgba: 0x00000040,
                inset: false,
            }]
        }),
        ..Style::default()
    }
}

fn operation(id: u32, mask: u32, style: Option<Style>, text: Option<String>) -> PatchOperation {
    PatchOperation::Update {
        id,
        mask,
        text,
        style,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
        observes_layout: false,
    }
}

fn main() {
    println!(
        "{{\"style_bytes\":{},\"stored_node_bytes\":{},\"optional_style_bytes\":{}}}",
        std::mem::size_of::<Style>(),
        std::mem::size_of::<StoredNode>(),
        std::mem::size_of::<Option<Style>>()
    );
    for heap in [false, true] {
        let nodes = (0..10_000).flat_map(|index| {
            let id = 2 + index * 2;
            let mut node = Node::new(id, 1, index, solid_gpui::KIND_TEXT);
            node.style = Some(style(heap, 0.0));
            let mut raw = Node::new(id + 1, id, 0, solid_gpui::KIND_RAW_TEXT);
            raw.text = Some("before".into());
            [node, raw]
        });
        let snapshot = Snapshot::new(
            1,
            1,
            0,
            1,
            std::iter::once(Node::new(1, 0, 0, solid_gpui::KIND_VIEW))
                .chain(nodes)
                .collect(),
        );
        let name = if heap { "heap" } else { "compact" };
        let mut store = measure(&format!("{name}_mount"), || {
            let mut store = NodeStore::empty();
            store.apply_snapshot(snapshot).unwrap();
            store
        });
        let text_patches: Vec<_> = (2..=1_001)
            .map(|revision| {
                Patch::new(
                    1,
                    1,
                    revision - 1,
                    revision,
                    vec![operation(
                        3,
                        UPDATE_TEXT,
                        None,
                        Some(format!("value {revision}")),
                    )],
                )
            })
            .collect();
        measure(&format!("{name}_text_journal"), || {
            for patch in text_patches {
                store.apply_patch(patch).unwrap();
            }
        });
        let noops: Vec<_> = (1_002..=2_001)
            .map(|revision| {
                Patch::new(
                    1,
                    1,
                    revision - 1,
                    revision,
                    vec![operation(2, UPDATE_STYLE, Some(style(heap, 0.0)), None)],
                )
            })
            .collect();
        measure(&format!("{name}_equal_style"), || {
            for patch in noops {
                store.apply_patch(patch).unwrap();
            }
        });
        let drag: Vec<_> = (2_002..=3_001)
            .map(|revision| {
                Patch::new(
                    1,
                    1,
                    revision - 1,
                    revision,
                    vec![operation(
                        2,
                        UPDATE_STYLE,
                        Some(style(heap, revision as f32)),
                        None,
                    )],
                )
            })
            .collect();
        measure(&format!("{name}_drag_style"), || {
            for patch in drag {
                store.apply_patch(patch).unwrap();
            }
        });
        assert_eq!(store.revision(), 3_001);
        assert_eq!(
            store.get(2).unwrap().style.as_ref().unwrap().left,
            Some(3_001.0)
        );
        assert_eq!(
            store.get(2).unwrap().text_content.as_deref(),
            Some("value 1001")
        );
        measure(&format!("{name}_collapse"), || drop(store));
    }
}

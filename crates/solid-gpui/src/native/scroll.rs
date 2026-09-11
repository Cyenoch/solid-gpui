use gpui::{Axis, Bounds, ListState, Pixels, Point, ScrollHandle};
use std::cell::Cell;
use std::rc::Rc;

/// A unified viewport borrowed by scroll decorators.
#[derive(Clone)]
pub struct ScrollViewport {
    kind: ViewportKind,
    axis: Axis,
    suppression: Rc<Cell<usize>>,
}

#[derive(Clone)]
enum ViewportKind {
    Handle(ScrollHandle),
    List(ListState),
}

impl ScrollViewport {
    pub fn new(handle: ScrollHandle, axis: Axis) -> Self {
        Self {
            kind: ViewportKind::Handle(handle),
            axis,
            suppression: Rc::new(Cell::new(0)),
        }
    }

    pub fn list(state: ListState) -> Self {
        Self {
            kind: ViewportKind::List(state),
            axis: Axis::Vertical,
            suppression: Rc::new(Cell::new(0)),
        }
    }

    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn with_axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn is_decorated(&self) -> bool {
        self.suppression.get() != 0
    }

    pub fn decorate(self) -> ScrollDecoration {
        self.suppression.set(self.suppression.get() + 1);
        ScrollDecoration { viewport: self }
    }

    pub fn offset(&self) -> Point<Pixels> {
        match &self.kind {
            ViewportKind::Handle(handle) => handle.offset(),
            ViewportKind::List(state) => state.scroll_px_offset_for_scrollbar(),
        }
    }

    pub fn set_offset(&self, offset: Point<Pixels>) {
        match &self.kind {
            ViewportKind::Handle(handle) => handle.set_offset(offset),
            ViewportKind::List(state) => state.set_offset_from_scrollbar(offset),
        }
    }

    pub fn max_offset(&self) -> Point<Pixels> {
        match &self.kind {
            ViewportKind::Handle(handle) => handle.max_offset(),
            ViewportKind::List(state) => state.max_offset_for_scrollbar(),
        }
    }

    pub fn viewport_bounds(&self) -> Bounds<Pixels> {
        match &self.kind {
            ViewportKind::Handle(handle) => handle.bounds(),
            ViewportKind::List(state) => state.viewport_bounds(),
        }
    }
}

pub struct ScrollDecoration {
    viewport: ScrollViewport,
}

impl ScrollDecoration {
    pub fn viewport(&self) -> &ScrollViewport {
        &self.viewport
    }
}

impl Drop for ScrollDecoration {
    fn drop(&mut self) {
        let state = &self.viewport.suppression;
        state.set(state.get() - 1);
    }
}

#[cfg(feature = "component-runtime")]
impl gpui_base::ScrollbarHandle for ScrollViewport {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.viewport_bounds()
    }
    fn offset(&self) -> Point<Pixels> {
        self.offset()
    }
    fn set_offset(&self, offset: Point<Pixels>) {
        self.set_offset(offset)
    }
    fn content_size(&self) -> gpui::Size<Pixels> {
        self.viewport_bounds().size + self.max_offset().into()
    }
    fn start_drag(&self) {
        if let ViewportKind::List(state) = &self.kind {
            state.scrollbar_drag_started();
        }
    }
    fn end_drag(&self) {
        if let ViewportKind::List(state) = &self.kind {
            state.scrollbar_drag_ended();
        }
    }
}

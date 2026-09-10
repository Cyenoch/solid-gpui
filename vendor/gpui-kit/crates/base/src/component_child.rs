//! A concrete compound child whose rendering boundary survives parent configuration.
use crate::component_traits::Collapsible;
use gpui::{AnyElement, App, IntoElement, RenderOnce, StyleRefinement, Styled, Window};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

#[derive(Clone)]
pub struct ComponentChild<T> {
    native: T,
    boundary: Option<Rc<dyn Fn(AnyElement) -> AnyElement>>,
}
impl<T> From<T> for ComponentChild<T> {
    fn from(native: T) -> Self {
        Self {
            native,
            boundary: None,
        }
    }
}
impl<T> ComponentChild<T> {
    pub fn wrap_element(&self, element: AnyElement) -> AnyElement {
        match &self.boundary {
            Some(boundary) => boundary(element),
            None => element,
        }
    }
    pub fn with_boundary(native: T, boundary: Rc<dyn Fn(AnyElement) -> AnyElement>) -> Self {
        Self {
            native,
            boundary: Some(boundary),
        }
    }
    pub fn map_native<U>(self, f: impl FnOnce(T) -> U) -> ComponentChild<U> {
        ComponentChild {
            native: f(self.native),
            boundary: self.boundary,
        }
    }
    /// Materialize a descriptor while retaining its boundary until the element exists.
    pub fn render_with<E: IntoElement>(self, f: impl FnOnce(T) -> E) -> AnyElement {
        self.map_native(f).into_any_element()
    }
    /// Enter the child's rendering scope before invoking its native builder.
    pub fn render_deferred<E: IntoElement + 'static, F>(self, render: F) -> AnyElement
    where
        T: 'static,
        F: FnOnce(T, &mut Window, &mut App) -> E + 'static,
    {
        self.map_native(|native| DeferredChild { native, render })
            .into_any_element()
    }
}
#[derive(IntoElement)]
struct DeferredChild<
    T: 'static,
    E: IntoElement + 'static,
    F: FnOnce(T, &mut Window, &mut App) -> E + 'static,
> {
    native: T,
    render: F,
}
impl<T: 'static, E: IntoElement + 'static, F: FnOnce(T, &mut Window, &mut App) -> E + 'static>
    RenderOnce for DeferredChild<T, E, F>
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        (self.render)(self.native, window, cx)
    }
}
impl<T> Deref for ComponentChild<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.native
    }
}
impl<T> DerefMut for ComponentChild<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.native
    }
}
impl<T: IntoElement> IntoElement for ComponentChild<T> {
    type Element = AnyElement;
    fn into_element(self) -> AnyElement {
        let element = self.native.into_any_element();
        match self.boundary {
            Some(boundary) => boundary(element),
            None => element,
        }
    }
}
impl<T: Styled> Styled for ComponentChild<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.native.style()
    }
}
impl<T: Collapsible> Collapsible for ComponentChild<T> {
    fn is_collapsed(&self) -> bool {
        self.native.is_collapsed()
    }
    fn collapsed(self, value: bool) -> Self {
        self.map_native(|v| v.collapsed(value))
    }
}

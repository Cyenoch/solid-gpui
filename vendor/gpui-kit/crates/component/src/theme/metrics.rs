//! Application-level component metrics, resolved before per-instance styles.
use gpui::{Pixels, Styled};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ComponentMetrics {
    pub height: Option<Pixels>,
    pub font_size: Option<Pixels>,
    pub line_height: Option<Pixels>,
    pub padding_x: Option<Pixels>,
    pub padding_y: Option<Pixels>,
    pub radius: Option<Pixels>,
}
impl ComponentMetrics {
    pub fn apply<E: Styled>(&self, mut element: E) -> E {
        if let Some(value) = self.height {
            element = element.h(value);
        }
        if let Some(value) = self.font_size {
            element = element.text_size(value);
        }
        if let Some(value) = self.line_height {
            element = element.line_height(value);
        }
        if let Some(value) = self.padding_x {
            element = element.px(value);
        }
        if let Some(value) = self.padding_y {
            element = element.py(value);
        }
        if let Some(value) = self.radius {
            element = element.rounded(value);
        }
        element
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ComponentMetricsSet {
    pub button: ComponentMetrics,
    pub input: ComponentMetrics,
    pub select: ComponentMetrics,
    pub tag: ComponentMetrics,
    pub menu: ComponentMetrics,
    pub dialog: ComponentMetrics,
}

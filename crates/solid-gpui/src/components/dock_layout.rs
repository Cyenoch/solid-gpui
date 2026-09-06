//! Serializable dock layout uses stable application pane names, never native entity IDs.
use super::{
    plot::{coordinate, length},
    primitives::Orientation,
};
use gpui::{Bounds, Pixels, point, px, size};
use gpui_component::dock::{
    DockAreaState, DockPlacement, DockState, PanelInfo, PanelState, TileMeta,
};
use std::collections::{BTreeMap, HashSet};
pub(super) const PANEL_NAME: &str = "solid-gpui-pane";
pub(super) fn name(v: &str) -> Result<(), String> {
    if v.is_empty() || v.len() > 256 {
        Err("dock pane names must contain 1 to 256 bytes".into())
    } else {
        Ok(())
    }
}
fn yes() -> bool {
    true
}
fn dock_size() -> f32 {
    240.
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Default)]
#[serde(untagged)]
pub enum DockValue {
    #[default]
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<DockValue>),
    Object(BTreeMap<String, DockValue>),
}
impl DockValue {
    pub(super) fn validate(&self) -> Result<(), String> {
        fn visit(v: &DockValue, depth: usize, count: &mut usize) -> Result<(), String> {
            *count += 1;
            if depth > 32 || *count > 4096 {
                return Err("pane data supports 32 levels and 4096 values".into());
            }
            match v {
                DockValue::Number(v)
                    if !v.is_finite() || (v.fract() == 0. && v.abs() > 9_007_199_254_740_991.) =>
                {
                    return Err("pane data numbers must be finite and safe JSON integers".into());
                }
                DockValue::Array(a) => {
                    for v in a {
                        visit(v, depth + 1, count)?;
                    }
                }
                DockValue::Object(o) => {
                    for v in o.values() {
                        visit(v, depth + 1, count)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        visit(self, 0, &mut 0)
    }
}
#[crate::native_type]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum DockRegion {
    #[default]
    Center,
    Left,
    Right,
    Bottom,
}
impl From<DockRegion> for DockPlacement {
    fn from(v: DockRegion) -> Self {
        match v {
            DockRegion::Center => Self::Center,
            DockRegion::Left => Self::Left,
            DockRegion::Right => Self::Right,
            DockRegion::Bottom => Self::Bottom,
        }
    }
}
impl From<DockPlacement> for DockRegion {
    fn from(v: DockPlacement) -> Self {
        match v {
            DockPlacement::Center => Self::Center,
            DockPlacement::Left => Self::Left,
            DockPlacement::Right => Self::Right,
            DockPlacement::Bottom => Self::Bottom,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, PartialEq)]
pub struct DockBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl DockBounds {
    pub(super) fn validate(&self) -> Result<(), String> {
        coordinate(self.x)?;
        coordinate(self.y)?;
        length(self.width)?;
        length(self.height)?;
        if self.width < 100. || self.height < 100. {
            return Err("dock tiles require width and height of at least 100 pixels".into());
        }
        Ok(())
    }
    pub(super) fn native(self) -> Bounds<Pixels> {
        Bounds::new(
            point(px(self.x), px(self.y)),
            size(px(self.width), px(self.height)),
        )
    }
}
impl From<Bounds<Pixels>> for DockBounds {
    fn from(v: Bounds<Pixels>) -> Self {
        Self {
            x: v.origin.x.as_f32(),
            y: v.origin.y.as_f32(),
            width: v.size.width.as_f32(),
            height: v.size.height.as_f32(),
        }
    }
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
pub struct DockSplitChild {
    pub layout: DockNode,
    #[serde(default)]
    pub size: Option<f32>,
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DockTile {
    pub pane: String,
    pub bounds: DockBounds,
    #[serde(default)]
    pub z_index: u32,
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DockNode {
    Split {
        #[serde(default)]
        orientation: Orientation,
        children: Vec<DockSplitChild>,
    },
    Tabs {
        panes: Vec<String>,
        #[serde(default)]
        active_index: usize,
    },
    Tiles {
        panes: Vec<DockTile>,
    },
}
impl Default for DockNode {
    fn default() -> Self {
        Self::Tabs {
            panes: vec![],
            active_index: 0,
        }
    }
}
#[crate::native_type]
#[derive(Clone, PartialEq)]
pub struct DockRegionLayout {
    pub layout: DockNode,
    #[serde(default = "dock_size")]
    pub size: f32,
    #[serde(default = "yes")]
    pub open: bool,
    #[serde(default = "yes")]
    pub collapsible: bool,
}
#[crate::native_type]
#[derive(Clone, Default, PartialEq)]
pub struct DockLayoutSpec {
    #[serde(default)]
    pub center: DockNode,
    #[serde(default)]
    pub left: Option<DockRegionLayout>,
    #[serde(default)]
    pub right: Option<DockRegionLayout>,
    #[serde(default)]
    pub bottom: Option<DockRegionLayout>,
}
impl DockLayoutSpec {
    pub(super) fn regions(&self) -> [(DockRegion, Option<&DockRegionLayout>); 3] {
        [
            (DockRegion::Left, self.left.as_ref()),
            (DockRegion::Right, self.right.as_ref()),
            (DockRegion::Bottom, self.bottom.as_ref()),
        ]
    }
    pub(super) fn validate(&self, known: Option<&HashSet<&str>>) -> Result<(), String> {
        fn pane(
            p: &str,
            known: Option<&HashSet<&str>>,
            used: &mut HashSet<String>,
        ) -> Result<(), String> {
            name(p)?;
            if known.is_some_and(|set| !set.contains(p)) {
                return Err(format!("layout references an unconfigured pane: {p}"));
            }
            if !used.insert(p.to_owned()) {
                return Err(format!("pane appears more than once in layout: {p}"));
            }
            if used.len() > 128 {
                return Err("dock supports at most 128 panes".into());
            }
            Ok(())
        }
        fn visit(
            node: &DockNode,
            known: Option<&HashSet<&str>>,
            used: &mut HashSet<String>,
            depth: usize,
            count: &mut usize,
        ) -> Result<(), String> {
            *count += 1;
            if depth > 16 || *count > 256 {
                return Err("dock layouts support 16 levels and 256 containers".into());
            }
            match node {
                DockNode::Split { children, .. } => {
                    for child in children {
                        if let Some(v) = child.size {
                            length(v)?;
                            if v == 0. {
                                return Err("split size must be positive when supplied".into());
                            }
                        }
                        visit(&child.layout, known, used, depth + 1, count)?;
                    }
                }
                DockNode::Tabs {
                    panes,
                    active_index,
                } => {
                    if *active_index >= panes.len().max(1) {
                        return Err("activeIndex is outside its tab group".into());
                    }
                    for p in panes {
                        pane(p, known, used)?;
                    }
                }
                DockNode::Tiles { panes } => {
                    for p in panes {
                        pane(&p.pane, known, used)?;
                        p.bounds.validate()?;
                    }
                }
            }
            Ok(())
        }
        let mut used = HashSet::new();
        let mut count = 0;
        visit(&self.center, known, &mut used, 0, &mut count)?;
        for (_, dock) in self.regions() {
            if let Some(dock) = dock {
                length(dock.size)?;
                visit(&dock.layout, known, &mut used, 0, &mut count)?;
            }
        }
        Ok(())
    }
    pub(super) fn native(
        &self,
        version: Option<u32>,
        data: &BTreeMap<String, DockValue>,
    ) -> DockAreaState {
        let region = |r: DockPlacement, v: &Option<DockRegionLayout>| {
            v.as_ref()
                .map(|v| DockState::new(v.layout.native(data), r, px(v.size), v.open))
        };
        DockAreaState {
            version: version.map(|v| v as usize),
            center: self.center.native(data),
            left_dock: region(DockPlacement::Left, &self.left),
            right_dock: region(DockPlacement::Right, &self.right),
            bottom_dock: region(DockPlacement::Bottom, &self.bottom),
        }
    }
}
fn leaf(pane: &str, data: &BTreeMap<String, DockValue>) -> PanelState {
    PanelState {
        panel_name: PANEL_NAME.into(),
        children: vec![],
        info: PanelInfo::panel(serde_json::json!({"pane":pane,"data":data.get(pane)})),
    }
}
impl DockNode {
    fn native(&self, data: &BTreeMap<String, DockValue>) -> PanelState {
        match self {
            Self::Split {
                orientation,
                children,
            } => PanelState {
                panel_name: "StackPanel".into(),
                children: children.iter().map(|c| c.layout.native(data)).collect(),
                info: PanelInfo::stack(
                    children.iter().map(|c| px(c.size.unwrap_or(0.))).collect(),
                    (*orientation).into(),
                ),
            },
            Self::Tabs {
                panes,
                active_index,
            } => PanelState {
                panel_name: "TabPanel".into(),
                children: panes.iter().map(|p| leaf(p, data)).collect(),
                info: PanelInfo::tabs(*active_index),
            },
            Self::Tiles { panes } => PanelState {
                panel_name: "Tiles".into(),
                children: panes.iter().map(|p| leaf(&p.pane, data)).collect(),
                info: PanelInfo::tiles(
                    panes
                        .iter()
                        .map(|p| TileMeta {
                            bounds: p.bounds.native(),
                            z_index: p.z_index as usize,
                        })
                        .collect(),
                ),
            },
        }
    }
    pub(super) fn from_native(
        v: &PanelState,
        data: &mut BTreeMap<String, DockValue>,
    ) -> Result<Self, String> {
        let mut read_leaf = |v: &PanelState| -> Result<String, String> {
            let PanelInfo::Panel(info) = &v.info else {
                return Err("dock leaf is not a panel".into());
            };
            if v.panel_name != PANEL_NAME {
                return Err("dock contains a panel outside this JS owner".into());
            }
            let pane = info
                .get("pane")
                .and_then(|v| v.as_str())
                .ok_or("dock panel is missing its stable name")?
                .to_owned();
            data.insert(
                pane.clone(),
                serde_json::from_value(info.get("data").cloned().unwrap_or_default())
                    .map_err(|e| e.to_string())?,
            );
            Ok(pane)
        };
        match &v.info {
            PanelInfo::Stack { sizes, axis } => Ok(Self::Split {
                orientation: if *axis == 0 {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                },
                children: v
                    .children
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        Ok(DockSplitChild {
                            layout: Self::from_native(c, data)?,
                            size: sizes.get(i).map(|v| v.as_f32()).filter(|v| *v > 0.),
                        })
                    })
                    .collect::<Result<_, String>>()?,
            }),
            PanelInfo::Tabs { active_index } => Ok(Self::Tabs {
                panes: v
                    .children
                    .iter()
                    .map(&mut read_leaf)
                    .collect::<Result<_, _>>()?,
                active_index: *active_index,
            }),
            PanelInfo::Tiles { metas } => Ok(Self::Tiles {
                panes: v
                    .children
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let meta = metas.get(i).ok_or("tile metadata missing")?;
                        Ok(DockTile {
                            pane: read_leaf(c)?,
                            bounds: meta.bounds.into(),
                            z_index: u32::try_from(meta.z_index)
                                .map_err(|_| "tile zIndex overflow")?,
                        })
                    })
                    .collect::<Result<_, String>>()?,
            }),
            PanelInfo::Panel(_) => Err("dock root is not a container".into()),
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct DockSnapshot {
    pub version: Option<u32>,
    pub layout: DockLayoutSpec,
    pub data: BTreeMap<String, DockValue>,
}

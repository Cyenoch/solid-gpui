//! Bounded native plot computations, executed through the shared native command pool.
use super::{
    charts::{FlowAlignment, FlowLink, FlowScale},
    plot::{coordinate, items, length},
};
use gpui_component::plot::{
    scale::{Scale, ScaleBand, ScaleLinear, ScaleOrdinal, ScalePoint},
    shape::{Arc, ArcData, Pie, Sankey, SankeyGraph, SankeyLinkLayout, SankeyNodeLayout, Stack},
};
use std::collections::{BTreeMap, HashMap, HashSet};
fn finite(v: f64) -> Result<(), String> {
    if v.is_finite() {
        Ok(())
    } else {
        Err("plot values and computed ranges must be finite".into())
    }
}
fn range(v: [f32; 2], ordered: bool) -> Result<(), String> {
    coordinate(v[0])?;
    coordinate(v[1])?;
    if ordered && v[0] > v[1] {
        return Err("categorical scale range must be ascending".into());
    }
    Ok(())
}
fn values(v: &[f64]) -> Result<(), String> {
    items(v.len())?;
    let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
    for v in v {
        finite(*v)?;
        low = low.min(*v);
        high = high.max(*v);
    }
    if !v.is_empty() {
        finite(high - low)?;
    }
    Ok(())
}
fn ticks(v: impl Iterator<Item = Option<f32>>) -> Result<Vec<Option<f32>>, String> {
    v.map(|v| {
        if let Some(v) = v {
            finite(v as f64)?;
        }
        Ok(v)
    })
    .collect()
}
fn one() -> f32 {
    1.
}
fn tau() -> f32 {
    std::f32::consts::TAU
}
fn width() -> f32 {
    24.
}
fn padding() -> f32 {
    8.
}
fn iterations() -> usize {
    6
}
#[crate::native_type]
pub struct LinearScaleRequest {
    pub domain: Vec<f64>,
    pub range: [f32; 2],
    pub values: Vec<f64>,
    #[serde(default)]
    pub cursor: Option<f32>,
}
#[crate::native_type]
pub struct ScaleNearest {
    pub index: usize,
    pub tick: f32,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct LinearScaleResult {
    pub ticks: Vec<Option<f32>>,
    pub nearest: Option<ScaleNearest>,
}
fn linear_scale(p: LinearScaleRequest) -> Result<LinearScaleResult, String> {
    values(&p.domain)?;
    values(&p.values)?;
    range(p.range, false)?;
    let s = ScaleLinear::new(p.domain.clone(), p.range.to_vec());
    // Native scale extrapolates. Reject overflow instead of serializing NaN/Infinity as null.
    let result = ticks(p.values.iter().map(|v| s.tick(v)))?;
    let domain_ticks = ticks(p.domain.iter().map(|v| s.tick(v)))?;
    let nearest = if let Some(cursor) = p.cursor {
        coordinate(cursor)?;
        if domain_ticks.iter().any(Option::is_some) {
            let (index, tick) = s.least_index_with_domain(cursor, &p.domain);
            Some(ScaleNearest { index, tick })
        } else {
            None
        }
    } else {
        None
    };
    Ok(LinearScaleResult {
        ticks: result,
        nearest,
    })
}
#[crate::native_type]
pub struct PointScaleRequest {
    pub domain: Vec<String>,
    pub range: [f32; 2],
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub cursor: Option<f32>,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct CategoricalScaleResult {
    pub domain: Vec<String>,
    pub domain_ticks: Vec<Option<f32>>,
    pub ticks: Vec<Option<f32>>,
    pub nearest: Option<ScaleNearest>,
    pub band_width: Option<f32>,
}
fn point_scale(p: PointScaleRequest) -> Result<CategoricalScaleResult, String> {
    items(p.domain.len() + p.values.len())?;
    range(p.range, true)?;
    let s = ScalePoint::new(p.domain.clone(), p.range.to_vec());
    let mut first = HashMap::new();
    for (i, v) in p.domain.iter().enumerate() {
        first.entry(v).or_insert(i);
    }
    let result = p
        .values
        .iter()
        .map(|v| first.get(v).and_then(|i| s.tick_at(*i)))
        .collect();
    let nearest = if let Some(cursor) = p.cursor {
        coordinate(cursor)?;
        if p.domain.is_empty() {
            None
        } else {
            let index = s.least_index(cursor);
            Some(ScaleNearest {
                index,
                tick: s.tick_at(index).expect("nonempty domain"),
            })
        }
    } else {
        None
    };
    Ok(CategoricalScaleResult {
        domain_ticks: (0..p.domain.len()).map(|i| s.tick_at(i)).collect(),
        domain: p.domain,
        ticks: result,
        nearest,
        band_width: None,
    })
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct BandScaleRequest {
    pub domain: Vec<String>,
    pub range: [f32; 2],
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub cursor: Option<f32>,
    #[serde(default)]
    pub padding_inner: f32,
    #[serde(default)]
    pub padding_outer: f32,
}
fn band_scale(p: BandScaleRequest) -> Result<CategoricalScaleResult, String> {
    items(p.domain.len() + p.values.len())?;
    range(p.range, true)?;
    for v in [p.padding_inner, p.padding_outer] {
        if !(0.0..=1.).contains(&v) {
            return Err("band padding must be between 0 and 1".into());
        }
    }
    let mut seen = HashSet::new();
    let domain = p
        .domain
        .into_iter()
        .filter(|v| seen.insert(v.clone()))
        .collect::<Vec<_>>();
    let s = ScaleBand::new(domain.clone(), p.range.to_vec())
        .padding_inner(p.padding_inner)
        .padding_outer(p.padding_outer);
    let nearest = if let Some(cursor) = p.cursor {
        coordinate(cursor)?;
        if domain.is_empty() {
            None
        } else {
            let index = s.least_index(cursor);
            Some(ScaleNearest {
                index,
                tick: s.tick(&domain[index]).expect("known category"),
            })
        }
    } else {
        None
    };
    Ok(CategoricalScaleResult {
        domain_ticks: domain.iter().map(|v| s.tick(v)).collect(),
        domain,
        ticks: p.values.iter().map(|v| s.tick(v)).collect(),
        nearest,
        band_width: Some(s.band_width()),
    })
}
#[crate::native_type]
pub struct OrdinalScaleRequest {
    pub domain: Vec<String>,
    pub range: Vec<String>,
    pub values: Vec<String>,
    #[serde(default)]
    pub unknown: Option<String>,
}
fn ordinal_scale(p: OrdinalScaleRequest) -> Result<Vec<Option<String>>, String> {
    items(p.range.len())?;
    items(p.domain.len())?;
    items(p.values.len())?;
    // Native ordinal accepts PartialEq domains and does a linear lookup per query.
    items(p.domain.len().saturating_mul(p.values.len()))?;
    let mut s = ScaleOrdinal::new(p.domain, p.range);
    if let Some(v) = p.unknown {
        s = s.unknown(v);
    }
    Ok(p.values.iter().map(|v| s.map(v)).collect())
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PieRequest {
    pub values: Vec<Option<f32>>,
    #[serde(default)]
    pub start_angle: f32,
    #[serde(default = "tau")]
    pub end_angle: f32,
    #[serde(default)]
    pub pad_angle: f32,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PlotArcData {
    pub index: usize,
    pub value: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub pad_angle: f32,
}
fn pie_arcs(p: PieRequest) -> Result<Vec<PlotArcData>, String> {
    items(p.values.len())?;
    coordinate(p.start_angle)?;
    coordinate(p.end_angle)?;
    length(p.pad_angle)?;
    let mut sum = 0f32;
    for v in p.values.iter().flatten() {
        finite(*v as f64)?;
        if *v < 0. {
            return Err("pie values must be nonnegative".into());
        }
        sum += v;
    }
    finite(sum as f64)?;
    Ok(Pie::new()
        .value(|v: &Option<f32>| *v)
        .start_angle(p.start_angle)
        .end_angle(p.end_angle)
        .pad_angle(p.pad_angle)
        .arcs(&p.values)
        .into_iter()
        .map(|v| PlotArcData {
            index: v.index,
            value: v.value,
            start_angle: v.start_angle,
            end_angle: v.end_angle,
            pad_angle: v.pad_angle,
        })
        .collect())
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct ArcCentroidRequest {
    pub start_angle: f32,
    pub end_angle: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
}
#[crate::native_type]
pub struct PlotCoordinate {
    pub x: f32,
    pub y: f32,
}
fn arc_centroid(p: ArcCentroidRequest) -> Result<PlotCoordinate, String> {
    coordinate(p.start_angle)?;
    coordinate(p.end_angle)?;
    length(p.inner_radius)?;
    length(p.outer_radius)?;
    if p.inner_radius > p.outer_radius {
        return Err("innerRadius must not exceed outerRadius".into());
    }
    let v = Arc::new()
        .inner_radius(p.inner_radius)
        .outer_radius(p.outer_radius)
        .centroid(&ArcData {
            data: &(),
            index: 0,
            value: 0.,
            start_angle: p.start_angle,
            end_angle: p.end_angle,
            pad_angle: 0.,
        });
    Ok(PlotCoordinate { x: v.x, y: v.y })
}
#[crate::native_type]
pub struct StackRequest {
    pub rows: Vec<BTreeMap<String, Option<f32>>>,
    pub keys: Vec<String>,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PlotStackPoint {
    pub row_index: usize,
    pub y0: f32,
    pub y1: f32,
}
#[crate::native_type]
pub struct PlotStackSeries {
    pub key: String,
    pub index: usize,
    pub points: Vec<PlotStackPoint>,
}
fn stack_series(p: StackRequest) -> Result<Vec<PlotStackSeries>, String> {
    items(p.rows.len())?;
    if p.keys.len() > 32 {
        return Err("stack supports at most 32 series".into());
    }
    items(p.rows.len().saturating_mul(p.keys.len()))?;
    let keys = p.keys.iter().collect::<HashSet<_>>();
    if keys.len() != p.keys.len() {
        return Err("stack keys must be unique".into());
    }
    let mut input_count = 0;
    for row in &p.rows {
        input_count += row.len();
        items(input_count)?;
        for v in row.values().flatten() {
            finite(*v as f64)?;
        }
        let mut sum = 0f32;
        for key in &p.keys {
            sum += row.get(key).copied().flatten().unwrap_or(0.);
            finite(sum as f64)?;
        }
    }
    // The native Stack uses zero for absent values. Keep rows once, carry their indices in the result.
    Ok(Stack::new()
        .data(0..p.rows.len())
        .keys(p.keys)
        .value(move |index: &usize, key| p.rows[*index].get(key).copied().flatten())
        .series()
        .into_iter()
        .map(|v| PlotStackSeries {
            key: v.key,
            index: v.index,
            points: v
                .points
                .into_iter()
                .map(|v| PlotStackPoint {
                    row_index: v.data,
                    y0: v.y0,
                    y1: v.y1,
                })
                .collect(),
        })
        .collect())
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct SankeyLayoutRequest {
    pub node_count: usize,
    pub links: Vec<FlowLink>,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "one")]
    pub width: f32,
    #[serde(default = "one")]
    pub height: f32,
    #[serde(default = "width")]
    pub node_width: f32,
    #[serde(default = "padding")]
    pub node_padding: f32,
    #[serde(default)]
    pub alignment: FlowAlignment,
    #[serde(default = "iterations")]
    pub iterations: usize,
    #[serde(default)]
    pub value_scale: FlowScale,
    #[serde(default)]
    pub topology_only: bool,
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PlotSankeyNode {
    pub index: usize,
    pub value: f64,
    pub depth: usize,
    pub height: usize,
    pub layer: usize,
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub source_links: Vec<usize>,
    pub target_links: Vec<usize>,
}
impl From<SankeyNodeLayout> for PlotSankeyNode {
    fn from(n: SankeyNodeLayout) -> Self {
        Self {
            index: n.index,
            value: n.value,
            depth: n.depth,
            height: n.height,
            layer: n.layer,
            x0: n.x0,
            x1: n.x1,
            y0: n.y0,
            y1: n.y1,
            source_links: n.source_links,
            target_links: n.target_links,
        }
    }
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PlotSankeyLink {
    pub index: usize,
    pub source: usize,
    pub target: usize,
    pub value: f64,
    pub y0: f32,
    pub y1: f32,
    pub width: f32,
    pub source_width: f32,
    pub target_width: f32,
}
impl From<SankeyLinkLayout> for PlotSankeyLink {
    fn from(n: SankeyLinkLayout) -> Self {
        Self {
            index: n.index,
            source: n.source,
            target: n.target,
            value: n.value,
            y0: n.y0,
            y1: n.y1,
            width: n.width,
            source_width: n.source_width,
            target_width: n.target_width,
        }
    }
}
#[crate::native_type]
#[serde(rename_all = "camelCase")]
pub struct PlotSankeyGraph {
    pub nodes: Vec<PlotSankeyNode>,
    pub links: Vec<PlotSankeyLink>,
    pub layer_count: usize,
}
fn sankey_layout(p: SankeyLayoutRequest) -> Result<PlotSankeyGraph, String> {
    if p.node_count > 512 || p.links.len() > 4096 || p.iterations > 32 {
        return Err("Sankey supports at most 512 nodes, 4096 links and 32 iterations".into());
    }
    coordinate(p.x)?;
    coordinate(p.y)?;
    for v in [p.width, p.height, p.node_width, p.node_padding] {
        length(v)?;
    }
    let mut sum = 0.;
    for v in &p.links {
        finite(v.value)?;
        if v.value < 0. {
            return Err("Sankey values must be nonnegative".into());
        }
        sum += v.value;
    }
    finite(sum)?;
    let native = Sankey::new()
        .extent(p.x, p.y, p.x + p.width, p.y + p.height)
        .node_width(p.node_width)
        .node_padding(p.node_padding)
        .node_align(p.alignment.into())
        .iterations(p.iterations)
        .value_scale(p.value_scale.into());
    let graph = native
        .topology(
            p.node_count,
            &p.links.iter().map(Into::into).collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;
    let graph: SankeyGraph = if p.topology_only {
        graph
    } else {
        native.layout_from(graph)
    };
    Ok(PlotSankeyGraph {
        layer_count: graph.layer_count(),
        nodes: graph.nodes.into_iter().map(Into::into).collect(),
        links: graph.links.into_iter().map(Into::into).collect(),
    })
}
pub(super) fn native_module() -> crate::native::ModuleDefinition {
    use crate::native::{CommandDefinition as C, ModuleDefinition};
    ModuleDefinition::new(
        "gpui-component",
        vec![],
        vec![
            C::sync("scaleLinear", |request, _context| linear_scale(request)),
            C::sync("scalePoint", |request, _context| point_scale(request)),
            C::sync("scaleBand", |request, _context| band_scale(request)),
            C::sync("scaleOrdinal", |request, _context| ordinal_scale(request)),
            C::sync("pieArcs", |request, _context| pie_arcs(request)),
            C::sync("arcCentroid", |request, _context| arc_centroid(request)),
            C::sync("stackSeries", |request, _context| stack_series(request)),
            C::sync("sankeyLayout", |request, _context| sankey_layout(request)),
        ],
    )
    .with_contract(include_str!("plot_math.rs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p<T: serde::de::DeserializeOwned>(v: &str) -> T {
        crate::native::decode_json(v.as_bytes()).unwrap()
    }
    #[test]
    fn scale_coordinates_preserve_offsets_duplicates_and_reversed_numeric_ranges() {
        let v = linear_scale(p(
            r#"{"domain":[0,5,10],"range":[200,100],"values":[0,5,10],"cursor":149}"#,
        ))
        .unwrap();
        assert_eq!(v.ticks, vec![Some(200.), Some(150.), Some(100.)]);
        assert_eq!(v.nearest.unwrap().index, 1);
        let v = point_scale(p(
            r#"{"domain":["a","a","b"],"range":[40,80],"values":["a","missing"],"cursor":60}"#,
        ))
        .unwrap();
        assert_eq!(v.domain_ticks, vec![Some(40.), Some(60.), Some(80.)]);
        assert_eq!(v.ticks, vec![Some(40.), None]);
        assert_eq!(v.nearest.unwrap().index, 1);
        let v = band_scale(p(
            r#"{"domain":["a","a","b"],"range":[40,100],"values":["a","b"],"cursor":72}"#,
        ))
        .unwrap();
        assert_eq!(v.domain, vec!["a", "b"]);
        assert_eq!(v.ticks, vec![Some(40.), Some(70.)]);
        assert_eq!(v.nearest.unwrap().index, 1);
        assert_eq!(v.band_width, Some(30.));
        assert!(linear_scale(p(r#"{"domain":[0,1e-300],"range":[0,100],"values":[1]}"#)).is_err());
    }
    #[test]
    fn geometry_preserves_source_indices_and_rejects_invalid_graphs_and_work() {
        let arcs = pie_arcs(p(r#"{"values":[0,1,null,3]}"#)).unwrap();
        assert_eq!(arcs.iter().map(|a| a.index).collect::<Vec<_>>(), vec![1, 3]);
        assert!((arcs[0].end_angle - std::f32::consts::FRAC_PI_2).abs() < 0.001);
        let stack = stack_series(p(
            r#"{"rows":[{"a":2,"b":3},{"a":null,"b":4}],"keys":["a","b"]}"#,
        ))
        .unwrap();
        assert_eq!(stack[1].points[0].row_index, 0);
        assert_eq!(stack[1].points[0].y0, 2.);
        assert_eq!(stack[1].points[0].y1, 5.);
        assert_eq!(stack[1].points[1].y0, 0.);
        let graph = sankey_layout(p(r#"{"nodeCount":2,"links":[{"source":0,"target":1,"value":16}],"x":20,"y":30,"width":300,"height":200,"valueScale":"sqrt"}"#)).unwrap();
        assert_eq!(graph.layer_count, 2);
        assert_eq!(graph.nodes[0].value, 4.);
        assert_eq!(graph.nodes[0].x0, 20.);
        assert!(graph.links[0].width.is_finite());
        assert!(sankey_layout(p(r#"{"nodeCount":2,"links":[{"source":0,"target":1,"value":1},{"source":1,"target":0,"value":1}]}"#)).err().unwrap().contains("circular"));
        assert!(
            sankey_layout(p(
                r#"{"nodeCount":1,"links":[{"source":0,"target":1,"value":1}]}"#
            ))
            .err()
            .unwrap()
            .contains("missing node: 1")
        );
        assert!(
            stack_series(StackRequest {
                rows: vec![BTreeMap::new(); 1024],
                keys: (0..32).map(|n| n.to_string()).collect()
            })
            .is_err()
        );
    }
}

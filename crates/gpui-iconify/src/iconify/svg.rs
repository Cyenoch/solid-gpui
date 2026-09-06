use super::model::ResolvedIcon;
use super::resolve::normalize_rotate;

/// Wraps a resolved icon as a complete SVG document (no baked size/color).
pub fn icon_to_svg(icon: &ResolvedIcon) -> String {
    let mut box_ = ViewBox {
        left: icon.left,
        top: icon.top,
        width: icon.width,
        height: icon.height,
    };
    let body = apply_transforms(&icon.body, &mut box_, icon.h_flip, icon.v_flip, icon.rotate);
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">{}</svg>"#,
        box_.attr(),
        body
    )
}

/// `lucide:triangle-alert` → `LucideTriangleAlert`.
pub fn variant_ident(prefix: &str, name: &str) -> String {
    format!("{}{}", pascal_case(prefix), pascal_case(name))
}

#[derive(Clone, Copy)]
struct ViewBox {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl ViewBox {
    fn attr(self) -> String {
        format!(
            "{} {} {} {}",
            fmt_num(self.left),
            fmt_num(self.top),
            fmt_num(self.width),
            fmt_num(self.height)
        )
    }
}

fn apply_transforms(
    body: &str,
    box_: &mut ViewBox,
    h_flip: bool,
    v_flip: bool,
    rotate: i32,
) -> String {
    let mut transformations = Vec::new();
    let mut rotation = rotate;
    if h_flip {
        if v_flip {
            rotation += 2;
        } else {
            transformations.push(format!(
                "translate({} {})",
                fmt_num(box_.width + box_.left),
                fmt_num(0.0 - box_.top)
            ));
            transformations.push("scale(-1 1)".to_string());
            box_.top = 0.0;
            box_.left = 0.0;
        }
    } else if v_flip {
        transformations.push(format!(
            "translate({} {})",
            fmt_num(0.0 - box_.left),
            fmt_num(box_.height + box_.top)
        ));
        transformations.push("scale(1 -1)".to_string());
        box_.top = 0.0;
        box_.left = 0.0;
    }
    rotation = normalize_rotate(rotation);
    push_rotation(&mut transformations, box_, rotation);
    if rotation % 2 == 1 {
        std::mem::swap(&mut box_.left, &mut box_.top);
        std::mem::swap(&mut box_.width, &mut box_.height);
    }
    if transformations.is_empty() {
        return body.to_string();
    }
    format!("<g transform=\"{}\">{body}</g>", transformations.join(" "))
}

fn push_rotation(transformations: &mut Vec<String>, box_: &ViewBox, rotation: i32) {
    match rotation {
        1 => {
            let pivot = box_.height / 2.0 + box_.top;
            transformations.insert(
                0,
                format!("rotate(90 {} {})", fmt_num(pivot), fmt_num(pivot)),
            );
        }
        2 => {
            transformations.insert(
                0,
                format!(
                    "rotate(180 {} {})",
                    fmt_num(box_.width / 2.0 + box_.left),
                    fmt_num(box_.height / 2.0 + box_.top)
                ),
            );
        }
        3 => {
            let pivot = box_.width / 2.0 + box_.left;
            transformations.insert(
                0,
                format!("rotate(-90 {} {})", fmt_num(pivot), fmt_num(pivot)),
            );
        }
        _ => {}
    }
}

fn fmt_num(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn pascal_case(value: &str) -> String {
    value
        .split(['-', '_', ':'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

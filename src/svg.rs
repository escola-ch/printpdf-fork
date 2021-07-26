use std::borrow::BorrowMut;

use usvg::{LinearGradient, PathSegment, Tree, prelude::*};
use lopdf::content::Operation;
use lopdf::Object;

use crate::{ PdfLayerReference, LineCapStyle };

// #[derive(Copy, Clone, Debug)]
// pub enum SvgSizeConstraint {
//     Width(f64),
//     Height(f64),
// }

pub fn draw_svg(
    layer: &PdfLayerReference,
    tree: &Tree,
    // constraint: SvgSizeConstraint,
    // x: f64, y: f64,
) {
    let svg = tree.svg_node();

    let root = &tree.root();
    draw_node(layer, root, tree);
}

fn draw_node(
    layer: &PdfLayerReference,
    node: &usvg::Node,
    tree: &Tree,
) -> Option<[f64; 2]> {
    match *node.borrow() {
        usvg::NodeKind::Svg(svg) => {
            use lopdf::Object::Real;

            layer.save_graphics_state();

            let view_box_rect = svg.view_box.rect;

            layer.add_op(Operation::new("cm", vec![
                Real(1.),
                Real(0.),
                Real(0.),
                Real(1.),
                Real(-view_box_rect.x()),
                Real(-view_box_rect.y()),
            ]));

            let ret = draw_group(layer, node, tree);
            layer.restore_graphics_state();
            ret
        }
        usvg::NodeKind::Path(ref path) => draw_path(layer, path, tree),
        usvg::NodeKind::Group(ref g) => {
            layer.save_graphics_state();
            apply_transform(&layer, g.transform);
            let ret = draw_group(layer, node, tree);
            layer.restore_graphics_state();
            ret
        }
        _ => None,
    }
}

fn draw_group(
    layer: &PdfLayerReference,
    parent: &usvg::Node,
    tree: &Tree,
) -> Option<[f64; 2]> {
    for node in parent.children() {
        draw_node(layer, &node, tree);
    }
    None
}

fn apply_transform(layer: &PdfLayerReference, transform: svgtypes::Transform) {
    use lopdf::Object::Real;

    layer.add_op(Operation::new("cm", vec![
        Real(transform.a),
        Real(transform.b),
        Real(transform.c),
        Real(transform.d),
        Real(transform.e),
        Real(transform.f),
    ]));
}

fn color(c: svgtypes::Color) -> Vec<Object> {
    return vec![
        (c.red as f64 / 255.).into(),
        (c.green as f64 / 255.).into(),
        (c.blue as f64 / 255.).into(),
    ];
}

fn linear_gradient(layer: &PdfLayerReference, lg: &LinearGradient) -> lopdf::ObjectId {
    use lopdf::dictionary;

    let stops = &lg.base.stops;

    layer.add_object(dictionary!(
        "Type" => "Pattern",
        "PatternType" => 2, // shading pattern
        "Matrix" => {
            let t = lg.base.transform;
            vec![t.a.into(), t.b.into(), t.c.into(), t.d.into(), t.e.into(), t.f.into()]
        },
        "Shading" => layer.add_object(dictionary!(
            "ShadingType" => 2, // axial shading
            "ColorSpace" => "DeviceRGB", // idk
            "Coords" => vec![
                // x0 y0 x1 y1
                lg.x1.into(),
                lg.y1.into(),
                lg.x2.into(),
                lg.y2.into(),
            ],

            // domain is implicitly [0, 1]

            // TODO: Spread Method

            // TODO
            "Function" => layer.add_object(dictionary!(
                "FunctionType" => 3, // Sampled Function
                "Domain" => vec![0.into(), 1.into()],

                "Functions" => stops
                    .windows(2)
                    .map(|w| layer.add_object(dictionary!(
                        "FunctionType" => 2,
                        "Domain" => vec![0.into(), 1.into()],
                        "C0" => color(w[0].color),
                        "C1" => color(w[1].color),
                        "N" => 1, // idk
                    )).into())
                    .collect::<Vec<Object>>(),
                "Bounds" => stops[1..stops.len() - 1].iter()
                    .map(|s| s.offset.value().into())
                    .collect::<Vec<Object>>(),
                "Encode" => [Object::Integer(0), Object::Integer(1)]
                    .iter()
                    .cloned()
                    .cycle()
                    .take(2 * stops.len().checked_sub(1).unwrap_or(0))
                    .collect::<Vec<Object>>(),
            )),
        )),
    ))
}

fn linear_gradient_shading(layer: &PdfLayerReference, lg: &LinearGradient) -> lopdf::Dictionary {
    use lopdf::dictionary;

    let stops = &lg.base.stops;

    dictionary!(
        "ShadingType" => 2, // axial shading
        "ColorSpace" => "DeviceRGB", // idk
        "Coords" => vec![
            // x0 y0 x1 y1
            lg.x1.into(),
            lg.y1.into(),
            lg.x2.into(),
            lg.y2.into(),
        ],

        "Extend" => vec![true.into(), true.into()],

        // domain is implicitly [0, 1]

        // TODO: Spread Method

        // TODO
        "Function" => dictionary!(
            "FunctionType" => 3, // Sampled Function
            "Domain" => vec![0.into(), 1.into()],

            "Functions" => stops
                .windows(2)
                .map(|w| dictionary!(
                    "FunctionType" => 2,
                    "Domain" => vec![0.into(), 1.into()],
                    "C0" => color(w[0].color),
                    "C1" => color(w[1].color),
                    "N" => 1, // idk
                ).into())
                .collect::<Vec<Object>>(),
            "Bounds" => stops[1..stops.len() - 1].iter()
                .map(|s| s.offset.value().into())
                .collect::<Vec<Object>>(),
            "Encode" => [Object::Integer(0), Object::Integer(1)]
                .iter()
                .cloned()
                .cycle()
                .take(2 * stops.len().checked_sub(1).unwrap_or(0))
                .collect::<Vec<Object>>(),
        ),
    )
}

fn draw_path(layer: &PdfLayerReference, path: &usvg::Path, tree: &Tree) -> Option<[f64; 2]> {
    layer.save_graphics_state();
    if let Some(usvg::Fill { paint: usvg::Paint::Color(color), ref opacity, ref rule }) = path.fill {
        layer.set_fill_color(crate::Color::Rgb(crate::Rgb::new(
            color.red as f64 / 255.0,
            color.green as f64 / 255.0,
            color.blue as f64 / 255.0,
            None,
        )));

        layer.set_fill_alpha(opacity.value());
    }

    // match path.fill {
    //     Some(usvg::Fill { paint: usvg::Paint::Color(color), ref opacity, ref rule }) => {
    //         layer.set_fill_color(crate::Color::Rgb(crate::Rgb::new(
    //             color.red as f64 / 255.0,
    //             color.green as f64 / 255.0,
    //             color.blue as f64 / 255.0,
    //             None,
    //         )));

    //         layer.set_fill_alpha(opacity.value());
    //     }
    //     Some(usvg::Fill { paint: usvg::Paint::Link(ref id), ref opacity, ref rule }) => {
    //         if let Some(node) = tree.defs_by_id(id) {
    //             match *node.borrow() {
    //                 usvg::NodeKind::LinearGradient(ref lg) => {
    //                     // not sure in what case a pattern needs color space parameters
    //                     layer.add_op(Operation::new("cs", vec!["Pattern".into()]));
    //                     layer.add_op(Operation::new(
    //                         "scn",
    //                         vec![Object::Name(layer.add_pattern(linear_gradient(lg)))]),
    //                     );
    //                 }
    //                 usvg::NodeKind::RadialGradient(ref rg) => (),
    //                 usvg::NodeKind::Pattern(ref pattern) => (),
    //                 _ => (),
    //             }
    //         }
    //     }
    //     _ => (), // just do nothing
    // }

    if let Some(ref stroke) = path.stroke {
        let dash_array = stroke.dasharray.as_deref().unwrap_or(&[]);
        let dash_phase = stroke.dashoffset;
        layer.add_op(Operation::new("d", vec![
            Object::Array(dash_array.iter().map(|d| Object::Integer(*d as i64)).collect()),
            Object::Integer(dash_phase as i64),
        ]));
        layer.set_outline_thickness(stroke.width.value());

        layer.set_line_cap_style(match stroke.linecap {
            usvg::LineCap::Butt => LineCapStyle::Butt,
            usvg::LineCap::Round => LineCapStyle::Round,
            usvg::LineCap::Square => LineCapStyle::ProjectingSquare,
        });

        if let usvg::Paint::Color(color) = stroke.paint {
            layer.set_outline_color(crate::Color::Rgb(crate::Rgb::new(
                color.red as f64 / 255.0,
                color.green as f64 / 255.0,
                color.blue as f64 / 255.0,
                None,
            )));
        }
    }

    let mut ops = Vec::new();

    let mut closed = false;

    apply_transform(&layer, path.transform);

    for s in path.data.iter() {
        match s {
            &PathSegment::MoveTo { x, y } => ops.push(Operation::new("m", vec![x.into(), y.into()])),
            &PathSegment::LineTo { x, y } => ops.push(Operation::new("l", vec![x.into(), y.into()])),
            &PathSegment::CurveTo { x1, y1, x2, y2, x, y } => ops.push(Operation::new("c", vec![
                x1.into(), y1.into(),
                x2.into(), y2.into(),
                x.into(), y.into(),
            ])),
            &PathSegment::ClosePath => closed = true,
        }
    }

    if let Some(usvg::Fill { paint: usvg::Paint::Link(ref id), ref opacity, ref rule }) = path.fill {
        if let Some(node) = tree.defs_by_id(id) {
            match *node.borrow() {
                usvg::NodeKind::LinearGradient(ref lg) => {
                    // not sure in what case a pattern needs color space parameters
                    // layer.add_op(Operation::new("cs", vec!["Pattern".into()]));
                    // layer.add_op(Operation::new(
                    //     "scn",
                    //     vec![Object::Name(layer.add_pattern(linear_gradient(lg)))]),
                    // );
                    let pattern = layer.add_pattern(linear_gradient_shading(layer, lg));
                    ops.push(Operation::new("h", Vec::new()));
                    ops.push(Operation::new("W", Vec::new()));
                    ops.push(Operation::new("n", Vec::new()));
                    ops.push(Operation::new("sh", vec![Object::Name(pattern)]));
                    layer.add_ops(ops);
                }
                usvg::NodeKind::RadialGradient(ref rg) => (),
                usvg::NodeKind::Pattern(ref pattern) => (),
                _ => (),
            }
        }
    } else {
        // TODO: Check fill and stroke combination
        match (path.stroke.is_some(), path.fill.is_some(), closed) {
            (true, true, true) => ops.push(Operation::new("b", Vec::new())),
            (true, true, false) => ops.push(Operation::new("f", Vec::new())),
            (true, false, true) => ops.push(Operation::new("s", Vec::new())),
            (true, false, false) => ops.push(Operation::new("S", Vec::new())),
            (false, true, _) => ops.push(Operation::new("f", Vec::new())),
            _ => ops.push(Operation::new("n", Vec::new())),
        }
        layer.add_ops(ops);
    }

    layer.restore_graphics_state();

    None
}

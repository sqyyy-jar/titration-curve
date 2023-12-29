#![allow(unused)]

use svg::{
    node::element::{
        tag::LinearGradient, Circle, Definitions, Line, LinearGradient, Polyline, Rectangle, Stop,
        Style, Text,
    },
    Document, Node,
};

use crate::worker::Output;

use super::options::Options;

const DIAGRAM_FRAME_WIDTH: f32 = 400.0;
const DIAGRAM_FRAME_HEIGHT: f32 = 300.0;
const DIAGRAM_MARGIN: f32 = 20.0;
const DIAGRAM_WIDTH: f32 = DIAGRAM_FRAME_WIDTH - 2.0 * DIAGRAM_MARGIN;
const DIAGRAM_HEIGHT: f32 = DIAGRAM_FRAME_HEIGHT - 2.0 * DIAGRAM_MARGIN;
const DIAGRAM_TOP: f32 = DIAGRAM_MARGIN;
const DIAGRAM_BOTTOM: f32 = DIAGRAM_TOP + DIAGRAM_HEIGHT;
const DIAGRAM_LEFT: f32 = DIAGRAM_MARGIN;
const DIAGRAM_RIGHT: f32 = DIAGRAM_LEFT + DIAGRAM_WIDTH;
const DIAGRAM_X_GAPS: f32 = 5.0;
/// Maximum pH
const DIAGRAM_MAX_Y: f32 = 14.0;

const STYLE_LIGHT: &str = include_str!("style/light.css");
const STYLE_DARK: &str = include_str!("style/dark.css");

pub fn render_graph(options: &Options, output: &Output) -> String {
    diagram(options, output).to_string()
}

fn diagram(options: &Options, output: &Output) -> impl Node {
    let max_m_v = output.max_m_v();
    let x_steps = (max_m_v / DIAGRAM_X_GAPS).ceil() as usize;
    let scale = (
        DIAGRAM_WIDTH / DIAGRAM_X_GAPS / x_steps as f32,
        DIAGRAM_HEIGHT / DIAGRAM_MAX_Y,
    );
    let mut doc = Document::new()
        .set(
            "viewBox",
            format!("0 0 {DIAGRAM_FRAME_WIDTH} {DIAGRAM_FRAME_HEIGHT}"),
        )
        .add(style(options));
    diagram_frame(options, &mut doc, x_steps);
    diagram_graph(options, output, &mut doc, scale);
    doc
}

fn style(options: &Options) -> Style {
    Style::new(if options.dark {
        STYLE_DARK
    } else {
        STYLE_LIGHT
    })
}

fn diagram_frame(options: &Options, doc: &mut Document, x_steps: usize) {
    if options.colored {
        colored_background(doc);
    }
    // y-Axis
    for ph in 0..=14 {
        let y = DIAGRAM_BOTTOM - DIAGRAM_HEIGHT / DIAGRAM_MAX_Y * ph as f32;
        doc.append(
            Line::new()
                .set("class", "grid")
                .set("x1", DIAGRAM_LEFT)
                .set("y1", y)
                .set("x2", DIAGRAM_RIGHT)
                .set("y2", y),
        );
        doc.append(
            Line::new()
                .set("class", "axis")
                .set("x1", DIAGRAM_LEFT - 3.0)
                .set("y1", y)
                .set("x2", DIAGRAM_LEFT + 3.0)
                .set("y2", y),
        );
        doc.append(
            Text::new()
                .set("class", "axis-number anchor-end")
                .set("x", DIAGRAM_LEFT - 5.0)
                .set("y", y)
                .add(text(ph.to_string())),
        );
    }
    // x-Axis
    for step in 0..=x_steps {
        let x = DIAGRAM_MARGIN + DIAGRAM_WIDTH / x_steps as f32 * step as f32;
        doc.append(
            Line::new()
                .set("class", "grid")
                .set("x1", x)
                .set("y1", DIAGRAM_BOTTOM)
                .set("x2", x)
                .set("y2", DIAGRAM_TOP),
        );
        doc.append(
            Line::new()
                .set("class", "axis")
                .set("x1", x)
                .set("y1", DIAGRAM_BOTTOM - 3.0)
                .set("x2", x)
                .set("y2", DIAGRAM_BOTTOM + 3.0),
        );
        doc.append(
            Text::new()
                .set("class", "axis-number anchor-middle")
                .set("x", x)
                .set("y", DIAGRAM_BOTTOM + 10.0)
                .add(text((step as f32 * DIAGRAM_X_GAPS).to_string())),
        );
    }
    doc.append(
        Polyline::new()
            .set("class", "axis")
            .set("points", format!("{DIAGRAM_MARGIN},{DIAGRAM_MARGIN} {DIAGRAM_MARGIN},{DIAGRAM_BOTTOM} {DIAGRAM_RIGHT},{DIAGRAM_BOTTOM}"))
    );
    doc.append(
        Text::new()
            .set("class", "text anchor-middle")
            .set("x", DIAGRAM_RIGHT + 10.0)
            .set("y", DIAGRAM_TOP + DIAGRAM_HEIGHT / 2.0)
            .add(text("pH")),
    );
    doc.append(
        Text::new()
            .set("class", "text anchor-middle")
            .set("x", DIAGRAM_LEFT + DIAGRAM_WIDTH / 2.0)
            .set("y", DIAGRAM_TOP - 10.0)
            .add(text("Volumen")),
    );
}

fn diagram_graph(
    options: &Options,
    output: &Output,
    doc: &mut Document,
    (scale_x, scale_y): (f32, f32),
) {
    // Lines
    for items in output.items.windows(2) {
        doc.append(
            Line::new()
                .set("class", "graph-line")
                .set("x1", DIAGRAM_LEFT + items[0].m_v * scale_x)
                .set("y1", DIAGRAM_BOTTOM - items[0].ph * scale_y)
                .set("x2", DIAGRAM_LEFT + items[1].m_v * scale_x)
                .set("y2", DIAGRAM_BOTTOM - items[1].ph * scale_y),
        );
    }
    // Points
    for item in &output.items {
        doc.append(
            Circle::new()
                .set("class", "graph-point")
                .set("cx", DIAGRAM_LEFT + item.m_v * scale_x)
                .set("cy", DIAGRAM_BOTTOM - item.ph * scale_y),
        );
    }
}

fn colored_background(doc: &mut Document) {
    color_gradient(doc);
    doc.append(
        Rectangle::new()
            .set("fill", "url(#color-gradient)")
            .set("x", DIAGRAM_LEFT)
            .set("y", DIAGRAM_TOP)
            .set("width", DIAGRAM_WIDTH)
            .set("height", DIAGRAM_HEIGHT),
    );
}

fn color_gradient(doc: &mut Document) {
    doc.append(
        Definitions::new().add(
            LinearGradient::new()
                .set("id", "color-gradient")
                .set("x1", 0)
                .set("y1", 1)
                .set("x2", 0)
                .set("y2", 0)
                .add(Stop::new().set("stop-color", "red").set("offset", "0%"))
                .add(Stop::new().set("stop-color", "yellow").set("offset", "50%"))
                .add(Stop::new().set("stop-color", "green").set("offset", "100%")),
        ),
    )
}

fn text(content: impl Into<String>) -> svg::node::Text {
    svg::node::Text::new(content)
}

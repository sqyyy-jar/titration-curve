use std::rc::Rc;

use dioxus::prelude::*;

use crate::curve::Output;

const DIAGRAM_FRAME_WIDTH: f64 = 400.0;
const DIAGRAM_FRAME_HEIGHT: f64 = 300.0;
const DIAGRAM_MARGIN: f64 = 50.0;
const DIAGRAM_WIDTH: f64 = DIAGRAM_FRAME_WIDTH - 2.0 * DIAGRAM_MARGIN;
const DIAGRAM_HEIGHT: f64 = DIAGRAM_FRAME_HEIGHT - 2.0 * DIAGRAM_MARGIN;
const DIAGRAM_TOP: f64 = DIAGRAM_MARGIN;
const DIAGRAM_BOTTOM: f64 = DIAGRAM_TOP + DIAGRAM_HEIGHT;
const DIAGRAM_LEFT: f64 = DIAGRAM_MARGIN;
const DIAGRAM_RIGHT: f64 = DIAGRAM_LEFT + DIAGRAM_WIDTH;
const DIAGRAM_X_GAPS: f64 = 5.0;
/// Maximum pH
const DIAGRAM_X_MAX: f64 = 14.0;

#[component]
pub fn Diagram(cx: Scope, data: Rc<Output>) -> Element {
    let max = data.max_v();
    let x_steps = (max / DIAGRAM_X_GAPS).ceil() as usize;
    let scale = (
        DIAGRAM_WIDTH / DIAGRAM_X_GAPS / x_steps as f64,
        DIAGRAM_HEIGHT / DIAGRAM_X_MAX,
    );
    render! {
        svg {
            class: "diagram",
            view_box: "0 0 {DIAGRAM_FRAME_WIDTH} {DIAGRAM_FRAME_HEIGHT}",
            DiagramFrame { x_steps: x_steps }
            DiagramGraph { data: data.clone(), scale: scale }
        }
    }
}

#[component]
fn DiagramFrame(cx: Scope, x_steps: usize) -> Element {
    render! {
        // y-Axis
        (0..=14).map(|ph| {
            let y = DIAGRAM_BOTTOM - DIAGRAM_HEIGHT / DIAGRAM_X_MAX * ph as f64;
            rsx! {
                line {
                    class: "diagram-grid",
                    x1: DIAGRAM_LEFT,
                    y1: y,
                    x2: DIAGRAM_RIGHT,
                    y2: y,
                }
                line {
                    class: "diagram-axis",
                    x1: DIAGRAM_LEFT - 3.0,
                    y1: y,
                    x2: DIAGRAM_LEFT + 3.0,
                    y2: y,
                }
                text {
                    class: "diagram-axis-number anchor-end",
                    x: DIAGRAM_LEFT - 5.0,
                    y: y,
                    "{ph}"
                }
            }
        }),
        // x-Axis
        (0..=*x_steps).map(|v| {
            let x = DIAGRAM_MARGIN + DIAGRAM_WIDTH / *x_steps as f64 * v as f64;
            rsx! {
                line {
                    class: "diagram-grid",
                    x1: x,
                    y1: DIAGRAM_BOTTOM,
                    x2: x,
                    y2: DIAGRAM_TOP,
                }
                line {
                    class: "diagram-axis",
                    x1: x,
                    y1: DIAGRAM_BOTTOM - 3.0,
                    x2: x,
                    y2: DIAGRAM_BOTTOM + 3.0,
                }
                text {
                    class: "diagram-axis-number anchor-middle",
                    x: x,
                    y: DIAGRAM_BOTTOM + 10.0, // Optimal offset - do not change
                    "{v as f64 * DIAGRAM_X_GAPS}"
                }
            }
        }),
        polyline {
            class: "diagram-axis",
            points: "{DIAGRAM_MARGIN},{DIAGRAM_MARGIN} {DIAGRAM_MARGIN},{DIAGRAM_BOTTOM} {DIAGRAM_RIGHT},{DIAGRAM_BOTTOM}",
        }
        text {
            class: "diagram-axis-number anchor-middle", // todo
            x: DIAGRAM_RIGHT + 10.0,
            y: DIAGRAM_TOP + DIAGRAM_HEIGHT / 2.0,
            "pH"
        }
        text {
            class: "diagram-axis-number anchor-middle", // todo
            x: DIAGRAM_LEFT + DIAGRAM_WIDTH / 2.0,
            y: DIAGRAM_TOP - 10.0,
            "Volumen"
        }
    }
}

#[component]
fn DiagramGraph(cx: Scope, data: Rc<Output>, scale: (f64, f64)) -> Element {
    render! {
        // Draw lines between points
        for (phs, vs) in data.ph.windows(2).zip(data.v_total.windows(2)) {
            line {
                class: "diagram-line",
                x1: DIAGRAM_LEFT + vs[0] * scale.0,
                y1: DIAGRAM_BOTTOM - phs[0] * scale.1,
                x2: DIAGRAM_LEFT + vs[1] * scale.0,
                y2: DIAGRAM_BOTTOM - phs[1] * scale.1,
            }
        }
        // Points
        for (ph, v) in data.ph.iter().zip(data.v_total.iter()) {
            circle {
                class: "diagram-point",
                cx: DIAGRAM_LEFT + v * scale.0,
                cy: DIAGRAM_BOTTOM - ph * scale.1,
            }
        }
    }
}

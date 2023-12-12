#![allow(non_snake_case)]

pub mod curve;

use std::rc::Rc;

use calamine::{open_workbook, Reader, Xlsx};
use curve::Output;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

const CSS_BASE: &str = include_str!("styles/base.css");
const CSS_THEMES: &[&str] = &[
    include_str!("styles/theme/light.css"),
    include_str!("styles/theme/dark.css"),
    include_str!("styles/theme/colored.css"),
];

fn main() {
    // test();
    dioxus_desktop::launch_cfg(
        App,
        Config::new().with_window(WindowBuilder::new().with_title("Titrationskurve")),
    );
}

fn _test() {
    let mut workbook: Xlsx<_> = open_workbook("table.xlsx").unwrap();
    let range = workbook.worksheet_range_at(0).unwrap().unwrap();
    println!("{range:?}");
}

fn App(cx: Scope) -> Element {
    let output = Rc::new(Output {
        v_total: vec![
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.1, 9.2, 9.3,
            9.4, 9.5, 9.6, 9.7, 9.8, 9.9, 9.91, 9.92, 9.93, 9.94, 9.95, 9.96, 9.97, 9.98, 9.99,
            10.0, 20.0,
        ],
        ph: vec![
            1.00, 1.09, 1.18, 1.27, 1.37, 1.48, 1.54, 1.60, 1.67, 1.75, 1.85, 1.95, 2.09, 2.28,
            2.33, 2.38, 2.44, 2.51, 2.59, 2.69, 2.82, 3.00, 3.30, 3.34, 3.40, 3.45, 3.52, 3.60,
            3.70, 3.82, 4.00, 4.30, 7.00, 12.52,
        ],
    });
    assert_eq!(output.v_total.len(), output.ph.len());
    let theme = 0;
    render! {
        style { CSS_BASE }
        style { CSS_THEMES[theme] }
        Diagram { data: output.clone() }
    }
}

const DIAGRAM_WIDTH: f64 = 360.0;
const DIAGRAM_HEIGHT: f64 = 260.0;
const DIAGRAM_TOP: f64 = 20.0;
const DIAGRAM_BOTTOM: f64 = 280.0;
const DIAGRAM_LEFT: f64 = 20.0;
const DIAGRAM_RIGHT: f64 = 380.0;
const DIAGRAM_X_GAPS: f64 = 5.0;

#[component]
fn Diagram(cx: Scope, data: Rc<Output>) -> Element {
    let max = data.max_v();
    let x_steps = (max / DIAGRAM_X_GAPS).ceil() as usize;
    let scale = (
        DIAGRAM_WIDTH / DIAGRAM_X_GAPS / x_steps as f64,
        DIAGRAM_HEIGHT / 14.0,
    );
    render! {
        svg {
            class: "diagram",
            view_box: "0 0 400 300",
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
            let y = DIAGRAM_BOTTOM - DIAGRAM_HEIGHT / 14.0 * ph as f64;
            rsx! {
                // if ph % 2 == 0 {rsx!{
                line {
                    class: "diagram-grid",
                    x1: DIAGRAM_LEFT,
                    y1: y,
                    x2: DIAGRAM_RIGHT,
                    y2: y,
                }
                // }}
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
            let x = 20.0 + DIAGRAM_WIDTH / *x_steps as f64 * v as f64;
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
                    y: DIAGRAM_BOTTOM + 10.0, //291.5,
                    "{v as f64 * DIAGRAM_X_GAPS}"
                }
            }
        }),
        polyline {
            class: "diagram-axis",
            points: "20,20 20,280 380,280",
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

#![allow(non_snake_case)]

pub mod curve;
pub mod diagram;

use std::rc::Rc;

use calamine::{open_workbook, Reader, Xlsx};
use curve::Output;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

use crate::diagram::Diagram;

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
    let theme = use_state(cx, || 0usize);
    render! {
        style { CSS_BASE }
        style { CSS_THEMES[*theme.get()] }
        button {
            onclick: move |_| {
                theme.set((theme.get() + 1) % CSS_THEMES.len());
            },
            "☀️"
        }
        Diagram { data: output.clone() }
    }
}

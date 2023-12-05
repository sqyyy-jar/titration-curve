#![allow(non_snake_case)]

pub mod curve;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        Config::new().with_window(WindowBuilder::new().with_title("Titrationskurve")),
    );
}

fn App(cx: Scope) -> Element {
    render! {
        style { include_str!("./style.css") }
        h1 { "Titrationskurve" }
        Diagram {}
    }
}

fn Diagram(cx: Scope) -> Element {
    render! {
        svg {
            class: "frame",
            view_box: "0 0 120 120",
            rect {
                x: 0,
                y: 0,
                width: 20,
                height: 20,
                fill: "green"
            }
        }
    }
}

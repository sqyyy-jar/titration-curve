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
        Diagram {}
    }
}

fn Diagram(cx: Scope) -> Element {
    render! {
        svg {
            class: "frame",
            view_box: "0 0 400 300",
            polyline {
                class: "diagram-axis",
                points: "20,20 20,280 380,280",
                // fill: "none",
                // stroke: "white",
            }
            text {
                class: "diagram-text",
                x: 0,
                y: 150,
                "pH"
            }
        }
    }
}

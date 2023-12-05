pub mod curve;

use dioxus::prelude::*;

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    render! {
        h1 { "Titrationskurve" }
        b { "Hello world!" }
    }
}

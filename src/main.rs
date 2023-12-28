pub mod app;
pub mod worker;

use app::TitrationCurve;
use iced::{Application, Settings, Size};

fn main() -> iced::Result {
    TitrationCurve::run(Settings {
        window: iced::window::Settings {
            min_size: Some(Size::new(880.0, 660.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

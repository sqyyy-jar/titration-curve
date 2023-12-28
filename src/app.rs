pub mod options;

use std::rc::Rc;

use iced::{
    executor,
    widget::{button, checkbox, column, container, row, text},
    Application, Command, Element, Length, Theme,
};
use rfd::FileDialog;

use self::options::Options;

#[derive(Clone, Debug)]
pub enum Message {
    /// Set the `dark` option
    SetDark(bool),
    /// Set the `colored` options
    SetColored(bool),
    /// Open a file
    SelectFile,
}

#[derive(Default)]
pub struct TitrationCurve {
    options: Rc<Options>,
}

impl Application for TitrationCurve {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Titrationskurve".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SetDark(dark) => self.options.set_dark(dark),
            Message::SetColored(colored) => self.options.set_colored(colored),
            Message::SelectFile => {
                if let Some(file) = FileDialog::new()
                    .add_filter(
                        "Tabelle",
                        &["xls", "xlsx", "xlsm", "xlsb", "xla", "xlam", "ods"],
                    )
                    .pick_file()
                {
                    println!("{file:?}");
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let controls = {
            let dark_toggle = checkbox("Dunkel", self.options.is_dark(), Message::SetDark);
            let colored_toggle =
                checkbox("Gefärbt", self.options.is_colored(), Message::SetColored);
            let file_button = button("Datei auswählen").on_press(Message::SelectFile);
            container(
                column![dark_toggle, colored_toggle, file_button]
                    .spacing(5)
                    .padding(10),
            )
            .width(Length::Fixed(110.0))
            .height(Length::Fill)
        };
        let graph = container(text("TODO"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10);
        let content = row![controls, graph];
        content.into()
    }

    fn theme(&self) -> Self::Theme {
        if self.options.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

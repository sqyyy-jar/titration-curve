pub mod diagram;
pub mod options;
pub mod strings;

use std::{
    sync::{mpsc::Receiver, Arc},
    time::{Duration, Instant},
};

use iced::{
    executor,
    time::every,
    widget::{button, checkbox, column, container, row, svg, svg::Handle, text},
    Application, Command, ContentFit, Element, Length, Subscription, Theme,
};

use crate::{
    util::*,
    worker::{Output, Response, Signal, Worker},
};

use self::{
    options::Options,
    strings::{BUTTON_SELECT_FILE, MESSAGE_NO_CONTENT, OPTION_COLORED, OPTION_DARK, WINDOW_TITLE},
};

#[derive(Clone, Debug)]
pub enum Message {
    /// Sets the `dark` option.
    SetDark(bool),
    /// Sets the `colored` option.
    SetColored(bool),
    /// Opens a file dialog.
    SelectFile,
    /// Processes the response queue.
    Update(Instant),
}

pub struct TitrationCurve {
    options: Options,
    worker: Arc<Worker>,
    response_receiver: Receiver<Response>,
    /// The content of the window.
    ///
    /// Either a graph of the output or a message.
    content: Either<Arc<Output>, String>,
}

impl Application for TitrationCurve {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (worker, response_receiver) = Worker::spawn();
        let app = Self {
            options: Options::default(),
            worker,
            response_receiver,
            content: Right(MESSAGE_NO_CONTENT.into()),
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        WINDOW_TITLE.into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SetDark(dark) => self.options.dark = dark,
            Message::SetColored(colored) => self.options.colored = colored,
            Message::SelectFile => self.worker.send_signal(Signal::FileDialog),
            Message::Update(_) => {
                while let Ok(response) = self.response_receiver.try_recv() {
                    match response {
                        Response::Unload => self.content = Right(MESSAGE_NO_CONTENT.into()),
                        Response::Output(output) => self.content = Left(output),
                        Response::Error(err) => {
                            self.content = Right(format!("Ein Fehler ist aufgetreten: {err}"))
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let controls = {
            let dark_toggle = checkbox(OPTION_DARK, self.options.dark, Message::SetDark);
            let colored_toggle =
                checkbox(OPTION_COLORED, self.options.colored, Message::SetColored);
            let file_button = button(BUTTON_SELECT_FILE).on_press(Message::SelectFile);
            container(
                column![dark_toggle, colored_toggle, file_button]
                    .spacing(5)
                    .padding(10),
            )
            .width(Length::Fixed(110.0))
            .height(Length::Fill)
        };
        let content = match &self.content {
            Left(output) => {
                let svg_text = diagram::render_graph(&self.options, &output);
                let handle = Handle::from_memory(svg_text.into_bytes());
                container(
                    svg(handle)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(ContentFit::Contain),
                )
            }
            Right(message) => container(text(message)),
        }
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(10);
        row![controls, content].into()
    }

    fn theme(&self) -> Self::Theme {
        if self.options.dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([every(Duration::from_millis(500)).map(Message::Update)])
    }
}

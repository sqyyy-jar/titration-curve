use std::cell::Cell;

#[derive(Default)]
pub struct Options {
    dark: Cell<bool>,
    colored_background: Cell<bool>,
}

impl Options {
    pub fn is_dark(&self) -> bool {
        self.dark.get()
    }

    pub fn set_dark(&self, value: bool) {
        self.dark.set(value);
    }

    pub fn is_colored(&self) -> bool {
        self.colored_background.get()
    }

    pub fn set_colored(&self, value: bool) {
        self.colored_background.set(value);
    }
}

macro_rules! theme {
    ($icon: literal, $name: literal, $id: literal) => {
        Theme {
            icon: $icon,
            name: $name,
            id: $id,
            sheet: include_str!(concat!("styles/theme/", $id, ".css")),
        }
    };
    (@include $path: literal) => {
        include_str!($path)
    };
}

pub struct Theme<'a> {
    pub icon: &'a str,
    pub name: &'a str,
    pub id: &'a str,
    pub sheet: &'a str,
}

pub const CSS_BASE: &str = include_str!("styles/base.css");
pub const CSS_THEMES: &[Theme] = &[
    theme!("🌍", "System", "os"),
    theme!("☀️", "Hell", "light"),
    theme!("🌙", "Dunkel", "dark"),
    theme!("🌈", "Gefärbt", "colored"),
];

pub struct Settings {
    pub window_width: i32,
    pub window_height: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 500,
        }
    }
}

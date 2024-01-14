#[derive(Clone, Debug)]
pub struct Config {
    pub trace: bool,
}

impl Config {
    pub fn new() -> Config {
        Config { trace: false }
    }
}

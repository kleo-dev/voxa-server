use std::fmt::Display;

pub struct Logger {
    name: String,
}

impl Logger {
    pub fn new<T: Display>(name: T) -> Self {
        Logger {
            name: name.to_string(),
        }
    }

    pub fn info<T: Display>(&self, message: T) {
        println!("\x1b[32mINFO\x1b[0m ({}) › {}", self.name, message);
    }

    pub fn warn<T: Display>(&self, message: T) {
        println!("\x1b[33mWARN\x1b[0m ({}) › {}", self.name, message);
    }

    pub fn error<T: Display>(&self, message: T) {
        println!("\x1b[31mERROR\x1b[0m ({}) › {}", self.name, message);
    }
}

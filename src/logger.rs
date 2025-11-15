pub trait Logger {
    fn log(&mut self, message: String);
    fn can_write(&self) -> bool;
    fn results(self) -> String;
}

// Реализация, которая ничего не делает
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn log(&mut self, _message: String) {
        // Ничего не делаем
    }

    fn can_write(&self) -> bool {
        false
    }

    fn results(self) -> String {
        "Логирование было отключено.".to_string()
    }
}

// Реализация, которая собирает логи в строку
pub struct StringLogger {
    buffer: String,
}

impl StringLogger {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl Logger for StringLogger {
    fn log(&mut self, message: String) {
        self.buffer.push_str(&message);
        self.buffer.push('\n');
    }

    fn can_write(&self) -> bool {
        true
    }

    fn results(self) -> String {
        if self.buffer.is_empty() {
            "Алгоритм не произвел никаких логов.".to_string()
        } else {
            self.buffer
        }
    }
}
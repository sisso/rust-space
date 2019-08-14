use std::io::Write;

pub trait Save {
    fn init(&mut self);
    fn add(&mut self, content: String);
    fn close(&mut self);
}

pub struct SaveToFile {
    target: String,
    buffer: Vec<String>
}

impl SaveToFile {
    pub fn new(target: String) -> Self {
        SaveToFile {
            target,
            buffer: Vec::new(),
        }
    }
}

impl Save for SaveToFile {
    fn init(&mut self) {
    }

    fn add(&mut self, content: String) {
        self.buffer.push(content);
    }

    fn close(&mut self) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create(&self.target).unwrap();
        file.write_all(self.buffer.join("\n").as_bytes()).unwrap();
        file.flush();
    }
}

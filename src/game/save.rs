use serde_json::json;

use std::io::Write;

pub trait Save {
    fn init(&mut self);
    fn add_header(&mut self, header_name: &str, value: serde_json::Value);
    fn add(&mut self, id: u32, component: &str, value: serde_json::Value);
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

    fn add_header(&mut self, header_name: &str, header_value: serde_json::Value) {
        self.buffer.push(json!({
            "header": header_name,
            "value": header_value
        }).to_string());
    }

    fn add(&mut self, id: u32, component: &str, value: serde_json::Value) {
        self.buffer.push(json!({
            "id": id,
            "component": component,
            "value": value
        }).to_string());
    }

    fn close(&mut self) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create(&self.target).unwrap();
        let _ = file.write_all(self.buffer.join("\n").as_bytes()).unwrap();
        let _ = file.flush().unwrap();
    }
}

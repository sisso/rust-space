use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::{Write, BufReader};
use std::collections::HashMap;

pub trait Save {
    fn init(&mut self);
    fn add_header(&mut self, header_name: &str, value: serde_json::Value);
    fn add(&mut self, id: u32, component: &str, value: serde_json::Value);
    fn close(&mut self);
}

pub trait Load {
    fn get_headers(&mut self) -> Vec<serde_json::Value>;
    fn get_components(&mut self, component: &str) -> Vec<(u32, serde_json::Value)>;
}

pub struct SaveToFile {
    file_path: String,
    buffer: Vec<String>
}

impl SaveToFile {
    pub fn new(file_path: &str) -> Self {
        SaveToFile {
            file_path: file_path.to_string(),
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
        let mut file = File::create(self.file_path.to_string()).unwrap();
        let _ = file.write_all(self.buffer.join("\n").as_bytes()).unwrap();
        let _ = file.flush().unwrap();
    }
}

pub struct LoadFromFile {
    headers: Vec<Value>,
    components: HashMap<String, Vec<Value>>,
}

impl LoadFromFile {
    pub fn new(file_path: &str) -> Self {
        let mut file = File::create(file_path).unwrap();
        let mut headers: Vec<Value> = vec![];
        let mut components: HashMap<String, Vec<Value>> = HashMap::new();

        for line in BufReader::new(file).lines() {
            let line = line.unwrap();

            unimplemented!();
        }

        LoadFromFile {
            headers: headers,
            components: components,
        }
    }
}

impl Load for LoadFromFile {
    fn get_headers(&mut self) -> Vec<Value> {
        unimplemented!()
    }

    fn get_components(&mut self, component: &str) -> Vec<(u32, Value)> {
        unimplemented!()
    }
}

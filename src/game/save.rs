use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::{Write, BufReader};
use std::collections::HashMap;
use crate::game::jsons::JsonValueExtra;

pub trait CanSave {
    fn save(&self, save: &mut impl Save);
}

pub trait CanLoad {
    fn load(&mut self, load: &mut impl Load);
}

pub trait Save {
    fn add_header(&mut self, header_name: &str, value: serde_json::Value);
    fn add(&mut self, id: u32, component: &str, value: serde_json::Value);
    fn close(&mut self);
}

pub trait Load {
    fn get_headers(&mut self, header: &str) -> Vec<&serde_json::Value>;
    fn get_components(&mut self, component: &str) -> Vec<&(u32, serde_json::Value)>;
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
    headers: HashMap<String, Vec<Value>>,
    components: HashMap<String, Vec<(u32, Value)>>,
}

impl LoadFromFile {
    pub fn new(file_path: &str) -> Self {
        let mut load = LoadFromFile {
            headers: HashMap::new(),
            components: HashMap::new(),
        };

        let file = File::open(file_path).expect(&format!("failed to open file {:?}", file_path));

        for line in BufReader::new(file).lines() {
            let line = line.expect(&format!("failed to read line from {:?}", file_path));
            let ast: Value = serde_json::from_str(line.as_str()).unwrap();
            // TODO: remove clone
            let value: Value = ast["value"].clone();

            if let Some(header) = ast["header"].as_str() {
                let list = load.headers.entry(header.to_string()).or_default();
                list.push(value);
            } else {
                let id = ast["id"].to_u32();
                let component = ast["component"].as_str().unwrap();
                let list = load.components.entry(component.to_string()).or_default();
                list.push((id, value));
            }
        }

        load
    }
}

impl Load for LoadFromFile {
    fn get_headers(&mut self, header: &str) -> Vec<&Value> {
        self.headers
            .get(header)
            .map(|i| { i.iter().collect() })
            .unwrap_or(Vec::new())
    }

    fn get_components(&mut self, component: &str) -> Vec<&(u32, Value)> {
        self.components
            .get(component)
            .map(|i| { i.iter().collect() })
            .unwrap_or(Vec::new())
    }
}

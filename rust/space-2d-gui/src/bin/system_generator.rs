use commons::prob::Weighted;
use rand::prelude::*;
use space_2d_gui::system_generator::*;
use std::path::PathBuf;

fn main() {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);

    let cfg =
        space_2d_gui::system_generator::new_config(PathBuf::from("space-2d-gui/data").as_path());

    let _params = GenerateParams {};

    let system = new_system(&cfg, &mut rng);

    println!("System {:?}", system.coords);

    let mut tree = commons::tree::Tree::new();
    for b in system.bodies.iter() {
        if b.index == 0 && b.parent == 0 {
            continue;
        }
        tree.insert(b.index, b.parent);
    }

    for i in tree.iter_deepfirst() {
        let prefix = (0..i.deep).fold(String::new(), |acc, _v| format!("{}--", acc));
        println!("{}{:?}", prefix, system.bodies[i.index]);
    }
}

fn load_csv_into_weighted(raw: &str) -> Vec<Weighted<String>> {
    let mut r = vec![];
    let csv = commons::csv::parse_csv_ext(raw, '\t');
    for row in &csv {
        r.push(Weighted {
            prob: row[1].parse().expect("fail to parse prob"),
            value: row[0].to_string(),
        });
    }
    r
}

fn load_csv_into_resources(raw: &str) -> Vec<Resource> {
    fn to_str_array(s: &str) -> Vec<String> {
        s.split(",")
            .map(String::from)
            .filter(|i| !i.is_empty())
            .collect()
    }

    let csv = commons::csv::parse_csv_ext(raw, '\t');
    let mut list = vec![];
    for r in &csv {
        list.push(Resource {
            kind: r[0].to_string(),
            prob: r[1].parse().unwrap(),
            always: to_str_array(r[2]),
            require: to_str_array(r[3]),
            forbidden: to_str_array(r[4]),
        });
    }
    list
}

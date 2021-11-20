use std::{env, path::Path, time::Instant};

use flapigen::{DotNetConfig, LanguageConfig};

fn main() {
    env_logger::init();
    compile_dotnet();
}

fn compile_dotnet() {
    fn flapigen_expand(from: &Path, out: &Path) {
        println!("Run flapigen_expand");
        let config = DotNetConfig::new("ffi_domain_2".to_owned(), "generated_dotnet".into());
        let mut swig_gen = flapigen::Generator::new(LanguageConfig::DotNetConfig(config));
        swig_gen = swig_gen.rustfmt_bindings(true);
        swig_gen.expand("ffi_domain_2", from, out);
    }

    let now = Instant::now();

    let out_dir = env::var("OUT_DIR").unwrap();
    flapigen_expand(
        Path::new("src/glue.rs.in"),
        &Path::new(&out_dir).join("glue.rs"),
    );
    let expand_time = now.elapsed();
    println!(
        "flapigen expand time: {}",
        expand_time.as_secs() as f64 + (expand_time.subsec_nanos() as f64) / 1_000_000_000.
    );
    println!("cargo:rerun-if-changed=src/glue.rs.in");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
}

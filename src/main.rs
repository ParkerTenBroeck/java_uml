use std::{fs::File, io::BufWriter};

use java_uml::{java::project, plantuml::PlantUmlGen};

pub mod java;

fn main() {
    let mut files = project::Files::new();
    files
        .load_dir(std::env::args().nth(1).expect("Expected path to project"))
        .unwrap();

    let mut project = match project::Project::parse_all(&files) {
        Ok(ok) => ok,
        Err(errs) => {
            for err in errs {
                println!("{err}");
            }
            return;
        }
    };

    project.resolve_imports();
    project.resolve_types();
    let file = File::create("output.txt").expect("Failed to create output file");
    let mut writter = BufWriter::new(file);
    PlantUmlGen::new(&mut writter, &project)
        .write()
        .expect("Failed to create UML");
}

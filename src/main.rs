use std::{fs::File, io::BufWriter};

use java_uml::{java::project, plantuml::PlantUmlGen};

pub mod java;

fn main() {
    let mut files = project::Files::new();
    files
        .load_dir(std::env::args().nth(1).expect("Expected path to project"))
        .unwrap();

    let mut project = match project::Project::parse_all(&files){
        Ok(ok) => ok,
        Err((p, str, err)) => {
            let range = match &err{
                java_uml::java::parser::ParseError::UnexpectedToken { token, range } => range.clone(),
                java_uml::java::parser::ParseError::ExpectedFoundNone => todo!(),
                java_uml::java::parser::ParseError::ExpectedToken { expected, got, range } => range.clone(),
                java_uml::java::parser::ParseError::ExpectedTokenFoundNone { expected } => todo!(),
            };

            let line_start = str[..range.start].rfind('\n').map(|v|v+1).unwrap_or(0);
            let line_end = str[range.end..].find('\n').map(|v|v-1+range.end).unwrap_or(str.len());
            let col = range.start - line_start;
            let lines = str[..range.start].chars().filter(|v|*v=='\n').count();
            let msg = &str[line_start..=line_end];

            panic!("file: {p:?}:{lines}{col}\n{line_start}: {msg}");
        },
    };

    project.resolve_imports();
    project.resolve_types();
    let file = File::create("output.txt").expect("Failed to create output file");
    let mut writter = BufWriter::new(file);
    PlantUmlGen::new(&mut writter, &project)
        .write()
        .expect("Failed to create UML");
    println!("{:#?}", project);
}

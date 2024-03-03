use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use super::{
    ast::{
        class::Class,
        functions,
        types::{JType, TypePath, TypeResolution},
        Import, Imports, JPath, Metadata,
    },
    parser::{self, ParseError},
};

#[derive(Default, Debug)]
pub struct Files {
    pub files: HashMap<PathBuf, String>,
}

impl Files {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_dir(&mut self, root: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let root = root.as_ref();
        if std::fs::metadata(root)?.is_file() {
            let file = std::fs::read_to_string(root)?;
            self.files.insert(root.to_owned(), file);
        } else {
            for path in std::fs::read_dir(root)?.flatten() {
                self.load_dir(path.path())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct PackagePath(pub JPath);

impl Borrow<JPath> for ClassPath {
    fn borrow(&self) -> &JPath {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ClassPath(pub JPath);

#[derive(Debug, Default)]
pub struct Project<'a> {
    pub type_map: HashMap<ClassPath, Class<'a>>,
    pub types: HashSet<ClassPath>,
    pub imports: HashMap<ClassPath, Arc<Mutex<Imports>>>,
    pub files: HashMap<ClassPath, &'a Path>,
    pub packages: HashMap<PackagePath, Vec<ClassPath>>,
    pub path_resolves: HashMap<JPath, Vec<ClassPath>>,
}

#[derive(Debug, Clone)]
pub struct ParseFileError<'a>{
    pub path: &'a Path,
    pub contents: &'a str,
    pub error: ParseError<'a>,
}

const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
// const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
// const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0;22m";



impl<'a> std::fmt::Display for ParseFileError<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{BOLD}{RED}error{RESET}{RESET}{BOLD}: ")?;
        match self.error.kind{
            parser::ParseErrorKind::UnexpectedToken { message, got } => writeln!(f, "unexpected token {got:?} {message}")?,
            parser::ParseErrorKind::ExpectedToken { expected, got } =>  writeln!(f, "expected {expected} got {got:?}")?,
            parser::ParseErrorKind::ExpectedTokenFoundNone { expected } => writeln!(f, "expected {expected} got EOF")?,
            parser::ParseErrorKind::ArrayDegreeTooBig => writeln!(f, "array degree too big")?,
        }

        let (line, col, line_contents, adj_range) = if let Some(range) = &self.error.range{
            let line_start = self.contents[..range.start].rfind('\n').map(|v|v+1).unwrap_or(0);
            let line_end = self.contents[range.end..].find('\n').map(|v|v+range.end-1).unwrap_or(self.contents.len());
            let col = range.start - line_start;
            let lines = self.contents[..range.start].chars().filter(|v|*v=='\n').count()+1;
            let msg = &self.contents[line_start..=line_end];
            (lines, col, msg, (range.start-line_start)..(range.start-line_start + range.end-range.start))
        }else{
            let lines = self.contents.chars().filter(|v|*v=='\n').count()+1;
            let line_start = self.contents.rfind('\n').map(|v|v+1).unwrap_or(0);
            let col = self.contents.len() - line_start;
            let msg = &self.contents[line_start..];
            (lines, col, msg, 0..0)
        };

        // this is funny
        let space = "                                                                                                                                                                                                                                                                ";
        
        let space = &space[0..((line as f32).log10().floor() as u8) as usize + 1];

        writeln!(
            f,
            "{BLUE}{BOLD}{space}--> {RESET}{}:{}:{}",
            self.path.display(),
            line,
            col
        )?;
        writeln!(f, "{BLUE}{BOLD}{space} |")?;
        if self.error.range.is_some(){
            let mut index = 0;
            for (i, line_contents) in line_contents.split('\n').enumerate(){ 
                writeln!(
                    f,
                    "{} |{RESET} {}",
                    line + i,
                    &line_contents
                )?;
                write!(f, "{BLUE}{BOLD}{space} | ")?;
                for c in line_contents.chars(){
                    if adj_range.contains(&index){
                        write!(f, "~")?;
                    }else{
                        write!(f, " ")?;
                    }
                    index += c.len_utf8();
                }
                //nl
                if adj_range.contains(&index){
                    write!(f, "~")?;
                }else{
                    write!(f, " ")?;
                }
                index += 1;
                writeln!(f)?;
            }
            write!(f, "{RESET}")
        }else{
            writeln!(
                f,
                "{} |{RESET} {}",
                line,
                &line_contents
            )?;
            
            write!(f, "{BLUE}{BOLD}{space} | ")?;
            for _ in line_contents.chars(){
                write!(f, " ")?;
            }
            writeln!(f, "~{RESET}")
        }
    }
}

impl<'a> Project<'a> {
    pub fn parse_all(files: &'a Files) -> Result<Project<'a>, Vec<ParseFileError<'a>>> {
        let mut myself = Self::default();
        let mut vec = Vec::new();
        for (path, contents) in &files.files {
            let result = parser::Parser::new(contents).parse();
            match result {
                Ok(class) => {
                    myself
                        .imports
                        .insert(ClassPath(class.class_path.clone()), class.imports.clone());
                    myself.add_class(path, class)
                }
                Err(error) => vec.push(ParseFileError{path, contents, error}),
            }
        }
        if vec.is_empty(){
            Ok(myself)
        }else{
            Err(vec)
        }
    }

    fn add_class(&mut self, path: &'a Path, mut class: Class<'a>) {
        let mut inner_classes = Vec::new();
        std::mem::swap(&mut inner_classes, &mut class.inner_classes);
        for class in inner_classes {
            self.add_class(path, class);
        }
        self.packages
            .entry(PackagePath(class.package.clone().unwrap_or_default()))
            .or_default()
            .push(ClassPath(class.class_path.clone()));
        self.files.insert(ClassPath(class.class_path.clone()), path);

        let mut parent = class.class_path.clone();
        parent.pop_part();

        self.path_resolves
            .entry(parent)
            .or_default()
            .push(ClassPath(class.class_path.clone()));

        self.types.insert(ClassPath(class.class_path.clone()));
        self.type_map
            .insert(ClassPath(class.class_path.clone()), class);
    }

    pub fn resolve_imports(&mut self) {
        for (path, import) in &mut self.imports {
            let mut lock = import.lock().unwrap();
            let Imports { name_map, wildcard } = &mut *lock;

            let mut path = path.clone();
            for import in self.path_resolves.get(&path.0).unwrap_or(&Vec::new()) {
                if !name_map.contains_key(import.0.last()) {
                    name_map.insert(
                        import.0.last().to_owned(),
                        Import {
                            path: import.0.clone(),
                            is_static: false,
                        },
                    );
                }
            }
            path.0.pop_part();
            for import in self.path_resolves.get(&path.0).unwrap_or(&Vec::new()) {
                if !name_map.contains_key(import.0.last()) {
                    name_map.insert(
                        import.0.last().to_owned(),
                        Import {
                            path: import.0.clone(),
                            is_static: false,
                        },
                    );
                }
            }

            for mut import in wildcard.drain(..) {
                import.path.pop_part();
                for import in self.path_resolves.get(&import.path).unwrap_or(&Vec::new()) {
                    if !name_map.contains_key(import.0.last()) {
                        name_map.insert(
                            import.0.last().to_owned(),
                            Import {
                                path: import.0.clone(),
                                is_static: false,
                            },
                        );
                    }
                }
            }
        }
    }

    pub fn resolve_types(&mut self) {
        let Self {
            type_map, types, ..
        } = self;

        let prelude = Imports::new();
        for class in type_map.values_mut() {

            let class_imports = class.imports.clone();
            let class_imports = class_imports.lock().unwrap();
            let resolver = TypeResolve {
                prelude: &prelude,
                class_imports: &class_imports,
                classes: types,
                generics: class.generic_names.clone().unwrap_or_default(),
            };
            resolver.resolve_class(class);
        }
    }

}

struct TypeResolve<'a> {
    prelude: &'a Imports,
    class_imports: &'a Imports,
    classes: &'a HashSet<ClassPath>,
    generics: Arc<HashSet<String>>,
}

impl<'a> TypeResolve<'a> {
    pub fn resolve(&self, jtype: &mut TypePath) {
        if self.classes.contains(&jtype.origional) {
            jtype.resolved = TypeResolution::Some(jtype.origional.clone());
        } else {
            let start = jtype.origional.first();
            if let Some(resolved) = self.class_imports.name_map.get(start) {
                jtype.resolved = TypeResolution::Some(resolved.path.clone());
            } else if let Some(resolved) = self.prelude.name_map.get(start) {
                jtype.resolved = TypeResolution::Some(resolved.path.clone());
            } else if self.generics.contains(&jtype.origional.path){
                jtype.resolved = TypeResolution::Generic;
            }
        }
    }

    fn resolve_type(&self, jtype: &mut JType) {
        match jtype {
            JType::Primitive(_) => {}
            JType::PrimitiveArr(_, _) => {}
            JType::Object { path, generics, .. } => {
                self.resolve(path);
                if let Some(generics) = generics {
                    for invoction in &mut generics.invoctions {
                        match invoction {
                            super::ast::generics::GenericInvoctionPart::Type(jtype) => {
                                self.resolve_type(jtype)
                            }
                            super::ast::generics::GenericInvoctionPart::Wildcard(wildcard) => {
                                match wildcard {
                                    super::ast::generics::WildcardBound::None => {},
                                    super::ast::generics::WildcardBound::Extends(list)
                                    | super::ast::generics::WildcardBound::Super(list) => {
                                        for jtype in list {
                                            self.resolve_type(jtype);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn resolve_class(&self, class: &mut Class) {
        self.resolve_meta(&mut class.meta);
        for extends in class.extends.as_mut().unwrap_or(&mut Vec::new()) {
            self.resolve_type(extends);
        }
        for implements in class.implements.as_mut().unwrap_or(&mut Vec::new()) {
            self.resolve_type(implements);
        }
        for permits in class.permits.as_mut().unwrap_or(&mut Vec::new()) {
            self.resolve_type(permits);
        }

        for variables in &mut class.variables {
            self.resolve_meta(&mut variables.meta);
            self.resolve_type(&mut variables.jtype);
        }

        for functions in &mut class.functions {
            self.resolve_meta(&mut functions.meta);
            if let Some(generics) = &mut functions.generics {
                for definition in &mut generics.definitions {
                    if let Some(bounds) = &mut definition.extend_bound {
                        for bound in bounds {
                            self.resolve_type(bound);
                        }
                    }
                }
            }
            for param in &mut functions.parameters {
                match param {
                    functions::Parameter::Regular(jtype, _)
                    | functions::Parameter::VArgs(jtype, _) => {
                        self.resolve_type(jtype);
                    }
                }
            }

            if let Some(throws) = &mut functions.throws{
                for throw in throws{
                    self.resolve_type(throw);
                }
            }

            if let functions::FunctionKind::Regular(jtype) = &mut functions.kind {
                self.resolve_type(jtype);
            }
        }
    }

    fn resolve_meta(&self, _meta: &mut Metadata) {}
}

// ----------------------- Visitor stuff

impl<'a> Project<'a>{
    pub fn visit<T: Visitor>(visitor: &mut T) -> Result<T::Ok, T::Err>{
        visitor.visit_start()?;



        visitor.visit_end()
    }
}

pub trait Visitor{
    type Ok;
    type Err;

    fn visit_start(&mut self) -> Result<(), Self::Err>;

    fn visit_pre_meta(&mut self) -> Result<(), Self::Err>;

    fn visit_package_start(&mut self, package: &JPath) -> Result<(), Self::Err>;
    fn visit_pre_class_meta(&mut self) -> Result<(), Self::Err>;

    fn visit_class(&mut self, class: &Class) -> Result<(), Self::Err>;
    
    fn visit_post_class_meta(&mut self) -> Result<(), Self::Err>;
    fn visit_package_end(&mut self, package: &JPath) -> Result<(), Self::Err>;

    fn visit_post_meta(&mut self) -> Result<(), Self::Err>;

    fn visit_end(&mut self) -> Result<Self::Ok, Self::Err>;

}
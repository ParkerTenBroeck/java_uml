use std::collections::HashMap;

use bitfield_struct::bitfield;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct JPath {
    pub path: String,
}

impl JPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_wildcard(&self) -> bool {
        self.path.as_bytes().last() == Some(&b'*')
    }

    pub fn push_part(&mut self, part: &str) {
        if !self.path.is_empty() {
            self.path.push('.');
        }
        self.path.push_str(part);
    }

    pub fn pop_part(&mut self) {
        while let Some(char) = self.path.pop() {
            if char == '.' {
                break;
            }
        }
    }

    pub fn prefixes(&self, other: &Self) -> Option<Self> {
        let (start, rest) = other
            .path
            .split_once('.')
            .unwrap_or((other.path.as_str(), ""));
        if !start.is_empty() && self.path.ends_with(start) {
            let mut path = String::with_capacity(rest.len() + 1 + self.path.len());
            path.push_str(&self.path);
            path.push('.');
            path.push_str(rest);
            Some(JPath { path })
        } else {
            None
        }
    }

    pub fn last(&self) -> &str {
        self.path.split('.').last().unwrap()
    }

    pub fn first(&self) -> &str {
        self.path.split('.').next().unwrap()
    }
}

impl std::fmt::Display for JPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.path.split(|c| matches!(c, 'C' | 'B'));
        if let Some(part) = iter.next() {
            write!(f, "{}", part)?;
        }
        for part in iter {
            write!(f, ".{}", part)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
    None,
}

#[bitfield(u16, debug = false)]
pub struct Modifiers {
    pub m_static: bool,
    pub m_abstract: bool,
    pub m_synchronized: bool,
    pub m_transient: bool,
    pub m_volatile: bool,
    pub m_final: bool,
    pub m_native: bool,
    pub m_default: bool,
    pub m_strictfp: bool,
    pub m_sealed: bool,
    pub m_non_sealed: bool,
    #[bits(5)]
    _pad: u16,
}

impl std::fmt::Debug for Modifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = f.debug_list();
        type FieldMethod<'a> = (fn(&Modifiers) -> bool, &'a str);
        let fields: [FieldMethod; 11] = [
            (Self::m_static, "static"),
            (Self::m_abstract, "abstract"),
            (Self::m_synchronized, "synchronized"),
            (Self::m_transient, "transient"),
            (Self::m_volatile, "volatile"),
            (Self::m_final, "final"),
            (Self::m_native, "native"),
            (Self::m_default, "default"),
            (Self::m_strictfp, "strictfp"),
            (Self::m_sealed, "sealed"),
            (Self::m_non_sealed, "non-sealed"),
        ];
        for field in fields {
            if field.0(self) {
                formatter.entry(&field.1);
            }
        }
        formatter.finish()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Imports {
    pub name_map: HashMap<String, Import>,
    pub wildcard: Vec<Import>,
}
impl Imports {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, import: Import) {
        if !import.is_static {
            if import.path.is_wildcard() {
                self.wildcard.push(import);
            } else {
                self.name_map.insert(import.path.last().to_owned(), import);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: JPath,
    pub is_static: bool,
}
impl Import {
    pub fn new(path: JPath, is_static: bool) -> Self {
        Self { path, is_static }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Annotations {
    pub annotations: Vec<String>,
}
impl Annotations {
    pub fn new() -> Self {
        Self::default()
    }
}

pub mod class {
    use std::{collections::HashSet, sync::{Arc, Mutex}};

    use crate::java::tokenizer::UmlMeta;

    use super::{
        functions::Function, generics::GenericDefinition, types::JType, variable::Variable,
        Annotations, Imports, JPath, Modifiers, Visibility,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ClassType {
        Class,
        Interface,
        Record,
        Enum(Vec<String>),
    }

    #[derive(Debug, Clone)]
    pub struct Class<'a> {
        pub package: Option<JPath>,
        pub imports: Arc<Mutex<Imports>>,
        pub meta: Vec<UmlMeta<'a>>,
        pub annotations: Annotations,
        pub visibility: Visibility,
        pub modifiers: Modifiers,
        pub class_type: ClassType,
        pub name: &'a str,
        pub class_path: JPath,
        pub parent_class: Option<JPath>,
        pub generics: Option<GenericDefinition>,
        pub generic_names: Option<Arc<HashSet<String>>>,
        pub extends: Option<Vec<JType>>,
        pub implements: Option<Vec<JType>>,
        pub permits: Option<Vec<JType>>,
        pub variables: Vec<Variable<'a>>,
        pub functions: Vec<Function<'a>>,
        pub inner_classes: Vec<Class<'a>>,
    }
}

pub mod variable {
    use crate::java::tokenizer::UmlMeta;

    use super::{types::JType, Annotations, Modifiers, Visibility};

    #[derive(Debug, Clone)]
    pub struct Variable<'a> {
        pub meta: Vec<UmlMeta<'a>>,
        pub annotations: Annotations,
        pub visibility: Visibility,
        pub modifiers: Modifiers,
        pub jtype: JType,
        pub name: &'a str,
    }
}

pub mod functions {
    use crate::java::tokenizer::UmlMeta;

    use super::{generics::GenericDefinition, types::JType, Annotations, Modifiers, Visibility};

    #[derive(Debug, Clone)]
    pub enum FunctionKind {
        Regular(JType),
        Constructor,
        CompactConstructor,
    }

    #[derive(Debug, Clone)]
    pub struct Function<'a> {
        pub meta: Vec<UmlMeta<'a>>,
        pub annotations: Annotations,
        pub visibility: Visibility,
        pub modifiers: Modifiers,
        pub generics: Option<GenericDefinition>,
        pub kind: FunctionKind,
        pub name: &'a str,
        pub parameters: Vec<Parameter>,
    }

    #[derive(Debug, Clone)]
    pub enum Parameter {
        Regular(JType, String),
        VArgs(JType, String),
    }
}

pub mod types {
    use std::num::NonZeroU8;

    use super::{generics::GenericInvoction, JPath};

    #[derive(Debug, Clone)]
    pub enum Primitive {
        Byte,
        Short,
        Int,
        Long,
        Float,
        Double,
        Char,
        Void,
        Boolean,
    }

    #[derive(Debug, Clone)]
    pub enum TypeResolution{
        None,
        Some(JPath),
        Generic,
    }

    #[derive(Debug, Clone)]
    pub struct TypePath {
        pub origional: JPath,
        pub resolved: TypeResolution,
    }

    impl TypePath {
        pub fn new(origional: JPath) -> Self {
            Self {
                origional,
                resolved: TypeResolution::None,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum JType {
        Primitive(Primitive),
        PrimitiveArr(Primitive, NonZeroU8),
        Object {
            path: TypePath,
            generics: Option<GenericInvoction>,
            arr: Option<NonZeroU8>,
        },
    }
}

pub mod generics {
    use super::types::JType;

    #[derive(Debug, Default, Clone)]
    pub struct GenericDefinition {
        pub definitions: Vec<GenericDefinitionPart>,
    }
    impl GenericDefinition {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, part: GenericDefinitionPart) {
            self.definitions.push(part);
        }
    }

    #[derive(Debug, Default, Clone)]
    pub struct GenericInvoction {
        pub invoctions: Vec<GenericInvoctionPart>,
    }
    impl GenericInvoction {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add(&mut self, kind: GenericInvoctionPart) {
            self.invoctions.push(kind);
        }
    }

    #[derive(Debug, Clone)]
    pub enum GenericInvoctionPart {
        Type(JType),
        Wildcard(WildcardBound),
    }

    #[derive(Debug, Clone)]
    pub struct GenericDefinitionPart {
        pub name: String,
        pub extend_bound: Option<Vec<JType>>,
    }

    #[derive(Debug, Clone)]
    pub enum WildcardBound {
        None,
        Extends(Vec<JType>),
        Super(Vec<JType>),
    }
}

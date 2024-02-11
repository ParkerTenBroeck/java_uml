use std::{collections::HashSet, num::NonZeroU8, ops::Range, sync::Arc};

use crate::java::ast::{
    generics::GenericInvoction,
    types::{Primitive, TypePath},
};

use super::{
    ast::{
        class::{Class, ClassType},
        functions::{Function, FunctionKind, Parameter},
        generics::{GenericDefinition, GenericDefinitionPart, GenericInvoctionPart, WildcardBound},
        types::JType,
        variable::Variable,
        Annotations, Import, Imports, JPath, Modifiers, Visibility,
    },
    tokenizer::{Peek2, Peek2able, Token, Tokenizer, UmlMeta},
};

#[derive(Debug, Clone)]
pub enum ParseError<'a> {
    UnexpectedToken {
        token: Token<'a>,
        range: Range<usize>,
    },
    ExpectedFoundNone,
    ExpectedToken {
        expected: &'static str,
        got: Token<'a>,
        range: Range<usize>,
    },
    ExpectedTokenFoundNone {
        expected: &'static str,
    },
}

pub struct Parser<'a> {
    tokenizer: Peek2<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(data).peek2able(),
        }
    }

    pub fn expect_semi(&mut self) -> Result<(), ParseError<'a>> {
        match self.tokenizer.next() {
            Some((Token::Semicolon, _)) => Ok(()),
            Some((got, range)) => Err(ParseError::ExpectedToken {
                expected: "Semicolon",
                got,
                range,
            }),
            None => Err(ParseError::ExpectedTokenFoundNone {
                expected: "Semicolon",
            }),
        }
    }

    pub fn flush_semis(&mut self) {
        while let Some((Token::Semicolon, _)) = self.tokenizer.peek() {
            self.tokenizer.next();
        }
    }

    pub fn match_braces(&mut self) -> Result<(), ParseError<'a>> {
        let mut depth = 0;
        if matches!(self.tokenizer.peek(), Some((Token::LBrace, _))) {
            depth += 1;
            self.tokenizer.next();
        }
        while depth > 0 {
            match (depth, self.tokenizer.next()) {
                (_, Some((Token::LBrace, _))) => depth += 1,
                (_, Some((Token::RBrace, _))) => depth -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn match_parens(&mut self) -> Result<(), ParseError<'a>> {
        let mut depth = 0;
        if matches!(self.tokenizer.peek(), Some((Token::LPar, _))) {
            depth += 1;
            self.tokenizer.next();
        }
        while depth > 0 {
            match (depth, self.tokenizer.next()) {
                (_, Some((Token::LPar, _))) => depth += 1,
                (_, Some((Token::RPar, _))) => depth -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn remove_empty(&mut self) -> Result<(), ParseError<'a>> {
        loop {
            match self.tokenizer.peek() {
                Some((Token::LBrace, _)) => self.match_braces()?,
                Some((Token::Semicolon, _)) => self.flush_semis(),
                _ => return Ok(()),
            }
        }
    }

    pub fn parse(&mut self) -> Result<Class<'a>, ParseError<'a>> {
        self.remove_empty()?;

        let package = if let Some((Token::Package, _)) = self.tokenizer.peek() {
            self.tokenizer.next();
            let path = self.parse_path()?;
            self.expect_semi()?;
            Some(path)
        } else {
            None
        };

        let mut imports = Imports::new();

        self.remove_empty()?;
        while let Some((Token::Import, _)) = self.tokenizer.peek() {
            self.tokenizer.next();
            let is_static = if let Some((Token::Static, _)) = self.tokenizer.peek() {
                self.tokenizer.next();
                true
            } else {
                false
            };
            let path = self.parse_path()?;
            self.expect_semi()?;
            imports.add(Import::new(path, is_static));

            self.remove_empty()?;
        }

        let (meta, annotations, visibility, modifiers) = self.parse_stuff()?;

        let class_path_prefix = package.clone().unwrap_or_default();

        let class = self.parse_class(
            package,
            class_path_prefix,
            None,
            std::sync::Arc::new(std::sync::Mutex::new(imports)),
            meta,
            annotations,
            visibility,
            modifiers,
        )?;

        Ok(class)
    }

    pub fn parse_stuff(
        &mut self,
    ) -> Result<(Vec<UmlMeta<'a>>, Annotations, Visibility, Modifiers), ParseError<'a>> {
        let mut annotations = Annotations::new();
        let mut metas = Vec::new();

        loop {
            match self.tokenizer.peek() {
                Some((Token::Annotation(annotation), _)) => {
                    annotations.annotations.push((*annotation).into());
                    self.tokenizer.next();
                }
                Some((Token::UmlMeta(_), _)) => {
                    if let Some((Token::UmlMeta(meta), _)) = self.tokenizer.next() {
                        metas.push(meta);
                    }
                }
                _ => break,
            }
        }

        let visibility = self.parse_visibility();
        let modifiers = self.parse_modifiers();

        Ok((metas, annotations, visibility, modifiers))
    }

    pub fn parse_modifiers(&mut self) -> Modifiers {
        let mut modifiers = Modifiers::new();

        loop {
            match self.tokenizer.peek() {
                Some((Token::Static, _)) => {
                    modifiers.set_m_static(true);
                    self.tokenizer.next();
                }
                Some((Token::Abstract, _)) => {
                    modifiers.set_m_abstract(true);
                    self.tokenizer.next();
                }
                Some((Token::Synchronized, _)) => {
                    modifiers.set_m_synchronized(true);
                    self.tokenizer.next();
                }
                Some((Token::Transient, _)) => {
                    modifiers.set_m_transient(true);
                    self.tokenizer.next();
                }
                Some((Token::Volatile, _)) => {
                    modifiers.set_m_volatile(true);
                    self.tokenizer.next();
                }
                Some((Token::Final, _)) => {
                    modifiers.set_m_final(true);
                    self.tokenizer.next();
                }
                Some((Token::Native, _)) => {
                    modifiers.set_m_native(true);
                    self.tokenizer.next();
                }
                Some((Token::Default, _)) => {
                    modifiers.set_m_default(true);
                    self.tokenizer.next();
                }
                Some((Token::StrictFP, _)) => {
                    modifiers.set_m_strictfp(true);
                    self.tokenizer.next();
                }
                Some((Token::Sealed, _)) => {
                    modifiers.set_m_sealed(true);
                    self.tokenizer.next();
                }
                Some((Token::NonSealed, _)) => {
                    modifiers.set_m_non_sealed(true);
                    self.tokenizer.next();
                }
                _ => break,
            }
        }

        modifiers
    }

    pub fn parse_visibility(&mut self) -> Visibility {
        match self.tokenizer.peek() {
            Some((Token::Public, _)) => {
                self.tokenizer.next();
                Visibility::Public
            }
            Some((Token::Protected, _)) => {
                self.tokenizer.next();
                Visibility::Protected
            }
            Some((Token::Private, _)) => {
                self.tokenizer.next();
                Visibility::Private
            }
            _ => Visibility::None,
        }
    }

    pub fn parse_path(&mut self) -> Result<JPath, ParseError<'a>> {
        match self.tokenizer.next() {
            Some((Token::Ident(part), _)) => Ok(self.parse_path_with_start(part)?),
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }
    }

    pub fn parse_path_with_start(&mut self, start: &'a str) -> Result<JPath, ParseError<'a>> {
        let mut path = JPath::new();
        path.push_part(start);

        while let Some((Token::Dot, _)) = self.tokenizer.peek() {
            self.tokenizer.next();
            match self.tokenizer.next() {
                Some((Token::Ident(part), _)) => path.push_part(part),
                Some((Token::Star, _)) => path.push_part("*"),
                Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                None => Err(ParseError::ExpectedFoundNone)?,
            }
        }

        Ok(path)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn parse_class(
        &mut self,
        package: Option<JPath>,
        mut class_path_prefix: JPath,
        mut generic_names: Option<Arc<HashSet<String>>>,
        imports: std::sync::Arc<std::sync::Mutex<Imports>>,
        meta: Vec<UmlMeta<'a>>,
        annotations: Annotations,
        visibility: Visibility,
        mut modifiers: Modifiers,
    ) -> Result<Class<'a>, ParseError<'a>> {
        let mut class_type = match self.tokenizer.next() {
            Some((Token::Class, _)) => ClassType::Class,
            Some((Token::Interface, _)) => ClassType::Interface,
            Some((Token::Record, _)) => ClassType::Record,
            Some((Token::Enum, _)) => ClassType::Enum(Vec::new()),
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        };

        match class_type{
            ClassType::Interface
            | ClassType::Record
            | ClassType::Enum(_) => modifiers.set_m_static(true),
            _ => {}
        }

        let name = match self.tokenizer.next() {
            Some((Token::Ident(name), _)) => name,
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        };
        class_path_prefix.push_part(name);
        let class_path = class_path_prefix;

        let generics = self.parse_generic_definition()?;
        if let Some(generics) = &generics{
            let mut more_generic_names = generic_names.map(|v|(*v).clone()).unwrap_or_default();
            for generic in &generics.definitions{
                more_generic_names.insert(generic.name.clone());
            }
            generic_names = Some(Arc::new(more_generic_names));
        }

        let mut functions = Vec::new();
        let mut inner_classes = Vec::new();
        let mut variables = if ClassType::Record == class_type {
            self.parse_record_field_list()?
        } else {
            Vec::new()
        };

        let mut extends = None;
        let mut implements = None;
        let mut permits = None;

        loop {
            match self.tokenizer.peek() {
                Some((Token::Extends, _)) => {
                    self.tokenizer.next();
                    extends = Some(self.parse_type_comma_list()?)
                }
                Some((Token::Implements, _)) => {
                    self.tokenizer.next();
                    implements = Some(self.parse_type_comma_list()?)
                }
                Some((Token::Permits, _)) => {
                    self.tokenizer.next();
                    permits = Some(self.parse_type_comma_list()?)
                }
                _ => break,
            };
        }

        match self.tokenizer.next() {
            Some((Token::LBrace, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        };

        if let ClassType::Enum(names) = &mut class_type {
            loop {
                match self.tokenizer.peek().cloned() {
                    Some((Token::Ident(ident), _)) => {
                        self.tokenizer.next();
                        names.push(ident.to_string());
                        self.match_parens()?;
                        while let Some((Token::LBrace, _)) = self.tokenizer.peek() {
                            self.match_braces()?;
                        }
                    }
                    Some((Token::Semicolon, _)) => break,

                    Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                    None => Err(ParseError::ExpectedFoundNone)?,
                }

                match self.tokenizer.next() {
                    Some((Token::Comma, _)) => {}
                    Some((Token::Semicolon, _)) => break,
                    Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                    None => Err(ParseError::ExpectedFoundNone)?,
                }
            }
        }

        self.remove_empty()?;
        while {
            if matches!(self.tokenizer.peek(), Some((Token::RBrace, _))) {
                self.tokenizer.next();
                false
            } else {
                true
            }
        } {
            while let (Some((Token::Static, _)), Some((Token::LBrace, _))) =
                self.tokenizer.peek_both()
            {
                self.tokenizer.next();
                self.remove_empty()?;
            }

            let (meta, annotations, visibility, modifiers) = self.parse_stuff()?;

            match self.tokenizer.peek().cloned() {
                Some((Token::Class | Token::Interface | Token::Enum | Token::Record, _)) => {
                    let mut class = self.parse_class(
                        package.clone(),
                        class_path.clone(),
                        if modifiers.m_static() {None} else {generic_names.clone()},
                        imports.clone(),
                        meta,
                        annotations,
                        visibility,
                        modifiers,
                    )?;
                    class.parent_class = Some(class_path.clone());
                    inner_classes.push(class);
                }
                Some(start @ (Token::LAngle | Token::Ident(_), _)) => {
                    let generics = self.parse_generic_definition()?;

                    let kind = match self.tokenizer.peek_second() {
                        Some((Token::LPar, _)) => FunctionKind::Constructor,
                        Some((Token::LBrace, _)) => FunctionKind::CompactConstructor,
                        _ => FunctionKind::Regular(self.parse_type()?),
                    };

                    let name = match self.tokenizer.next() {
                        Some((Token::Ident(name), _)) => name,
                        Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                        None => Err(ParseError::ExpectedFoundNone)?,
                    };

                    match self.tokenizer.peek().cloned() {
                        Some((Token::LBrace, _)) => functions.push(Function {
                            meta,
                            annotations,
                            visibility,
                            modifiers,
                            generics,
                            kind,
                            name,
                            parameters: Vec::new(),
                        }),
                        Some((Token::LPar, _)) => functions.push(Function {
                            meta,
                            annotations,
                            visibility,
                            modifiers,
                            generics,
                            kind,
                            name,
                            parameters: self.parse_function_parameters()?,
                        }),
                        Some(_) => {
                            if generics.is_some() {
                                Err(ParseError::UnexpectedToken {
                                    token: Token::LAngle,
                                    range: start.1.clone(),
                                })?;
                            }
                            let jtype = if let FunctionKind::Regular(jtype) = kind {
                                jtype
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    token: start.0,
                                    range: start.1.clone(),
                                });
                            };
                            loop {
                                match self.tokenizer.next() {
                                    Some((Token::Semicolon, _)) => break,
                                    Some(_) => {}
                                    None => Err(ParseError::ExpectedTokenFoundNone {
                                        expected: "Expected semicolon found end of file",
                                    })?,
                                }
                            }
                            variables.push(Variable {
                                meta,
                                annotations,
                                visibility,
                                modifiers,
                                jtype,
                                name,
                            })
                        }
                        None => Err(ParseError::ExpectedFoundNone)?,
                    };
                }
                Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                None => Err(ParseError::ExpectedFoundNone)?,
            }

            self.remove_empty()?;
        }

        Ok(Class {
            package,
            imports,
            meta,
            annotations,
            visibility,
            modifiers,
            class_type,
            name,
            class_path,
            parent_class: None,
            generics,
            generic_names,
            extends,
            implements,
            permits,
            variables,
            functions,
            inner_classes,
        })
    }

    pub fn parse_type_comma_list(&mut self) -> Result<Vec<JType>, ParseError<'a>> {
        let mut list = Vec::new();
        while {
            list.push(self.parse_type()?);
            if matches!(self.tokenizer.peek(), Some((Token::Comma, _))) {
                self.tokenizer.next();
                true
            } else {
                false
            }
        } {}
        Ok(list)
    }

    pub fn parse_record_field_list(&mut self) -> Result<Vec<Variable<'a>>, ParseError<'a>> {
        match self.tokenizer.next() {
            Some((Token::LPar, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }

        let mut variables = Vec::new();

        if !matches!(self.tokenizer.peek(), Some((Token::RPar, _))) {
            while {
                let (meta, annotations, _, mut modifiers) = self.parse_stuff()?;
                let visibility = Visibility::Private;
                modifiers.set_m_final(true);

                let jtype = self.parse_type()?;
                let name = match self.tokenizer.next() {
                    Some((Token::Ident(name), _)) => name,
                    Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                    None => Err(ParseError::ExpectedFoundNone)?,
                };

                variables.push(Variable {
                    meta,
                    annotations,
                    visibility,
                    modifiers,
                    jtype,
                    name,
                });

                if matches!(self.tokenizer.peek(), Some((Token::Comma, _))) {
                    self.tokenizer.next();
                    true
                } else {
                    false
                }
            } {}
        }

        match self.tokenizer.next() {
            Some((Token::RPar, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }

        Ok(variables)
    }

    pub fn parse_function_parameters(&mut self) -> Result<Vec<Parameter>, ParseError<'a>> {
        match self.tokenizer.next() {
            Some((Token::LPar, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }

        let mut parameters = Vec::new();

        if !matches!(self.tokenizer.peek(), Some((Token::RPar, _))) {
            while {
                parameters.push(self.parse_function_parameter()?);

                if matches!(self.tokenizer.peek(), Some((Token::Comma, _))) {
                    self.tokenizer.next();
                    true
                } else {
                    false
                }
            } {}
        }

        match self.tokenizer.next() {
            Some((Token::RPar, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }

        Ok(parameters)
    }

    pub fn parse_function_parameter(&mut self) -> Result<Parameter, ParseError<'a>> {
        let jtype = self.parse_type()?;
        match self.tokenizer.next() {
            Some((Token::Ident(ident), _)) => {
                return Ok(Parameter::Regular(jtype, ident.to_owned()))
            }
            Some((Token::DotDotDot, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }

        match self.tokenizer.next() {
            Some((Token::Ident(ident), _)) => Ok(Parameter::VArgs(jtype, ident.to_owned())),
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }
    }

    pub fn parse_type(&mut self) -> Result<JType, ParseError<'a>> {
        enum Kind {
            Primitive(Primitive),
            Object(JPath, Option<GenericInvoction>),
        }
        let kind = match self.tokenizer.peek().cloned() {
            Some((Token::Ident(ident), _)) => match ident {
                "void" => Kind::Primitive(Primitive::Void),
                "boolean" => Kind::Primitive(Primitive::Boolean),
                "byte" => Kind::Primitive(Primitive::Byte),
                "char" => Kind::Primitive(Primitive::Char),
                "double" => Kind::Primitive(Primitive::Double),
                "float" => Kind::Primitive(Primitive::Float),
                "int" => Kind::Primitive(Primitive::Int),
                "long" => Kind::Primitive(Primitive::Long),
                "short" => Kind::Primitive(Primitive::Short),
                _ => {
                    let path = self.parse_path()?;
                    let generics = self.parse_generic_invoction()?;
                    Kind::Object(path, generics)
                }
            },
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        };

        if matches!(kind, Kind::Primitive(_)) {
            self.tokenizer.next();
        }

        let mut arr_degree = 0;

        while matches!(self.tokenizer.peek(), Some((Token::LBracket, _))) {
            self.tokenizer.next();
            match self.tokenizer.next() {
                Some((Token::RBracket, _)) => arr_degree += 1,
                Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                None => Err(ParseError::ExpectedFoundNone)?,
            }
        }

        Ok(match kind {
            Kind::Primitive(primitive) => {
                if let Some(arr_degree) = NonZeroU8::new(arr_degree) {
                    JType::PrimitiveArr(primitive, arr_degree)
                } else {
                    JType::Primitive(primitive)
                }
            }
            Kind::Object(type_path, generics) => JType::Object {
                path: TypePath::new(type_path),
                generics,
                arr: NonZeroU8::new(arr_degree),
            },
        })
    }

    pub fn parse_generic_definition(
        &mut self,
    ) -> Result<Option<GenericDefinition>, ParseError<'a>> {
        if matches!(self.tokenizer.peek(), Some((Token::LAngle, _))) {
            let mut definition = GenericDefinition::new();
            self.tokenizer.next();

            if !matches!(self.tokenizer.peek(), Some((Token::RAngle, _))) {
                while {
                    definition.add(self.parse_generic_definition_part()?);

                    if matches!(self.tokenizer.peek(), Some((Token::Comma, _))) {
                        self.tokenizer.next();
                        true
                    } else {
                        false
                    }
                } {}
            }

            match self.tokenizer.next() {
                Some((Token::RAngle, _)) => {}
                Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                None => Err(ParseError::ExpectedFoundNone)?,
            }

            Ok(Some(definition))
        } else {
            Ok(None)
        }
    }

    pub fn parse_generic_definition_part(
        &mut self,
    ) -> Result<GenericDefinitionPart, ParseError<'a>> {
        let name = match self.tokenizer.next() {
            Some((Token::Ident(ident), _)) => ident.to_owned(),
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        };

        let extend_bound = if matches!(self.tokenizer.peek(), Some((Token::Extends, _))) {
            self.tokenizer.next();
            Some(self.parse_bounded_type_list()?)
        } else {
            None
        };

        Ok(GenericDefinitionPart { name, extend_bound })
    }

    pub fn parse_generic_invoction(&mut self) -> Result<Option<GenericInvoction>, ParseError<'a>> {
        if matches!(self.tokenizer.peek(), Some((Token::LAngle, _))) {
            let mut invoction = GenericInvoction::new();
            self.tokenizer.next();

            if !matches!(self.tokenizer.peek(), Some((Token::RAngle, _))) {
                while {
                    let kind = if matches!(self.tokenizer.peek(), Some((Token::QuestionMark, _))) {
                        GenericInvoctionPart::Wildcard(self.parse_wildcard_bound()?)
                    } else {
                        GenericInvoctionPart::Type(self.parse_type()?)
                    };
                    invoction.add(kind);

                    if matches!(self.tokenizer.peek(), Some((Token::Comma, _))) {
                        self.tokenizer.next();
                        true
                    } else {
                        false
                    }
                } {}
            }

            match self.tokenizer.next() {
                Some((Token::RAngle, _)) => {}
                Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
                None => Err(ParseError::ExpectedFoundNone)?,
            }
            Ok(Some(invoction))
        } else {
            Ok(None)
        }
    }

    pub fn parse_wildcard_bound(&mut self) -> Result<WildcardBound, ParseError<'a>> {
        match self.tokenizer.next() {
            Some((Token::QuestionMark, _)) => {}
            Some((token, range)) => Err(ParseError::UnexpectedToken { token, range })?,
            None => Err(ParseError::ExpectedFoundNone)?,
        }

        Ok(match self.tokenizer.peek() {
            Some((Token::Extends, _)) => {
                self.tokenizer.next();
                WildcardBound::Extends(self.parse_bounded_type_list()?)
            }
            Some((Token::Super, _)) => {
                self.tokenizer.next();
                WildcardBound::Super(self.parse_bounded_type_list()?)
            }
            _ => WildcardBound::None,
        })
    }

    pub fn parse_bounded_type_list(&mut self) -> Result<Vec<JType>, ParseError<'a>> {
        let mut list = Vec::new();

        'outer: while {
            if !matches!(self.tokenizer.peek(), Some((Token::Ident(_), _))) {
                break 'outer;
            }

            list.push(self.parse_type()?);

            if matches!(self.tokenizer.peek(), Some((Token::And, _))) {
                self.tokenizer.next();
                true
            } else {
                false
            }
        } {}

        Ok(list)
    }
}

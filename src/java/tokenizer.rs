use std::ops::Range;

use plex::lexer;

// pub struct Uml

#[derive(Debug, Clone, Copy)]
pub enum UmlMeta<'a> {
    Invalid(&'a str),
    Hide,
    InnerClassNote(&'a str),
    InnerClassLinePC(&'a str),
    RawOuter(&'a str),
    Line(&'a str),
}

impl<'a> UmlMeta<'a> {
    pub fn parse(str: &'a str) -> Self {
        match str.split_once(' ') {
            Some(("INNER_CLASS_LINE_NOTE", msg)) => UmlMeta::InnerClassNote(msg),
            Some(("UML_INNER_CLASS_LINE_P_C", msg)) => UmlMeta::InnerClassLinePC(msg),
            Some(("UML_RAW_OUTER", msg)) => UmlMeta::RawOuter(msg),
            Some(("UML_LINE", msg)) => UmlMeta::Line(msg),
            Some(("HIDE", "")) => UmlMeta::Hide,
            _ => UmlMeta::Invalid(str),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
    Ident(&'a str),
    Comment(&'a str),
    Annotation(&'a str),

    UmlMeta(UmlMeta<'a>),

    Comma,
    LPar,
    RPar,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LAngle,
    RAngle,
    Semicolon,
    And,
    Star,
    QuestionMark,
    Dot,
    DotDotDot,

    Extends,
    Super,
    Implements,
    Record,
    Throws,
    Package,
    Class,
    Interface,
    Enum,
    Import,
    Public,
    Protected,
    Private,
    Permits,

    Static,
    Abstract,
    Synchronized,
    Transient,
    Volatile,
    Final,
    Native,
    Default,
    StrictFP,
    Sealed,
    NonSealed,

    Ignore,
}

#[derive(Clone, Copy)]
pub struct Tokenizer<'a>(&'a str, usize);

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, data.len())
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (Token<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        lexer! {
            fn next_token(text: 'a) -> Token<'a>;

            "public" => Token::Public,
            "protected" => Token::Protected,
            "private" => Token::Private,

            "static" => Token::Static,
            "abstract" => Token::Abstract,
            "synchronized" => Token::Synchronized,
            "transient" => Token::Transient,
            "volatile" => Token::Volatile,
            "final" => Token::Final,
            "native" => Token::Native,
            "default" => Token::Default,
            "strictfp" => Token::StrictFP,
            "sealed" => Token::Sealed,
            "non-sealed" => Token::NonSealed,


            "permits" => Token::Permits,
            "package" => Token::Package,
            "class" => Token::Class,
            "record" => Token::Record,
            "enum" => Token::Enum,
            "import" => Token::Import,
            "implements" => Token::Implements,
            "interface" => Token::Interface,
            "throws" => Token::Throws,
            "extends" => Token::Extends,
            "super" => Token::Super,
            "@[a-zA-Z0-9_]*" => Token::Annotation(&text["@".len()..]),


            r#"/[*]UML_(~(.*[*]/.*))[*]/"# => Token::UmlMeta(UmlMeta::parse(&text["/*UML_".len()..text.len()-"*/".len()])),

            "{" => Token::LBrace,
            "}" => Token::RBrace,
            "\\(" => Token::LPar,
            "\\)" => Token::RPar,
            ";" => Token::Semicolon,
            "\\*" => Token::Star,
            "\\&" => Token::And,
            "\\.\\.\\." => Token::DotDotDot,
            "\\." => Token::Dot,
            "," => Token::Comma,
            "\\?" => Token::QuestionMark,
            "<" => Token::LAngle,
            ">" => Token::RAngle,
            "\\[" => Token::LBracket,
            "\\]" => Token::RBracket,


            r#"[a-zA-Z_][a-zA-Z0-9_]*"# => Token::Ident(text),

            // comments
            r#"/[*](~(.*[*]/.*))[*]/"# => Token::Ignore,
            r#"//[^\n]*"# => Token::Ignore,
            // string / chars
            r#""([^"]|\\")*""# => Token::Ignore,
            r#"'"([^']|\\')*'"# => Token::Ignore,

            r#"."# => Token::Ignore,
        }

        loop {
            if let Some((tok, next)) = next_token(self.0) {
                let range = self.1 - self.0.len()..self.1 - next.len();
                self.0 = next;
                match tok {
                    Token::Comment(_) | Token::Ignore => continue,
                    _ => {}
                }

                break Some((tok, range));
            } else {
                break None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Peek2<T: Iterator> {
    iter: T,
    peek: [Option<Option<T::Item>>; 2],
}

pub trait Peek2able<T: Iterator> {
    fn peek2able(self) -> Peek2<T>;
}

impl<T: Iterator> Peek2able<T> for T {
    fn peek2able(self) -> Peek2<T> {
        Peek2 {
            iter: self,
            peek: std::array::from_fn(|_| None),
        }
    }
}

impl<T: Iterator> Peek2<T> {
    pub fn peek(&mut self) -> Option<&T::Item> {
        self.peek[0]
            .get_or_insert_with(|| self.iter.next())
            .as_ref()
    }

    pub fn peek_second(&mut self) -> Option<&T::Item> {
        if self.peek[0].is_none() {
            self.peek[0] = Some(self.iter.next());
        }
        self.peek[1]
            .get_or_insert_with(|| self.iter.next())
            .as_ref()
    }

    pub fn peek_both(&mut self) -> (Option<&T::Item>, Option<&T::Item>) {
        if self.peek[0].is_none() {
            self.peek[0] = Some(self.iter.next());
        }
        if self.peek[1].is_none() {
            self.peek[1] = Some(self.iter.next());
        }
        (
            self.peek[0].as_ref().unwrap().as_ref(),
            self.peek[1].as_ref().unwrap().as_ref(),
        )
    }
}

impl<T: Iterator> Iterator for Peek2<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.peek[0].take() {
            self.peek[0] = self.peek[1].take();
            item
        } else {
            self.iter.next()
        }
    }
}

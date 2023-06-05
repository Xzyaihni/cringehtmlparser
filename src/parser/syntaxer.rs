use std::iter::Peekable;

use lexer::{
    Lexeme,
    Lexer
};

pub use lexer::TextIterInner;

mod lexer;


fn unexpected_lexeme(lexeme: Option<Lexeme>, expected: &str) -> !
{
    let lexeme = match lexeme
    {
        Some(lexeme) => format!("Lexeme::{lexeme:?}"),
        None => "none".to_owned()
    };

    panic!("unexpected {}, expected Lexeme::{}", lexeme, expected)
}

#[derive(Debug, Clone)]
pub enum Leaf
{
    Body(ElementBody),
    Content(String),
    End(ElementEnd)
}

impl Leaf
{
    pub fn parse(mut lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        match lexemes.next()
        {
            Some(Lexeme::BracketLeft) => (),
            Some(Lexeme::Identifier(content)) => return Self::Content(content),
            x => unexpected_lexeme(x, "BracketLeft")
        }

        let mut lexemes = lexemes.take_while(|lexeme| *lexeme != Lexeme::BracketRight);
        let leaf = Self::parse_leaf(&mut lexemes);

        leaf
    }

    fn parse_leaf(mut lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        let name = match lexemes.next()
        {
            Some(Lexeme::Identifier(name)) => name,
            x => unexpected_lexeme(x, "Identifier")
        };

        if name.chars().next() == Some('/')
        {
            let end = ElementEnd::new(name.chars().skip(1).collect());

            // consume all the lexemes
            lexemes.for_each(drop);

            return Self::End(end);
        }

        let body = ElementBody::parse(name, &mut lexemes);

        Self::Body(body)
    }
}

#[derive(Debug, Clone)]
pub struct Tag
{
    name: String,
    content: String
}

impl Tag
{
    pub fn parse(mut lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        let name = match lexemes.next()
        {
            Some(Lexeme::Identifier(name)) => name,
            x => unexpected_lexeme(x, "Identifier")
        };

        match lexemes.next()
        {
            Some(Lexeme::Equals) => (),
            x => unexpected_lexeme(x, "Equals")
        }

        let content = match lexemes.next()
        {
            Some(Lexeme::Literal(content)) => content,
            x => unexpected_lexeme(x, "Literal")
        };

        Self{name, content}
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }

    pub fn content(&self) -> &str
    {
        &self.content
    }
}

#[derive(Debug, Clone)]
pub struct ElementBody
{
    pub name: String,
    pub tags: Box<[Tag]>
}

impl ElementBody
{
    pub fn parse(name: String, lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        let mut tags = Vec::new();

        let mut lexemes = lexemes.peekable();
        while let Some(_) = lexemes.peek()
        {
            let tag = Tag::parse(&mut lexemes);

            tags.push(tag);
        }

        let tags = tags.into_boxed_slice();

        Self{name, tags}
    }
}

#[derive(Debug, Clone)]
pub struct ElementEnd
{
    pub name: String
}

impl ElementEnd
{
    pub fn new(name: String) -> Self
    {
        Self{name}
    }
}

pub struct Syntaxer<'a>
{
    lexer: Peekable<Lexer<'a>>
}

impl<'a> Syntaxer<'a>
{
    pub fn new(text: TextIterInner<'a>) -> Self
    {
        let lexer = Lexer::new(text).peekable();

        Self{lexer}
    }
}

impl<'a> Iterator for Syntaxer<'a>
{
    type Item = Leaf;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.lexer.peek().is_some()
        {
            let leaf = Leaf::parse(&mut self.lexer);

            Some(leaf)
        } else
        {
            None
        }
    }
}
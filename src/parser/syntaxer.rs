use std::iter::Peekable;

use lexer::{
    Lexeme,
    LexemeType,
    Lexer
};

pub use lexer::TextIterInner;

mod lexer;


fn unexpected_lexeme(lexeme: impl Into<Option<Lexeme>>, expected: &str) -> !
{
    let lexeme = lexeme.into();
    let name = match lexeme
    {
        Some(ref lexeme) => format!("{lexeme:?}"),
        None => "none".to_owned()
    };

    match lexeme
    {
        Some(lexeme) =>
        {
            let line_num = lexeme.line();
            panic!("line {}: unexpected {}, expected {}", line_num, name, expected)
        },
        None =>
        {
            panic!("unexpected {}, expected {}", name, expected)
        }
    }
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
    pub fn parse(mut s_lexemes: impl Iterator<Item=Lexeme>) -> (Option<Self>, Self)
    {
        match s_lexemes.next()
        {
            Some(x) =>
            {
                match x.kind()
                {
                    LexemeType::BracketLeft => (),
                    // useless clone but who cares
                    LexemeType::Identifier(content) =>
                    {
                        return (None, Self::Content(content.clone()));
                    },
                    _ => unexpected_lexeme(x, "BracketLeft or Identifier")
                }
            },
            x => unexpected_lexeme(x, "BracketLeft or Identifier")
        }

        let mut has_preceding = false;
        let mut is_short_end = false;
        let mut lexemes = s_lexemes.by_ref().take_while(|lexeme|
        {
            match lexeme.kind()
            {
                LexemeType::BracketRight => false,
                LexemeType::EndSlash =>
                {
                    if has_preceding
                    {
                        is_short_end = true;

                        false
                    } else
                    {
                        true
                    }
                },
                _ =>
                {
                    has_preceding = true;
                    true
                }
            }
        });

        let leaf = Self::parse_leaf(&mut lexemes);

        let optional_leaf = is_short_end.then(||
        {
            let lexeme = s_lexemes.next();

            match lexeme.as_ref().map(|l| l.kind())
            {
                Some(LexemeType::BracketRight) =>
                {
                    let (name, line) = match &leaf
                    {
                        Self::Body(x) =>
                        {
                            (x.name.clone(), x.line)
                        },
                        Self::End(x) =>
                        {
                            (x.name.clone(), x.line)
                        },
                        Self::Content(_) => unreachable!()
                    };

                    let end = ElementEnd::new(name, line);

                    Self::End(end)
                },
                _ => unexpected_lexeme(lexeme, "BracketRight")
            }
        });

        (optional_leaf, leaf)
    }

    fn parse_leaf(mut lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        match lexemes.next()
        {
            Some(x) =>
            {
                match x.kind()
                {
                    LexemeType::Identifier(name) =>
                    {
                        let body = ElementBody::parse(name.clone(), x.line(), &mut lexemes);

                        Self::Body(body)
                    },
                    LexemeType::EndSlash =>
                    {
                        Self::parse_ending(lexemes)
                    },
                    _ => unexpected_lexeme(x, "Identifier or EndSlash")
                }
            }
            x => unexpected_lexeme(x, "Identifier or EndSlash")
        }
    }

    fn parse_ending(mut lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        match lexemes.next()
        {
            Some(lexeme) =>
            {
                match lexeme.kind()
                {
                    LexemeType::Identifier(name) =>
                    {
                        let end = ElementEnd::new(name.clone(), lexeme.line());

                        // consume all the lexemes
                        lexemes.for_each(drop);

                        Self::End(end)
                    },
                    _ => unexpected_lexeme(lexeme, "Identifier")
                }
            },
            x => unexpected_lexeme(x, "Identifier")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tag
{
    name: String,
    content: Option<String>
}

impl Tag
{
    pub fn parse(mut lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        let name = match lexemes.next()
        {
            Some(x) =>
            {
                match x.kind()
                {
                    LexemeType::Identifier(name) => name.clone(),
                    _ => unexpected_lexeme(x, "Identifier")
                }
            }
            x => unexpected_lexeme(x, "Identifier")
        };

        let content = if let Some(LexemeType::Equals) = lexemes.next().map(|l| l.kind().clone())
        {
            match lexemes.next()
            {
                Some(x) =>
                {
                    match x.kind()
                    {
                        LexemeType::Literal(content) => Some(content.clone()),
                        _ => unexpected_lexeme(x, "Literal")
                    }
                },
                x => unexpected_lexeme(x, "Literal")
            }
        } else
        {
            None
        };

        Self{name, content}
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }

    pub fn content(&self) -> &Option<String>
    {
        &self.content
    }
}

#[derive(Debug, Clone)]
pub struct ElementBody
{
    pub name: String,
    pub line: u64,
    pub tags: Box<[Tag]>
}

impl ElementBody
{
    pub fn parse(name: String, line: u64, lexemes: impl Iterator<Item=Lexeme>) -> Self
    {
        let mut tags = Vec::new();

        let mut lexemes = lexemes.peekable();
        while let Some(_) = lexemes.peek()
        {
            let tag = Tag::parse(&mut lexemes);

            tags.push(tag);
        }

        let tags = tags.into_boxed_slice();

        Self{name, line, tags}
    }
}

#[derive(Debug, Clone)]
pub struct ElementEnd
{
    pub name: String,
    pub line: u64
}

impl ElementEnd
{
    pub fn new(name: String, line: u64) -> Self
    {
        Self{name, line}
    }
}

pub struct Syntaxer<'a>
{
    cached_leaf: Option<Leaf>,
    lexer: Peekable<Lexer<'a>>
}

impl<'a> Syntaxer<'a>
{
    pub fn new(text: TextIterInner<'a>) -> Self
    {
        let lexer = Lexer::new(text).peekable();

        Self{cached_leaf: None, lexer}
    }
}

impl<'a> Iterator for Syntaxer<'a>
{
    type Item = Leaf;

    fn next(&mut self) -> Option<Self::Item>
    {
        if let Some(leaf) = self.cached_leaf.take()
        {
            return Some(leaf);
        }

        if self.lexer.peek().is_some()
        {
            let (optional_leaf, leaf) = Leaf::parse(&mut self.lexer);

            self.cached_leaf = optional_leaf;

            Some(leaf)
        } else
        {
            None
        }
    }
}

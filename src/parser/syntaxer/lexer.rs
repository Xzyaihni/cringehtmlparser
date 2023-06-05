use std::{
    str::Chars,
    iter::Peekable
};


enum Action
{
    ReturnLexeme(Lexeme),
    StopConsume,
    Stop,
    Skip,
    Continue
}

#[derive(Debug, Default)]
struct State
{
    is_content: bool,
    is_literal: bool
}

struct LexemeParser<'a, 'b>
{
    state: &'b mut State,
    collected: String,
    text: &'b mut TextIter<'a>
}

impl<'a, 'b> LexemeParser<'a, 'b>
{
    fn new(state: &'b mut State, text: &'b mut TextIter<'a>) -> Self
    {
        let collected = String::new();

        Self{state, collected, text}
    }

    fn parse(mut self) -> Lexeme
    {
        loop
        {
            match self.text.peek()
            {
                Some(&c) =>
                {
                    let action = self.parse_char(c);

                    match action
                    {
                        Action::ReturnLexeme(lexeme) =>
                        {
                            self.text.next();

                            if !self.collected.is_empty()
                            {
                                panic!("unparsed text: {}", self.collected);
                            }

                            return lexeme;
                        },
                        Action::StopConsume =>
                        {
                            self.text.next();
                            break;
                        },
                        Action::Stop => break,
                        Action::Skip =>
                        {
                            self.text.next();
                            continue;
                        },
                        Action::Continue => ()
                    }

                    self.text.next();
                    self.collected.push(c);
                },
                None => break
            }
        }

        self.parse_content()
    }

    fn parse_char(&mut self, c: char) -> Action
    {
        if self.state.is_literal
        {
            if c == '"'
            {
                self.state.is_literal = false;

                return Action::StopConsume;
            }

            return Action::Continue;
        }

        if self.state.is_content
        {
            if c == '<'
            {
                self.state.is_content = false;

                if !self.collected.is_empty()
                {
                    return Action::Stop;
                }
            } else
            {
                return Action::Continue;
            }
        }

        match c
        {
            '<' =>
            {
                if self.collected.is_empty()
                {
                    return Action::ReturnLexeme(Lexeme::BracketLeft);
                } else
                {
                    return Action::Stop;
                }
            },
            '>' =>
            {
                if self.collected.is_empty()
                {
                    self.state.is_content = true;

                    return Action::ReturnLexeme(Lexeme::BracketRight);
                } else
                {
                    return Action::Stop;
                }
            },
            '=' =>
            {
                return if self.collected.is_empty()
                {
                    Action::ReturnLexeme(Lexeme::Equals)
                } else
                {
                    Action::Stop
                };
            },
            '"' =>
            {
                self.state.is_literal = true;

                return Action::Continue;
            },
            c if c.is_whitespace() =>
            {
                return if self.collected.is_empty()
                {
                    Action::Skip
                } else
                {
                    Action::StopConsume
                };
            },
            _ => ()
        }

        Action::Continue
    }

    fn parse_content(self) -> Lexeme
    {
        if self.collected.chars().next() == Some('"')
        {
            let literal = self.collected.chars().skip(1).collect::<String>();
            Lexeme::Literal(literal)
        } else
        {
            Lexeme::Identifier(self.collected)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Lexeme
{
    BracketLeft,
    BracketRight,
    Identifier(String),
    Equals,
    Literal(String)
}

impl Lexeme
{
    fn parse(state: &mut State, text: &mut TextIter) -> Self
    {
        LexemeParser::new(state, text).parse()
    }
}

pub type TextIterInner<'a> = Chars<'a>;
type TextIter<'a> = Peekable<TextIterInner<'a>>;

pub struct Lexer<'a>
{
    state: State,
    text: TextIter<'a>
}

impl<'a> Lexer<'a>
{
    pub fn new(text: TextIterInner<'a>) -> Self
    {
        let state = State::default();

        Self{state, text: text.peekable()}
    }
}

impl<'a> Iterator for Lexer<'a>
{
    type Item = Lexeme;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.text.peek().is_some()
        {
            let lexeme = Lexeme::parse(&mut self.state, &mut self.text);

            Some(lexeme)
        } else
        {
            None
        }
    }
}
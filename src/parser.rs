use std::{
    iter::Peekable,
    ops::Index
};

use syntaxer::{
    Tag,
    Syntaxer,
    Leaf,
    TextIterInner
};

mod syntaxer;


fn unexpected_leaf(leaf: Option<Leaf>, expected: &str) -> !
{
    let leaf = match leaf
    {
        Some(leaf) => format!("Leaf::{leaf:?}"),
        None => "none".to_owned()
    };

    panic!("unexpected {}, expected Leaf::{}", leaf, expected)
}

#[derive(Debug)]
pub enum Child
{
    Element(Element),
    Text(String)
}

impl Child
{
    pub fn parse(mut leaves: &mut Peekable<Syntaxer<'_>>) -> Option<Self>
    {
        Some(match leaves.peek()
        {
            Some(Leaf::Content(_)) =>
            {
                let text = match leaves.next()
                {
                    Some(Leaf::Content(text)) => text,
                    _ => unreachable!()
                };

                if text.trim().is_empty()
                {
                    return None;
                }

                Self::Text(text)
            },
            Some(Leaf::Body(_)) => Self::Element(Element::parse(&mut leaves)),
            leaf => unexpected_leaf(leaf.cloned(), "{Content, Body}")
        })
    }

    pub fn element(&self) -> Option<&Element>
    {
        match self
        {
            Child::Element(ref element) => Some(element),
            _ => None
        }
    }

    pub fn text(&self) -> Option<&str>
    {
        match self
        {
            Child::Text(ref text) => Some(text),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Element
{
    name: String,
    tags: Box<[Tag]>,
    children: Box<[Child]>
}

impl Element
{
    pub fn parse(mut leaves: &mut Peekable<Syntaxer<'_>>) -> Self
    {
        let body = match leaves.next()
        {
            Some(Leaf::Body(body)) => body,
            leaf => unexpected_leaf(leaf, "Body")
        };

        let mut children = Vec::new();

        let childless = [
            "area",
            "base",
            "br",
            "col",
            "command",
            "embed",
            "hr",
            "img",
            "input",
            "keygen",
            "link",
            "meta",
            "param",
            "source",
            "track",
            "wbr"
        ];

        let has_children = !childless.contains(&body.name.as_ref());

        if has_children
        {
            while let Some(leaf) = leaves.peek()
            {
                match leaf
                {
                    Leaf::End(end) =>
                    {
                        if end.name != body.name
                        {
                            panic!(
                                "expected {} end (line {}), got {} end (line {})",
                                body.name,
                                body.line,
                                end.name,
                                end.line
                            );
                        }

                        leaves.next();

                        break;
                    },
                    _ => ()
                }

                if let Some(child) = Child::parse(&mut leaves)
                {
                    children.push(child);
                }
            }
        }

        let children = children.into_boxed_slice();

        Self{name: body.name, tags: body.tags, children}
    }

    #[allow(dead_code)]
    pub fn get_name(&self, name: &str) -> Option<&Element>
    {
        self.children.iter().find(|child|
        {
            match child
            {
                Child::Element(x) =>
                {
                    x.name == name
                },
                _ => false
            }
        }).map(|child|
        {
            match child
            {
                Child::Element(x) => x,
                _ => unreachable!()
            }
        })
    }

    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<&Child>
    {
        self.children.get(index)
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }

    pub fn tags(&self) -> &[Tag]
    {
        &self.tags
    }

    pub fn children(&self) -> &[Child]
    {
        &self.children
    }
}

impl Index<usize> for Element
{
    type Output = Child;

    fn index(&self, index: usize) -> &Self::Output
    {
        self.get(index).unwrap()
    }
}

pub struct Parser<'a>
{
    syntaxer: Syntaxer<'a>
}

impl<'a> Parser<'a>
{
    pub fn new(text: TextIterInner<'a>) -> Self
    {
        let syntaxer = Syntaxer::new(text);

        Self{syntaxer}
    }

    pub fn parse(self) -> Element
    {
        let mut leaves = self.syntaxer.peekable();

        Element::parse(&mut leaves)
    }
}

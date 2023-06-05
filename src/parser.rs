use std::{
    iter::Peekable
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
    pub fn parse(mut leaves: &mut Peekable<Syntaxer<'_>>) -> Self
    {
        match leaves.peek()
        {
            Some(Leaf::Content(_)) =>
            {
                let text = match leaves.next()
                {
                    Some(Leaf::Content(text)) => text,
                    _ => unreachable!()
                };

                Self::Text(text)
            },
            Some(Leaf::Body(_)) => Self::Element(Element::parse(&mut leaves)),
            leaf => unexpected_leaf(leaf.cloned(), "{Content, Body}")
        }
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
        let (name, tags) = match leaves.next()
        {
            Some(Leaf::Body(body)) => (body.name, body.tags),
            leaf => unexpected_leaf(leaf, "Body")
        };

        let mut children = Vec::new();

        let childless = ["img"];
        let has_children = !childless.contains(&name.as_ref());

        if has_children
        {
            while let Some(leaf) = leaves.peek()
            {
                match leaf
                {
                    Leaf::End(end) =>
                    {
                        if end.name != name
                        {
                            panic!("expected {} end, got {} end", name, end.name);
                        }

                        leaves.next();

                        break;
                    },
                    _ => ()

                }

                let child = Child::parse(&mut leaves);

                children.push(child);
            }
        }

        let children = children.into_boxed_slice();

        Self{name, tags, children}
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
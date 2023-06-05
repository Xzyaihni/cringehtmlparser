use std::{
    io,
    collections::VecDeque
};

use crate::parser::{
    Child,
    Element
};


fn select_element<'a>(html: &'a Element, indices: &[usize]) -> &'a Element
{
    let mut element = html;

    for &index in indices
    {
        element = element.children()[index].element().unwrap();
    }

    element
}

fn print_info(element: &Element)
{
    let name = element.name();
    let tags = element.tags().iter().map(|tag|
    {
        tag.name()
    }).fold(String::new(), |mut acc, this|
    {
        acc.push(' ');

        acc + this
    }).chars().skip(1).collect::<String>();

    println!("<{name} {tags}>");

    for (index, child) in element.children().iter().enumerate()
    {
        let content = match child
        {
            Child::Element(element) =>
            {
                let name = element.name();

                format!("<{name}>")
            },
            Child::Text(text) => text.clone()
        };

        println!("{:4}{content}", format!("{index}:"));
    }
}

pub fn explore(html: Element)
{
    let stdin = io::stdin();

    let mut selector_indices = VecDeque::new();

    println!("controls:");
    println!("b: back");
    println!("q: quit");
    println!("t tag-name: tag content");
    println!();

    loop
    {
        let this_element = select_element(&html, selector_indices.as_slices().0);

        print_info(this_element);

        let mut command = String::new();
        stdin.read_line(&mut command).unwrap();

        let command = command.trim();

        match command.to_lowercase().as_ref()
        {
            "b" | "back" =>
            {
                selector_indices.pop_back();
            },
            "q" | "quit" | "exit" => return,
            _ => ()
        }

        let tag_info_command = ["t", "tag", "tag_info"];

        let is_tag_info_command = tag_info_command.iter().any(|tag_command|
        {
            command.starts_with(tag_command)
        });

        if is_tag_info_command
        {
            let maybe_tag_name = command.split(char::is_whitespace).nth(1);

            match maybe_tag_name
            {
                Some(tag_name) =>
                {
                    let maybe_tag = this_element.tags().iter().find(|tag|
                    {
                        tag.name() == tag_name
                    });

                    match maybe_tag
                    {
                        Some(tag) =>
                        {
                            let name = tag.name();
                            let content = tag.content();

                            println!("{name} = \"{content}\"");
                        },
                        None =>
                        {
                            println!("could not find tag named \"{tag_name}\"");
                            continue;
                        }
                    }
                },
                None =>
                {
                    println!("enter the tag u want to inspect");
                    continue;
                }
            }
        }

        let index: Option<usize> = command.parse().ok();

        if let Some(index) = index
        {
            let max_index = this_element.children().len();

            if index >= max_index
            {
                continue;
            }

            if this_element.children()[index].text().is_some()
            {
                continue;
            }

            selector_indices.push_back(index);
        }
    }
}
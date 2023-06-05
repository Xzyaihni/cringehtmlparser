use std::{
    fs,
    env,
    process
};

use parser::{
    Element,
    Parser
};

mod parser;
mod explorer;


fn complain(message: &str) -> !
{
    eprintln!("{message}");

    process::exit(1)
}

fn main()
{
    let mut args = env::args().skip(1);
    let filepath = args.next()
        .unwrap_or_else(|| complain("pls provide a path as argument"));

    let use_explore = args.next().map(|mode|
    {
        mode.to_lowercase() == "explore"
    }).unwrap_or(false);

    let data = fs::read_to_string(filepath)
        .unwrap_or_else(|err| complain(&format!("error reading file: {err:?}")));

    let parser = Parser::new(data.chars());

    let html = parser.parse();

    if use_explore
    {
        explorer::explore(html);
    } else
    {
        normal_main(html);
    }
}

fn normal_main(html: Element)
{
    // nobody will know wut i wrote this for heheheheh

    let images = html.children().iter().filter_map(|child|
    {
        let element = child.element()?;

        let element = (element.name() == "button").then(|| element)?;

        let img = element.children()[0].element()?.children()[0].element()?;

        let tag = img.tags().iter().find(|tag| tag.name() == "src")?;

        let content = tag.content();

        let url = content.chars().skip(2).collect::<Vec<_>>();

        let new_num = '3';
        let url = url.into_iter().rev().enumerate().map(|(index, c)|
        {
            if index == 2
            {
                new_num
            } else
            {
                c
            }
        }).rev().collect::<String>();

        let url = "https://".to_owned() + &url;

        Some(url)
    });

    images.for_each(|image| println!("{image}"));
}
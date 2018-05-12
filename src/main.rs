extern crate cursive;

use std::collections::VecDeque;

use cursive::Cursive;
use cursive::view::View;
use cursive::traits::Identifiable;
use cursive::views::*;

struct BufferView {
    content: VecDeque<String>
}

impl BufferView
{
    fn new() -> Self
    {
        BufferView{
            content: VecDeque::default()
        }
    }

    fn add_content(&mut self, line: &str)
    {
        self.content.push_back(line.to_owned());
    }
}

impl View for BufferView
{
    fn draw(&self, printer: &cursive::Printer)
    {
        for (i, line) in self.content.iter().rev().take(printer.size.y).enumerate()
        {
            printer.print((0, printer.size.y - (i+1)), line);
        }
    }
}

fn input_cb(ctx: &mut Cursive, input: &str)
{
    match input
    {
        "/quit" => ctx.quit(),
        _ => 
        {
            ctx.find_id::<BufferView>("text").unwrap().add_content(input);
            ctx.find_id::<EditView>("input").unwrap().set_content("");
        }
    }
}

fn main() -> Result<(), std::io::Error>
{
    let mut context=Cursive::default();
    let mut layout=LinearLayout::vertical();

    context.add_global_callback('q', Cursive::quit);
    layout.add_child(BoxView::with_full_screen(
        BufferView::new().with_id("text")
    ));
    layout.add_child(EditView::new()
        .on_submit_mut(input_cb)
        .with_id("input")
    );

    //context.find_id::<BufferView>("text").unwrap().add_content("Hello World");

    context.add_layer(
        Panel::new(layout)
    );

    context.run();
    Ok(())
}
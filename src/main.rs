extern crate tui;
extern crate termion;

use termion::event;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Widget, Block, Borders, List, Item, Paragraph};
use tui::layout::{Group, Size, Direction};
use std::io;

fn draw_ui(term: &mut Terminal<RawBackend>) -> Result<(), io::Error>
{
    let size=term.size()?;
    term.clear()?;

    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Min(0), Size::Fixed(1)])
        .render(term, &size, |t, chunks|
        {
            List::new(vec![Item::Data("chat window"), Item::Data("another line  ")].into_iter())
                .render(t, &chunks[0]);
            Paragraph::default()
                .text("> ")
                .render(t, &chunks[1]);
        });
    term.draw()?;
    Ok(())
}

fn main() -> Result<(), io::Error>
{
    let backend=RawBackend::new().unwrap();
    let mut term=Terminal::new(backend).unwrap();

    draw_ui(&mut term)?;
    
    Ok(())
}

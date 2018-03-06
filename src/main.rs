extern crate pancurses;

use pancurses::{initscr,endwin,noecho,Input};
use std::vec::Vec;
use std::iter::Iterator;

fn print_log(window: &pancurses::Window, buf: &Vec<String>)
{
    let mut cur_y=window.get_max_y()-2;
    //cur_y=cur_y-2;
    for line in buf.iter().rev()
    {
        window.mv(cur_y, 0);
        window.clrtoeol();
        window.printw(line);
        cur_y=cur_y-1;
        if cur_y==0
        {
            break;
        }
    }
}

fn main() {
    let window = initscr();
    let prompt_y=window.get_max_y()-1;
    let prompt="> ";
    window.printw("Hello curses!");
    window.mvprintw(prompt_y, 0, prompt);
    window.refresh();
    window.keypad(true);
    noecho();
    let mut buf:Vec<String> = Vec::new();
    let mut input:String="".to_owned();
    loop {
      match window.getch() {
          Some(Input::Character('\n')) => {
              buf.push(input);
              input="".to_owned();
              print_log(&window, &buf);
              window.mvprintw(prompt_y, 0, prompt); 
              window.clrtoeol(); 
            }
          Some(Input::Character(c)) => {
              window.addch(c); 
              input.push(c); 
            },
          Some(Input::KeyDC) => break,
          Some(input_str) => { window.addstr(&format!("{:?}", input_str)); },
          None => ()
      }
  }
    endwin();
}

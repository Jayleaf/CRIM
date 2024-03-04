/*

File of public utility functions that may need to be used on many programs. Don't want to rewrite them, so store them here.

*/

use colored::{Colorize, CustomColor};
use std::io::{self, Write};

pub enum Position
{
    Center,
    Left,
    Right
}

pub fn addl_message(message: &str, color: &str)
{
    println!("{}", message.color(color));
}

pub fn clear()
{
    // clear console without having to type out that nasty stuff. Used to print a logo before UI change.
    print!("\x1b[2J");
}

pub fn pad_string(string: &str, length: usize) -> String
{
    let mut string = string.to_string();
    let length: isize = length as isize - string.len() as isize;

    for _ in 0..length
    {
        string.push(' ');
    }

    string
}

pub fn format_string_ui(string: &str, length: usize, pos: &Position) -> String
{
    let mut string: String = String::from(string);
    let string_length: usize = string.len();
    if string_length > length
    {
        panic!("UI string longer than UI width.")
    };
    match pos
    {
        Position::Center =>
        {
            let rng: isize = (length as isize) - (string_length as isize); // thanks rust for not letting me subtract from a usize!
            let mut temp_str: String = String::new();
            let rng: f64 = rng as f64 / 2 as f64;
            let wall_char = dotenv::var("UI_WALL_CHAR").unwrap().parse::<char>().unwrap();
            temp_str.push(wall_char);
            for _ in 0..rng as usize
            {
                temp_str.push(' ');
            }
            temp_str.push_str(&string);
            for _ in 0..rng.ceil() as usize
            {
                // ceil for odd numbers
                temp_str.push(' ');
            }
            temp_str.push(wall_char);
            string = temp_str;
        }
        Position::Left =>
        {
            for _ in 0..length
            {
                string.push(' ');
            }
        }
        Position::Right =>
        {
            for _ in 0..length
            {
                string.insert(0, ' ');
            }
        }
    }
    string
}

pub fn grab_int_input(msg: Option<&str>, lim: i32) -> i32
{
    let mut input: String = String::new();
    if let Some(msg) = msg
    {
        println!("{}", msg);
    }
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line.");
    match input.trim().parse()
    {
        Ok(num) =>
        {
            if num <= lim && num > 0
            {
                num
            }
            else
            {
                grab_int_input(msg, lim)
            }
        }
        Err(_) =>
        {
            if input.trim().to_lowercase() == "b"
            {
                0
            }
            else
            {
                grab_int_input(msg, lim)
            }
        }
    }
}

pub fn grab_str_input(msg: Option<&str>) -> String
{
    let mut input: String = String::new();
    if let Some(msg) = msg
    {
        println!("{}", msg);
    }
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line.");
    input = String::from(input.trim());
    input
}

pub fn grab_opt(msg: Option<&str>, valid_options: Vec<&str>) -> (String, String)
{
    loop
    {
        let mut input: String = String::new();
        if let Some(msg) = msg
        {
            println!("{}", msg);
        }
        std::io::stdin().read_line(&mut input).expect("Failed to read line.");
        for opt in &valid_options
        {
            if input.starts_with(*opt)
            {
                return (String::from(*opt), String::from(input.trim_start_matches(*opt).trim()));
            }
        }
        continue;
    }
}

pub fn create_ui(text: Vec<&str>, position: Position)
{
    let floor_char = dotenv::var("UI_FLOOR_CHAR").unwrap().parse::<char>().unwrap();
    let ui_width: usize = dotenv::var("UI_WIDTH").unwrap().parse::<usize>().unwrap();
    let title: String = {
        let mut tempstr: String = String::new();
        for _ in 0..ui_width
        {
            tempstr.push(floor_char);
        }
        tempstr
    };
    println!("{}", format_string_ui(&title, ui_width, &position));
    for line in text
    {
        println!("{}", format_string_ui(line, ui_width, &position));
    }
    println!("{}", format_string_ui(&title, ui_width, &position));
}

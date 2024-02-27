/*

File of public utility functions that may need to be used on many programs. Don't want to rewrite them, so store them here.

*/

use std::io::{self, Write};

use colored::{Colorize, CustomColor};

pub enum Position
{
    Center,
    Left,
    Right
}

pub fn clear(addl_message: Option<&str>)
{
    /*
       Clears terminal and throws the logo up. Because, you know, it's cool.
       Also lets you print an additional message if you wanted to convey extra info.
    */
    print!("\x1b[2J");
    println!(
        "{}",
        r"
	 _____  _____ _____ __  __
	/  __ \| ___ \_   _|  \/  |
	| /  \/| |_/ / | | | .  . |
	| |    |    /  | | | |\/| |
	| \__/\| |\ \ _| |_| |  | |
	 \____/\_| \_|\___/\_|  |_/
	"
        .custom_color(CustomColor { r: 100, g: 0, b: 0 })
    );

    if let Some(message) = addl_message
    {
        println!("{}", message.red());
    }
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
    match pos
    {
        Position::Center =>
        {
            let rng: isize = (length as isize) - (string_length as isize); // thanks rust for not letting me subtract from a usize!
            let mut temp_str: String = String::new();
            let rng: f64 = rng as f64 / 2 as f64;
            temp_str.push('*');
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
            temp_str.push('*');
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
        Ok(num) => {
            if num <= lim && num > 0
            {
                num
            }
            else
            {
                grab_int_input(msg, lim)
            }
        }
        Err(_) => grab_int_input(msg, lim),
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

pub fn grab_opt(msg: Option<&str>, valid_options: Vec<&str>) -> String
{
    loop {
        let mut input: String = String::new();
        if let Some(msg) = msg
        {
            println!("{}", msg);
        }
        std::io::stdin().read_line(&mut input).expect("Failed to read line.");
        input = String::from(input.trim());
        for opt in &valid_options
        {
            if input == *opt
            {
                return input;
            }
        }
        continue;
    }
}

pub fn create_ui(text: Vec<&str>, position: Position)
{
    print!("\x1b[2J");
    let ui_width: usize = dotenv::var("UI_WIDTH").unwrap().parse::<usize>().unwrap();
    let title: String = {
        let mut tempstr: String = String::new();
        for _ in 0..ui_width
        {
            tempstr.push('-');
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

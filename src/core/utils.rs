/*

File of public utility functions that may need to be used on many programs. Don't want to rewrite them, so store them here.

*/

use colored::Colorize;
use rand::RngCore;
use std::io::{self, Write};

pub enum Position
{
    Center,
    // Left and right can be implemented in the future
}

/// Prints a message.
pub fn addl_message(message: &str, color: &str)
{
    println!("{}", message.color(color));
}

/// Clears terminal
pub fn clear()
{
    print!("\x1b[2J");
}

/// Formats a string to be usable in the UI. Internal function used by `create_ui()`
fn format_string_ui(string: &str, length: usize, pos: &Position) -> String
{
    let mut string: String = String::from(string);
    let string_length: usize = string.len();
    match pos
    {
        Position::Center =>
        {
            let rng: isize = (length as isize) - (string_length as isize); // thanks rust for not letting me subtract from a usize!
            let mut temp_str: String = String::new();
            let rng: f64 = rng as f64 / 2_f64;
            let wall_char = dotenv::var("UI_WALL_CHAR")
                .unwrap()
                .parse::<char>()
                .unwrap();
            temp_str.push(wall_char);
            // debug print!("{}", string_length);
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
    }
    string
}

/// Grab a string input.
/// 
/// Ex: 
/// ```rust
/// let input: String = utils::grab_str_input(Some("Enter your username: "));
/// println!("You entered: {}", input);
/// ```
pub fn grab_str_input(msg: Option<&str>) -> String
{
    let mut input: String = String::new();
    if let Some(msg) = msg
    {
        println!("{}", msg);
    }
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line.");
    input = String::from(input.trim());
    input
}

/// Grab an option input.
/// 
/// Ex:
/// ```rust
/// let opt: (String, String) = utils::grab_opt("Enter an action:", vec!["-a", "-b", "-c"]);
/// match opt.0.as_str()
/// {
///    "-a" => println!("You chose option A!"),
///    "-b" => println!("You chose option B!"),
///    "-c" => println!("You chose option C!")
/// }
pub fn grab_opt(msg: Option<&str>, mut valid_options: Vec<&str>) -> (String, String)
{
    valid_options.sort_by_key(|a| a.len());
    // sort this by length so that things with flags get read first.
    loop
    {
        let mut input: String = String::new();
        if let Some(msg) = msg
        {
            println!("{}", msg);
        }
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line.");
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

/// Creates a UI from a given vector of strings. Stacked vertically.
pub fn create_ui(text: &Vec<String>, position: Position)
{
    let floor_char = dotenv::var("UI_FLOOR_CHAR")
        .unwrap()
        .parse::<char>()
        .unwrap();
    let ui_width: usize = {
        let mut t: Vec<String> = Vec::clone(text);
        // make a copy of the text variable because it needs to be altered for finding the largest string.
        t.sort_by_key(|a| a.len());
        t.reverse();
        if dotenv::var("UI_DYNAMIC").unwrap() == "true"
        {
            // if ui is dynamic, set it to the width of the longest string and add some number for cleanliness.
            t[0].len() + 6
        }
        else
        {
            panic!("Given UI string length supersedes that of the set UI width. Either increase the UI width or turn on dynamic UI.");
        }
    };
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

/// Outputs a random hexadecimal from 4 bytes.
pub fn rand_hex() -> String
{
    let mut bytes = [0; 4];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

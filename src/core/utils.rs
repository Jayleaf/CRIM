/*

File of public utility functions that may need to be used on many programs. Don't want to rewrite them, so store them here.

*/

use std::io::{self, Write};

use colored::{Colorize, CustomColor};

pub fn clear(addl_message: Option<&str>) {
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

    if let Some(message) = addl_message {
        println!("{}", message.red());
    }
}

pub fn pad_string(string: &str, length: usize) -> String {
    let mut string = string.to_string();
    let length = length - string.len();

    for _ in 0..length {
        string.push(' ');
    }

    string
}

pub fn grab_input(msg: Option<&str>) -> String {
    let mut input: String = String::new();
    if let Some(msg) = msg
    {
        println!("{}", msg);
    }
    std::io::stdin().read_line(&mut input).expect("Failed to read line.");
    input = String::from(input.trim());
    io::stdout().flush().expect("flush error"); // will this ever even error? do i even need to do this? life's biggest questions.
    input
}

/*

File of public utility functions that may need to be used on many programs. Don't want to rewrite them, so store them here.

*/

use colored::{Colorize, CustomColor};

pub fn clear(addl_message: Option<String>)
{
    /*
       Clears terminal and throws the logo up. Because, you know, it's cool.
       Also lets you print an additional message if you wanted to convey extra info.
    */
    print!("{}[2J", 27 as char);
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

    if addl_message.is_some()
    {
        println!("{}", addl_message.unwrap().red());
    }
}

pub fn pad_string(mut string: String, mut length: i32) -> String
{
    let len: i32 = string.chars().count().try_into().unwrap();
    length -= len;
    for _i in 0..length
    {
        string.push(' ');
    }
    string
}

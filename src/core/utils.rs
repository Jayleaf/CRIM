/*

File of public utility functions that may need to be used on many programs. Don't want to rewrite them, so store them here.

*/

pub fn clear()
{
    print!("{}[2J", 27 as char);
    println!(r"
     _____  _____ _____ __  __
    /  __ \| ___ \_   _|  \/  |
    | /  \/| |_/ / | | | .  . |
    | |    |    /  | | | |\/| |
    | \__/\| |\ \ _| |_| |  | |
     \____/\_| \_|\___/\_|  |_/
     
     "
    
    );
}
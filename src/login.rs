extern crate dotenv;
use dotenv::dotenv;
use std::io;
use std::path::Path;
use serde_derive::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
struct Profile
{
    
    username: String,
    password: String,
    token: String
}

fn update_saved_profiles()
{

}

fn read_saved_profiles()
{

}

fn validate_login_info(username: String, password: String)
{
    // "Profiles" will be client-sided login information so you can quick-log into multiple accounts.
    // When you log in, this will eventually be checked against the server to ensure that the account you're using actually exists.

    println!("Function not implemented yet :3 | https://www.youtube.com/watch?v=YLZtw8_aLwA");
    
}


fn record_login_info()
{
    let mut username: String = String::new();
    let mut password: String = String::new();
    println!("Please enter your username: ");
    io::stdin().read_line(&mut username).expect("Uh oh! Failed to read the line.");
    println!("Please enter your password: ");
    io::stdin().read_line(&mut password).expect("Uh oh! Failed to read the line.");
    validate_login_info(username, password)

}

pub fn login_init()
{
    dotenv().ok();
    if dotenv::var("TOKEN").unwrap() == ""
    {
        println!("Looks like you're not logged in. Let's fix that.");
        crate::login::record_login_info();
    }
    else
    {
        println!("You're logged in.")
    }
}
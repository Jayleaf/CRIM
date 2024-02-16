extern crate dotenv;
use dotenv::dotenv;
use std::io;
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string;
use uuid::Uuid;
use super::utils;

/*

This file handles the login system of CRIM. 
Profiles.json is a local cache of accounts, to allow for quick sign in.
Any account that is being logged in with will be checked against the account database in the server so as to prevent fake accounts; registering is necessary.
Registering is not possible yet because I haven't set up the DB lol

*/


#[derive(Deserialize, Serialize, Debug)]
struct Profile
{
    
    username: String,
    password: String,
    account_uuid: String
}


fn validate_login_info(username: &str, password: &str)
{
    /*
    
        First, ensure that whatever profile we're trying to sign into is in profiles.json.
        Then, check it against the database to ensure the account exists.
        If successful, log in.

     */
    
}

fn register_profile()
{
    utils::clear();
    let mut username: String = String::new();
    println!("Enter the username for your new profile. This will be your display name. : ");
    io::stdin().read_line(&mut username).expect("Uh oh! Failed to read the line.");
    let mut password: String = String::new();
    println!("Enter the password for your new profile. : ");
    io::stdin().read_line(&mut password).expect("Uh oh! Failed to read the line.");
    username.pop(); 
    password.pop();

    let new_profile: Profile = Profile{username: String::from(username), password: String::from(password), account_uuid: Uuid::new_v4().to_string()};
    /*
    
        Code here will connect to the mongoDB server here.
        First we will check for an already-existing username, and if there is one the user will be prompted to create a new username.
        Then, we check for an already existing account uuid. This is just a safety measure since it's incredibly unlikely it will ever happen, but it's better to be safe than sorry.
        If all is clear, we register the account in the database.

     */
    let serialized_profile: Result<String, serde_json::Error> = to_string(&new_profile);
    utils::clear();
    if serialized_profile.is_ok()
    {
        println!("{}", serialized_profile.ok().unwrap())
    }
    else
    {
        println!("{:#?}", serialized_profile.err());
    }
    
    // save the data to profiles.json here.
}


pub fn login_init()
{
    utils::clear();
    dotenv().ok();
    if dotenv::var("UUID").unwrap() == ""
    {   
        println!("Looks like you're not logged in. Let's fix that. \n \n");
        println!("Register New Profile    (1)");
        println!("Select Existing Profile (2)");
        println!("Exit                    (3)");

        let mut selection: String = String::new();
        io::stdin().read_line(&mut selection).expect("Failed to read the line.");
        selection.pop();
        match selection.as_str()
        {
            "1" => register_profile(),
            "2" => println!("Awaiting functionality."),
            "3" => std::process::exit(0),
            _   => login_init()
        }
    }
    else
    {
        println!("You're logged in.")
    }
}
extern crate dotenv;
use super::mongo;
use super::utils;
use colored::Colorize;
use mongodb::{bson::doc, bson::to_document, bson::Document, sync::Client};
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

/*

This file handles the login system of CRIM.
Profiles.json is a local cache of accounts, to allow for quick sign in.
Any account that is being logged in with will be checked against the account database in the server so as to prevent fake accounts; registering is necessary.

*/

/*

Structs

*/

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
struct Profile
{
    username: String,
    password: String
}

impl Default for Profile
{
    fn default() -> Profile { Profile { username: String::new(), password: String::new() } }
}

#[derive(Deserialize, Serialize, Debug)]
struct ProfileContainer
{
    profiles: Vec<Profile>
}

#[derive(Deserialize, Serialize, Debug)]
struct Token
{
    token: String
}

/*

    Login-Specific Utility Functions

*/

fn deserialize_profile_data(holder: &mut ProfileContainer)
{
    /*
       Deserialize the data in the profiles json file, and return it.
    */

    let mut f: File = fs::File::open("src/userdata/profiles.json").unwrap();
    let mut data: String = String::new();
    f.read_to_string(&mut data).unwrap();
    // ^^ should not really ever fail. if it does, somebody tampered with profiles.json.
    let profiles: ProfileContainer = {
        let pc: Result<ProfileContainer, serde_json::Error> = serde_json::from_str(&data.as_str());
        match pc
        {
            Ok(p) => p,
            Err(_) => ProfileContainer { profiles: Vec::new() }
        }
    };

    *holder = profiles;
}

fn serialize_profile_data(container: ProfileContainer)
{
    /*
        Serializes profile data and writes it to the profiles json file.
    */

    let serialized_data: Result<String, serde_json::Error> = to_string(&container);
    match serialized_data
    {
        Err(_) => println!("Failed to serialize profile data."),
        Ok(data) =>
        {
            fs::write("src/userdata/profiles.json", data).expect("Failed to write profile data to file.");
            // will only fail if the directory path doesn't exist. If this does fail, it's worth panicking for.
        }
    }
}

fn validate_login_info(profile_to_be_validated: &Profile) -> bool
{
    /*

       Validates login information against the database.
       Maybe also instead of storing profile username and passwords, you could only store the username since they must be unique; passwords laying around is a risk.

    */

    let db: mongodb::sync::Database = mongo::get_database("CRIM");
    let coll: mongodb::sync::Collection<Document> = db.collection::<Document>("accounts");
    let query: Option<Document> = coll
        .find_one(doc! { "username": &profile_to_be_validated.username, "password": &profile_to_be_validated.password}, None)
        .unwrap();
    if !query.is_some()
    {
        return false;
    }

    true
}

/*
|
|				Interactable Functions
|
====================================================*/

fn register_profile(addl_message: Option<String>)
{
    /*
    |  Function to register a new profile.
    |  This function will prompt the user for a username and password, and then save it to profiles.json and the database, if the username is unique.
    |  If the username is not unique, the function will return to the start of the function.
    /===================================*/

    utils::clear(addl_message);
    
    let db = mongo::get_database("CRIM");
    let coll = db.collection::<Document>("accounts");
    let mut username: String = String::new();

	// grab username input
    println!("Enter the username for your new profile. This will be your display name. : ");
    io::stdin().read_line(&mut username).expect("Uh oh! Failed to read the line.");
    username = String::from(username.trim());

    // check username uniquity
    let unique_query:Result<Option<Document>, mongodb::error::Error>  = coll.find_one(doc! {"username": &username}, None);
	match unique_query
	{
		Ok(query_option) =>
		{
			if query_option.is_some() { register_profile(Some("Username already exists. Please try again.".to_string())); }
		}
		Err(_) => panic!("Failed to query database")
	};

    let mut password: String = String::new();
    println!("Enter the password for your new profile. : ");
    io::stdin().read_line(&mut password).expect("Uh oh! Failed to read the line.");
    password = String::from(password.trim());

    let new_profile: Profile = Profile { username: String::from(&username), password: String::from(&password) };
    //utils::clear();

    // save the data to profiles.json here.

    let mut deserialized_data: ProfileContainer = ProfileContainer { profiles: Vec::new() };
    deserialize_profile_data(&mut deserialized_data);
    deserialized_data.profiles.push(Profile::clone(&new_profile));
    serialize_profile_data(deserialized_data);

    /*
       mang- i mean mongo time!
    */

    let doc: Result<Document, mongodb::bson::ser::Error> = to_document(&serde_json::to_value(&new_profile).unwrap());
    // handle this better
    let pushed_doc: Result<mongodb::results::InsertOneResult, mongodb::error::Error> = coll.insert_one(doc.unwrap(), None);
    let token: mongodb::bson::Bson = pushed_doc.unwrap().inserted_id;
    // write the token to token.json using serde_json
    let token_obj: Token = Token { token: token.to_string() };
    let token_json_str = to_string(&token_obj).unwrap();
    fs::write("src/userdata/token.json", token_json_str).expect("Failed to write token to file. Please ensure you have a token.json file existing.");
    println!("Created profile. Validating...");
    let validation_status: bool = validate_login_info(&Profile::clone(&new_profile));
    if validation_status == true
    {
        println!("Profile Validated. Logging you in...");
        login(Profile::clone(&new_profile));
    }
    else
    {
        println!("Profile was not validated. Return to login screen.");
        login_init();
    }
}

fn select_profile() -> Result<Profile, &'static str>
{
    /*
    |
    |  This function selects a profile from profiles.json, validates it against the database, and returns the profile if successful.
    |  This function additionally saves the token to token.json.
    |
    /===================================*/

    // loop until a valid profile is selected.
    loop
    {
        utils::clear(None);
        println!("Please select one of your profiles, or type B to go back. : \n \n");
        let mut profile_data: ProfileContainer = ProfileContainer { profiles: Vec::new() };
        deserialize_profile_data(&mut profile_data);
        let mut counter: i32 = 0;
        let mut profile_hashmap: HashMap<i32, Profile> = HashMap::new();
        for profile in profile_data.profiles
        {
            counter += 1;
            profile_hashmap.insert(counter, Profile::clone(&profile));
            println!("{} | ({})", utils::pad_string(String::from(&profile.username), 16), counter);
        }
        let mut selection: String = String::new();
        io::stdin().read_line(&mut selection).expect("Failed to read line.");
        selection = String::from(selection.trim());
        let _ = io::stdout().flush();
        let potential_selected_profile: Profile = {
            let hash_obj: Option<&Profile> = {
                // Try to handle all cases of invalid inputs.
                if selection.as_str() == "B" || selection.as_str() == "b"
                {
                    login_init()
                }
                if selection.as_str().parse::<i32>().is_err()
                {
                    continue;
                }
                profile_hashmap.get(&selection.as_str().parse::<i32>().unwrap())
            };
            match hash_obj
            {
                None => Profile::default(),
                _ => Profile { username: String::from(&hash_obj.unwrap().username), password: String::from(&hash_obj.unwrap().password) }
            }
        };
        if validate_login_info(&Profile::clone(&potential_selected_profile)) == true
        {
            return Ok(potential_selected_profile);
        }

        return Err("Selected profile was invalid. Please try again.");
    }
}

fn login(p: Profile) -> bool
{
    validate_login_info(&p);
    //
    false
}

pub fn login_select_profile()
{
    let selected_profile: Profile = {
        match select_profile()
        {
            Ok(p) =>
            {
                utils::clear(None);
                println!("Profile validated. Logging you in...");
                p
            }
            Err(e) =>
            {
                utils::clear(None);
                println!("{}", e.red());
                return login_select_profile();
            }
        };
        Profile::default() // this should NEVER run. only here because rust will babyrage if I don't
    };
    /*
       Call to login. Now the shitshow begins.
    */
    let res: bool = login(Profile::clone(&selected_profile)); //
    if res == true
    {
        println!("Successfully logged you in as {}. Opening messenger...", &selected_profile.username.red())
    }
}

pub fn login_init()
{
    utils::clear(None);
    println!("Welcome to CRIM. \n");
    println!("Register New Profile    (1)");
    println!("Select Existing Profile (2)");
    println!("Exit                    (3)");

    let mut selection: String = String::new();
    io::stdin().read_line(&mut selection).expect("Failed to read the line.");
    selection = String::from(selection.trim());
    match selection.as_str()
    {
        "1" => register_profile(None),
        "2" => login_select_profile(),
        "3" => std::process::exit(0),
        _ => login_init()
    }
}

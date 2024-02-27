extern crate dotenv;
use super::messenger;
use super::mongo;
use super::utils;
use colored::Colorize;
use mongodb::bson;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};

/*

This file handles the login system of CRIM.
Profiles.json is a local cache of accounts, to allow for quick sign in.
Any account that is being logged in with will be checked against the account database in the server so as to prevent fake accounts; registering is necessary.

*/

/*

Structs

*/

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Profile
{
    pub username: String,
    pub password: String
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct ProfileContainer
{
    profiles: Vec<Profile>
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Token
{
    pub token: String
}

/*

    Login-Specific Utility Functions

*/

fn deserialize_profile_data() -> ProfileContainer
{
    /*
       Deserialize the data in the profiles json file, and return it.
    */

    let f: File = fs::File::open("src/userdata/profiles.json").unwrap();
    serde_json::from_reader(BufReader::new(f)).unwrap_or_default()
}

fn serialize_profile_data(container: &ProfileContainer)
{
    /*
        Serializes profile data and writes it to the profiles json file.
    */

    let f: File = File::create("src/userdata/profiles.json").expect("Failed to write profile data to file.");

    if serde_json::to_writer(BufWriter::new(f), &container).is_err()
    {
        panic!("Failed to serialize profile data.");
    }
}

fn validate_login_info(profile_to_be_validated: &Profile) -> Option<bson::Document>
{
    /*

       Validates login information against the database.
       Maybe also instead of storing profile username and passwords, you could only store the username since they must be unique; passwords laying around is a risk.

    */

    let coll: mongodb::sync::Collection<bson::Document> = mongo::get_collection("accounts");
    coll.find_one(bson::doc! { "username": &profile_to_be_validated.username, "password": &profile_to_be_validated.password }, None)
        .unwrap()
}

/*
|
|				Interactable Functions
|
====================================================*/

fn register_profile(addl_message: Option<&str>)
{
    /*
    |  Function to register a new profile.
    |  This function will prompt the user for a username and password, and then save it to profiles.json and the database, if the username is unique.
    |  If the username is not unique, the function will return to the start of the function.
    /===================================*/

    utils::clear(addl_message);

    let coll: mongodb::sync::Collection<bson::Document> = mongo::get_collection("accounts");

    // grab username input
    let username: String = utils::grab_str_input(Some("Please input a username for your new profile. :"));

    // check username uniquity
    let unique_query: Result<Option<bson::Document>, mongodb::error::Error> = coll.find_one(bson::doc! {"username": &username}, None);
    match unique_query
    {
        Ok(Some(_)) =>
        {
            register_profile(Some("Username already exists. Please try again."));
        }
        Err(_) => panic!("Failed to query database"),
        _ =>
        {}
    };

    let password: String = utils::grab_str_input(Some("Please input a password for your new profile. :"));

    let new_profile: Profile = Profile { username, password };
    utils::clear(None);

    // save the data to profiles.json here.

    let mut deserialized_data: ProfileContainer = deserialize_profile_data();
    deserialized_data.profiles.push(new_profile.clone());
    serialize_profile_data(&deserialized_data);

    /*
       mang- i mean mongo time!
    */

    let doc: Result<bson::Document, bson::ser::Error> = bson::to_document(&serde_json::to_value(&new_profile).unwrap());
    let doc: Result<mongodb::results::InsertOneResult, mongodb::error::Error> = coll.insert_one(doc.unwrap(), None);
    let token: bson::Bson = doc.unwrap().inserted_id;
    // write the token to token.json using serde_json
    let token_obj: Token = Token { token: token.as_object_id().unwrap().to_string() };

    serde_json::to_writer(
        BufWriter::new(File::create("src/userdata/token.json").expect("Failed to write token to file. Please ensure you have a token.json file existing.")),
        &token_obj
    )
    .expect("Failed to write token to file. Please ensure you have a token.json file existing.");

    let validation_status: Option<bson::Document> = validate_login_info(&new_profile);
    match validation_status
    {
        Some(_) =>
        {
            println!("Profile validated. Logging you in...");
            let _ = messenger::create_user(&new_profile);
            // we don't need the actual messageuser here. Really there's no reason to return it at all. But maybe i'll need it someday.
            login(&new_profile);
        }
        None =>
        {
            println!("Profile was not validated. Return to login screen.");
            login_init();
        }
    };
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
    let mut msg: Option<String> = None;
    loop
    {
        utils::clear(msg.as_deref());
        println!("Please select one of your profiles, or type B to go back. : \n \n");
        let profile_data: ProfileContainer = deserialize_profile_data();

        let profile_hashmap: HashMap<usize, Profile> = profile_data
            .profiles
            .iter()
            .enumerate()
            .map(|(i, p)| {
                println!("{} | ({})", utils::pad_string(&p.username, 16), i + 1);

                (i + 1, p.clone())
            })
            .collect::<HashMap<_, _>>();

        let selection: i32 = utils::grab_int_input(Some("Please input the number of the profile you'd like to select. :"), profile_hashmap.len() as i32);
        let potential_selected_profile: Profile = {
            let hash_obj: Option<&Profile> = {
                if let Ok(i) = selection.try_into()
                {
                    profile_hashmap.get(&i)
                }
                else
                {
                    continue;
                }
            };

            hash_obj.cloned().unwrap_or_default()
        };

        msg = match validate_login_info(&potential_selected_profile)
        {
            Some(_) => return Ok(potential_selected_profile),
            None => Some(String::from("Profile was not validated. Please try again."))
        };
    }
}

fn login(p: &Profile)
{
    validate_login_info(p);
    messenger::init(p);
}

pub fn login_select_profile()
{
    let selected_profile: Option<Profile> = {
        match select_profile()
        {
            Ok(p) =>
            {
                utils::clear(None);
                println!("Profile validated. Logging you in...");
                Some(p)
            }
            Err(e) =>
            {
                utils::clear(None);
                println!("{}", e.red());
                None
            }
        }
    };

    let token: Option<bson::Document> = validate_login_info(selected_profile.as_ref().unwrap());
    match token
    {
        Some(token) =>
        {
            let token_obj: Token = Token { token: token.get("_id").unwrap().as_object_id().unwrap().to_string() };
            let token_json_str: String = serde_json::to_string(&token_obj).unwrap();
            fs::write("src/userdata/token.json", token_json_str).expect("Failed to write token to file. Please ensure you have a token.json file existing.");
        }
        None =>
        {
            // how the fuck is this ever going to run?
            panic!("{:#?}", token)
        }
    };
    login(selected_profile.as_ref().unwrap()); //
}

pub fn login_init()
{
    utils::clear(None);
    let ui = vec!
    [
        "Welcome to CRIM.",
        "",
        "",
        "",
        "register : register an account",
        "profile : select a profile",
        "exit : leave CRIM"
    ];
    utils::create_ui(ui, utils::Position::Center);
    let selection: String = utils::grab_opt(None, vec!["register", "profile", "exit"]);
    match selection.as_str()
    {
        "register" => register_profile(None),
        "profile" => login_select_profile(),
        "exit" => std::process::exit(0),
        _ => login_init()
    }
}

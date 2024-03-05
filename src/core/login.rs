extern crate dotenv;
use super::messenger;
use super::mongo;
use super::utils;
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;
use mongodb::bson;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use openssl::{pkey::PKey, rsa::Rsa};

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
    pub hash: String,
    pub salt: Vec<u8>,
    pub public_key: String,
    pub priv_key_hash: String,
    pub priv_key_salt: Vec<u8>
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


fn validate_login_info(profile_to_be_validated: &Profile) -> Option<bson::Document>
{
    /*

       Validates login information against the database.
       I don't think I need to check the salt. That should be left out of code as much as possible for security sake.

    */

    let coll: mongodb::sync::Collection<bson::Document> = mongo::get_collection("accounts");
    coll.find_one(bson::doc! { "username": &profile_to_be_validated.username, "hash": &profile_to_be_validated.hash }, None)
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

    if let Some(msg) = addl_message
    {
        // hard-code red because if this function succeeds everything is cleared anyway. only errors need to be shown
        utils::addl_message(msg, "red");
    }
    utils::clear();

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

    // crypto login

    let mut salt: [u8; 256] = [0; 256];
    getrandom(&mut salt).expect("Failed to generate random salt.");
    let mut output: [u8; 256] = [0u8; 256];
    Argon2::default().hash_password_into(&password.into_bytes(), &salt, &mut output).expect("failed to hash password");
    let base64_encoded = general_purpose::STANDARD.encode(&output);

    // gen public and private keys

    let rsa = Rsa::generate(2048).unwrap();
    let private_key = PKey::from_rsa(rsa).unwrap();
    let mut private_key_salt = [0u8; 256];
    getrandom(&mut private_key_salt).expect("Failed to generate key salt.");
    let mut output: [u8; 256] = [0u8; 256];
    //Argon2::default().hash_password_into(&private_key.p.unwrap(), &private_key_salt, &mut output).expect("failed to hash private key");
    let public_key = private_key.public_key_to_pem().unwrap();
    let private_key = private_key.private_key_to_pkcs8().unwrap();
    panic!("{:#?} {:#?}", public_key, private_key);
    
    let new_profile = Profile::default();

    /*
        mang- i mean mongo time!
    */


    utils::clear();

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


fn login_upass()
{
    let mut msg: &str = "";
    //TODO: let user type something to exit this and return to login screen
    loop
    {
        utils::clear();
        if msg != ""
        {
            utils::addl_message(msg, "red");
        }
        let username = utils::grab_str_input(Some("Type your username."));
        let password = utils::grab_str_input(Some("Type your password."));
        let coll: mongodb::sync::Collection<bson::Document> = mongo::get_collection("accounts");
        let mut trip: bool = false;
        let query = coll.find_one(bson::doc! { "username": &username }, None).unwrap();
        match query
        {
            Some(doc) =>
            {
                let hash = doc.get("hash").unwrap().as_str().unwrap();
                let salt: &Vec<bson::Bson> = doc.get_array("salt").unwrap();
                let salt: Vec<u8> = salt.iter().map(|x| x.as_i64().unwrap() as u8).collect::<Vec<u8>>();
                let mut output: [u8; 256] = [0u8; 256];
                Argon2::default().hash_password_into(&password.into_bytes(), &salt, &mut output).expect("failed to hash password");
                let base64_encoded = general_purpose::STANDARD.encode(&output);
                if base64_encoded == hash
                {
                    let token: bson::Bson = doc.get("_id").unwrap().clone();
                    let token_obj: Token = Token { token: token.as_object_id().unwrap().to_string() };
                    serde_json::to_writer(
                        BufWriter::new(File::create("src/userdata/token.json").expect("Failed to write token to file. Please ensure you have a token.json file existing.")),
                        &token_obj
                    )
                    .expect("Failed to write token to file. Please ensure you have a token.json file existing.");
                    let profile = Profile::default();
                    //TODO: fix this to get the keys from db
                    //let profile = Profile { username: username, hash: hash.to_string(), salt: salt };
                    login(&profile);
                }
                else
                {
                    trip = true;
                }
            }
            None =>
            {
                /* 
                The problem with this is that if an invalid username is entered, the trip variable has already been tripped.
                This means that if someone was guessing usernames and passwords, they would be able to tell if a username is valid based solely on response time from the program.
                I don't know if this is a real issue, but it is something to note.
                */
                trip = true;
            }
        }
        if trip
        {
            msg = "Invalid username or password.";
            continue;
        }
    }
}

fn login(p: &Profile)
{
    validate_login_info(p);
    messenger::init(p);
}


pub fn login_init()
{
    utils::clear();
    let ui = vec![
        "Welcome to CRIM.",
        "",
        "",
        "",
        "register : register an account",
        "login : login to an existing account",
        "exit : leave CRIM",
    ];
    utils::create_ui(ui, utils::Position::Center);
    let selection: (String, String) = utils::grab_opt(None, vec!["register", "login", "exit"]);
    match selection.0.as_str()
    {
        "register" => register_profile(None),
        "login" => login_upass(),
        "exit" => std::process::exit(0),
        _ => login_init()
    }
}

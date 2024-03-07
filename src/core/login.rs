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
use std::io::Write;
use openssl::{pkey::PKey, rsa::Rsa, symm::Cipher};
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
    pub public_key: Vec<u8>,
    pub priv_key_enc: Vec<u8>,
}

impl Profile
{
    fn from_document(doc: bson::Document) -> Profile
    {
        Profile 
        {
            username: doc.get_str("username").unwrap().to_string(),
            hash: doc.get_str("hash").unwrap().to_string(),
            salt: doc.get_array("salt").unwrap().iter().map(|x| x.as_i64().unwrap() as u8).collect::<Vec<u8>>(),
            public_key: doc.get_array("public_key").unwrap().iter().map(|x| x.as_i64().unwrap() as u8).collect::<Vec<u8>>(),
            priv_key_enc: doc.get_array("priv_key_enc").unwrap().iter().map(|x| x.as_i64().unwrap() as u8).collect::<Vec<u8>>(),
        }
    }
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
    // may be a better way to do this than use .clone()
    Argon2::default().hash_password_into(&password.clone().into_bytes(), &salt, &mut output).expect("failed to hash password");
    let b64_pass: String = general_purpose::STANDARD.encode(output);

    // gen public and private keys

    let rsa: Rsa<openssl::pkey::Private> = Rsa::generate(2048).unwrap();
    let pkey: PKey<openssl::pkey::Private> = PKey::from_rsa(rsa).unwrap();
    // private key will be encrypted with the user's password

    let cipher: Cipher = Cipher::aes_256_cbc();
    let public_key: Vec<u8> = pkey.public_key_to_pem().unwrap();

    /*
    This is a dilemma between security and convenience. Realistically, it would be better practice to have the private key encrypted at all times when it's stored,
    no matter if its stored locally temporarily or on the server (obviously encrypted on the server already.) However, it is left in plaintext on the client's computer;
    a security risk only solved if we had to get the client to enter a password *each time* they opened up their message logs-- and even if they did that, it would still
    be stored in plaintext for an unknown amount of time. Further enhanced if the client exits the terminal without letting the program clear their keyfile, I believe this is
    just something I have to accept.
     */

    let private_key: Vec<u8> = pkey.private_key_to_pem_pkcs8().unwrap();
    let mut file = File::create("src/userdata/pkey.key").unwrap(); // could be an env variable as to what pkey.key could be named
    file.write_all(&private_key).expect("failed to write priv key to pkey.key");
    
    // actually encrypt priv key and save it
    let private_key: Vec<u8> = pkey.private_key_to_pem_pkcs8_passphrase(cipher, &password.as_bytes()).unwrap(); 
    //https://docs.rs/openssl/latest/openssl/symm/index.html
    let new_profile: Profile = Profile { username: username, hash: b64_pass, salt: salt.to_vec(), public_key: public_key, priv_key_enc: private_key };
    // ^^ this is fat as HELL in the database. 33kb for a single user entry!!! Could compress somehow for strict data limits, but not important atm
   
   
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
                let profile = Profile::from_document(doc.clone());
                let mut output: [u8; 256] = [0u8; 256];
                Argon2::default().hash_password_into(&password.clone().into_bytes(), &profile.salt, &mut output).expect("failed to hash password");
                let base64_encoded = general_purpose::STANDARD.encode(&output);
                /*
                The glorious flaw. Currently, authentication is purely and wholly based on an if statement.
                The one silver lining here is that it would be impossible to decrypt the private key without the password, so even if someone did
                manipulate the binaries to let them log in to any account they want, they wouldn't be able to read anything. If they tried to send something,
                the recipient would notice it's unreadable and realize something's up.
                 */
                if base64_encoded == profile.hash
                {
                    let token: bson::Bson = doc.get("_id").unwrap().clone();
                    let token_obj: Token = Token { token: token.as_object_id().unwrap().to_string() };
                    serde_json::to_writer(
                        BufWriter::new(File::create("src/userdata/token.json").expect("Failed to write token to file. Please ensure you have a token.json file existing.")),
                        &token_obj
                    )
                    .expect("Failed to write token to file. Please ensure you have a token.json file existing.");

                    let private_key: Vec<u8> = Rsa::private_key_from_pem_passphrase(&profile.priv_key_enc, &password.into_bytes()).unwrap().private_key_to_pem().unwrap();
                    let mut file = File::create("src/userdata/pkey.key").unwrap();
                    file.write_all(&private_key).expect("failed to write priv key to pkey.key");
                    login(&profile);
                    break;
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
    let ui: Vec<String> = vec![
        "Welcome to CRIM.".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "register : register an account".to_string(),
        "login : login to an existing account".to_string(),
        "exit : leave CRIM".to_string(),
    ];
    utils::create_ui(&ui, utils::Position::Center);
    let selection: (String, String) = utils::grab_opt(None, vec!["register", "login", "exit"]);
    match selection.0.as_str()
    {
        "register" => register_profile(None),
        "login" => login_upass(),
        "exit" => std::process::exit(0),
        _ => login_init()
    }
}

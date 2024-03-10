extern crate dotenv;
use crate::messenger::messenger_panel;
use super::utils;
use super::structs::Account;
use argon2::Argon2;
use base64::{engine::general_purpose, Engine as _};
use getrandom::getrandom;
use openssl::{pkey::PKey, rsa::Rsa, symm::Cipher};
use std::fs::File;
use std::io::Write;

/*

This file handles the login system of CRIM.
Accounts.json is a local cache of accounts, to allow for quick sign in.
Any account that is being logged in with will be checked against the account database in the server so as to prevent fake accounts; registering is necessary.
*/

fn validate_login_info(account_to_be_validated: &Account) -> Option<Account>
{
    Account::get_account(&account_to_be_validated.username)
}

/*
|
|				Interactable Functions
|
====================================================*/

fn register_account(addl_message: Option<&str>)
{
    /*
    Registers a new account in the database. Username uniquity is enforced, and usernames cannot currently be changed, because the friend system relies on usernames.
    Could be refactored to use UUIDs instead of usernames to allow for username changing, but i still think uniquity makes things clearer for everyone.
    */

    if let Some(msg) = addl_message
    {
        // hard-code red because if this function succeeds everything is cleared anyway. only errors need to be shown
        utils::addl_message(msg, "red");
    }

    utils::clear();

    let username: String = utils::grab_str_input(Some("Please input a username for your new account:"));
    let unique_query: Option<Account> = Account::get_account(&username);
    if unique_query.is_some()
    {
        register_account(Some("Username already exists. Please try again."));
    }

    let password: Vec<u8> = utils::grab_str_input(Some("Please input a password for your new account:")).into_bytes();
    // turn this into bytes immediately so I don't have to clone it in the hash function

    // crypto login

    let mut salt: [u8; 256] = [0; 256];
    getrandom(&mut salt).expect("Failed to generate random salt.");
    let mut output: [u8; 256] = [0u8; 256];
    Argon2::default()
        .hash_password_into(&password, &salt, &mut output)
        .expect("failed to hash password");
    let b64_pass: String = general_purpose::STANDARD.encode(output);

    // gen public and private keys
    let pkey: PKey<openssl::pkey::Private> = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let cipher: Cipher = Cipher::aes_256_cbc();
    let public_key: Vec<u8> = pkey.public_key_to_pem().unwrap();
    let private_key: Vec<u8> = pkey.private_key_to_pem_pkcs8().unwrap();
    let mut file = File::create("src/userdata/pkey.key").unwrap(); // could be an env variable as to what pkey.key could be named
    file.write_all(&private_key)
        .expect("Error writing private key to pkey.key");

    // actually encrypt priv key and save it
    let private_key: Vec<u8> = pkey
        .private_key_to_pem_pkcs8_passphrase(cipher, &password)
        .unwrap();
    //https://docs.rs/openssl/latest/openssl/symm/index.html
    let new_account: Account = Account { username, hash: b64_pass, salt: salt.to_vec(), public_key, priv_key_enc: private_key, friends: Vec::new() };
    // ^^ this is fat as HELL in the database. 33kb for a single user entry!!! Could compress somehow for strict data limits, but not important atm

    /*
        mang- i mean mongo time!
    */

    utils::clear();

    match Account::create_account(&new_account)
    {
        Ok(_) =>
        {
            println!("Account validated. Logging you in...");
            login(&new_account);
        }
        Err(e) =>
        {
            panic!("An error occurred during account creation: {}", e)
        }
    };
}

fn login_upass()
{
    let mut msg: &str = "Type \"back\" to leave.";
    loop
    {
        utils::clear();
        utils::addl_message(msg, "red"); 
        let username = utils::grab_str_input(Some("Type your username."));
        let password = utils::grab_str_input(Some("Type your password."));
        if username == "back" || password == "back" {login_init()};
        let mut trip: bool = false;
        let query = Account::get_account(&username);
        match query
        {
            Some(account) =>
            {
                let mut output: [u8; 256] = [0u8; 256];
                Argon2::default()
                    .hash_password_into(&password.clone().into_bytes(), &account.salt, &mut output)
                    .expect("failed to hash password");
                let base64_encoded = general_purpose::STANDARD.encode(output);
                // unsecure. read readme.md
                if base64_encoded == account.hash
                {
                    let private_key: Vec<u8> = Rsa::private_key_from_pem_passphrase(&account.priv_key_enc, &password.into_bytes())
                        .unwrap()
                        .private_key_to_pem()
                        .unwrap();
                    let mut file = File::create("src/userdata/pkey.key").unwrap();
                    file.write_all(&private_key)
                        .expect("failed to write priv key to pkey.key");
                    login(&account);
                    break;
                }
                trip = true;
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
        };
        if trip
        {
            msg = "Invalid username or password.";
            continue;
        }
    }
}

fn login(p: &Account)
{
    validate_login_info(p);
    messenger_panel::init(p);
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
        "register" => register_account(None),
        "login" => login_upass(),
        "exit" => std::process::exit(0),
        _ => login_init()
    }
}

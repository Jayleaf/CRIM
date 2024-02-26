use super::login;
use super::mongo;
use login::{Profile, Token};
use mongodb::bson::Document;
use mongodb::bson::{doc, to_document};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io::BufReader;
use super::utils;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MessageUser {
    token: String,
    username: String,
    friends: Vec<String>, // this is going to be a vector of usernames
}

impl MessageUser {
    fn from_document(doc: Document) -> Self {
        MessageUser {
            token: doc.get_str("token").unwrap().to_string(),
            username: doc.get_str("username").unwrap().to_string(),
            friends: doc.get_array("friends").unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect(),
        }
    }
}

/*
|
|               Front-End Functions
|
=====================================================*/

pub fn draw_home(user: &MessageUser)
{
    let welcome_message: String = format!("Welcome, {}.", &user.username);
    let ui: Vec<&str> = vec![
        welcome_message.as_str(),
        "",
        "",
        "",
        "Please select an option.",
        "",
        "1. Open Messages",
        "2. Manage Friends",
        "3. Log Out"
    ];
    utils::create_ui(ui, "Home", utils::Position::Center);
}




pub fn create_user(profile: &login::Profile) -> MessageUser {
    // This function will find a user matching the profile in the accounts database, and create a messageuser database entry from it.
    let account_collection: mongodb::sync::Collection<Document> = mongo::get_collection("accounts");
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");

    let user: MessageUser = {
        match account_collection.find_one(doc! { "username": &profile.username, "password": &profile.password }, None) {
            Ok(Some(unwrapped_collection)) => {
                MessageUser { token: unwrapped_collection.get_object_id("_id").unwrap().to_string(), username: unwrapped_collection.get_str("username").unwrap().to_string(), friends: Vec::new() }
            }
            Err(_) => {
                panic!("Tried to create a user with an invalid profile.")
            }
            Ok(None) => {
                panic!("Tried to create a user with an invalid profile.")
            }
        }
    };
    let doc = to_document(&serde_json::to_value(&user).unwrap());
    user_collection.insert_one(doc.unwrap(), None).expect("Failed to create a new messageuser in the db.");
    user
}

fn update_user_data(user: &MessageUser) -> Option<MessageUser> {
    // This function will update the user's data in the database.
    None
}

fn retrieve_user_data(token: &str) -> MessageUser {
    // This function will retrieve the user's data from the database and return it as a MessageUser. Ideally, don't do this often, because you don't want to spam the db.
    // The reason this doesn't return an Option is because there there is nothing to retrieve if the token is invalid, and it would break everything going forward.
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");
    // messageusers and accounts are different, because the account coll holds passwords and shit that we don't need.
    let user: MessageUser = {
        match user_collection.find_one(doc! { "token": token  }, None) {
            Ok(data) => match data {
                Some(d) => MessageUser::from_document(d),
                None => {
                    panic!("Tried to retrieve user data with an invalid token.")
                }
            },
            Err(_) => {
                panic!("Tried to retrieve user data with an invalid token.")
            }
        }
    };

    user
}

pub fn init(profile: &Profile) {
    let f: fs::File = fs::File::open("src/userdata/token.json").unwrap();
    let token: Token = serde_json::from_reader(BufReader::new(f)).unwrap_or_default();
    let user: MessageUser = retrieve_user_data(&token.token);
    draw_home(&user);
}

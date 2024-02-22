use super::login;
use super::mongo;
use mongodb::bson::Document;
use mongodb::bson::{doc, to_document};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use login:: {Profile, Token};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MessageUser
{
    token: String,
    username: String,
    friends: Vec<String> // this is going to be a vector of usernames
}


impl MessageUser
{
    fn from_document(doc: Document) -> Self
    {
        MessageUser {
            token: doc.get_str("token").unwrap().to_string(),
            username: doc.get_str("username").unwrap().to_string(),
            friends: doc.get_array("friends").unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
        }
    }
}

pub fn create_user(profile: login::Profile) -> MessageUser
{
    // This function will find a user matching the profile in the accounts database, and create a messageuser database entry from it.
    let account_collection = mongo::get_collection("accounts");
    let user_collection = mongo::get_collection("messageusers");

    let user: MessageUser = {
        match account_collection.find_one(doc! { "username": &profile.username, "password": &profile.password }, None)
        {
            Ok(data) => 
            {
                let unwrapped_collection = data.unwrap();
                MessageUser {
                    token: unwrapped_collection.get_object_id("_id").unwrap().to_string(),
                    username: unwrapped_collection.get_str("username").unwrap().to_string(),
                    friends: Vec::new()
                }
            }
            Err(_) =>
            {
                panic!("Tried to create a user with an invalid profile.")
            }
        }
    };
    let doc = to_document(&serde_json::to_value(&user).unwrap());
    user_collection.insert_one(doc.unwrap(), None).expect("Failed to create a new messageuser in the db.");
    user
    
}

fn update_user_data(user: MessageUser) -> Option<MessageUser>
{
    // This function will update the user's data in the database.
    None
}

fn retrieve_user_data(token: String) -> MessageUser
{
    // This function will retrieve the user's data from the database and return it as a MessageUser. Ideally, don't do this often, because you don't want to spam the db.
    // The reason this doesn't return an Option is because there there is nothing to retrieve if the token is invalid, and it would break everything going forward.
    let user_collection = mongo::get_collection("messageusers");
    // messageusers and accounts are different, because the account coll holds passwords and shit that we don't need.
    let user: MessageUser = {
        match user_collection.find_one(doc! { "token": token.as_str()  }, None)
        {
            Ok(data) => 
            {
                match data
                {
                    Some(d) =>
                    {
                        MessageUser::from_document(d)
                    },
                    None =>
                    {
                        panic!("Tried to retrieve user data with an invalid token.")
                    }
                }
            },
            Err(_) =>
            {
                panic!("Tried to retrieve user data with an invalid token.")
            }
        }
    };

    user
}

pub fn init(profile: Profile)
{
    let mut f: File = fs::File::open("src/userdata/token.json").unwrap();
    let mut data: String = String::new();
    f.read_to_string(&mut data).unwrap();
    let token: Token = {
        let t: Result<Token, serde_json::Error> = serde_json::from_str(data.as_str());
        match t
        {
            Ok(t) => t,
            Err(e) => panic!("{:#?}", e) // todo: make this go back to the login instead of panicking
        }
    };
    let user: MessageUser = retrieve_user_data(String::clone(&token.token));
    println!("Welcome, {}!", user.username);
}

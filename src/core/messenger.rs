use super::login;
use super::mongo;
use super::utils;
use login::{Profile, Token};
use mongodb::bson::Document;
use mongodb::bson::{doc, to_document};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::ops::Deref;
use std::vec;

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
        "msg : opens the message panel",
        "manage : manage your friends",
        "logout : log out of your account.",
    ];
    utils::create_ui(ui,  utils::Position::Center);
    let opt: String = utils::grab_opt(None, vec! ["msg", "manage", "logout"]);
    match opt.as_str()
    {
        "msg" =>
        {
            println!("msg")
            // nothing here
        }
        "manage" =>
        {
            draw_friend_mgmt(user);
        }
        "logout" =>
        {
            println!("logout")
            //logout here
        }
        _ =>
        {
            // this should never run, because get_opt checks for all cases.
            panic!("get_opt failed.")
        }
    }
}

pub fn draw_friend_mgmt(user: &MessageUser)
{
    
    let friends: &Vec<String> = &user.friends;
    let mut ui = vec![
        "Friends Management",
        "",
        "",
    ];
    for friend in friends
    {
        ui.push(friend.as_str());
    }
    ui.push("");
    ui.push("add <friend> : adds friend by username");
    ui.push("rm <friend> : removes friend by username");
    ui.push("back : returns to home page");
    utils::create_ui(ui, utils::Position::Center);
    let opt: String = utils::grab_str_input(Some("Please input your option."));
    /* 
    look, i tried hard to not spam else if. but i couldn't make .starts_with work dynamically with a match block.
    maybe i could use the first x characters if all commands were the same length, but they're not, so we spam else if.
     */
    if opt.starts_with("add")
    {
        let friend: &str = opt.trim_start_matches("add").trim();
        if add_friend(user, &friend) { utils::clear(); draw_friend_mgmt(user); utils::addl_message("Successfully added friend.", "green") }
        else { utils::clear(); draw_friend_mgmt(user); utils::addl_message("Friend already added.", "red") }
    }
    else if opt.starts_with("rm")
    {
        let friend: String = utils::grab_str_input(Some("Please input the username of the friend you would like to remove."));
        remove_friend(user, &friend);
    }
    else if opt.starts_with("back")
    {
            draw_home(user);
    }
        
    }


pub fn create_user(profile: &login::Profile) -> MessageUser
{
    // This function will find a user matching the profile in the accounts database, and create a messageuser database entry from it.
    let account_collection: mongodb::sync::Collection<Document> = mongo::get_collection("accounts");
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");

    let user: MessageUser = {
        match account_collection.find_one(doc! { "username": &profile.username, "password": &profile.password }, None)
        {
            Ok(Some(unwrapped_collection)) =>
            {
                MessageUser { token: unwrapped_collection.get_object_id("_id").unwrap().to_string(), username: unwrapped_collection.get_str("username").unwrap().to_string(), friends: Vec::new() }
            }
            Err(_) =>
            {
                panic!("Tried to create a user with an invalid profile.")
            }
            Ok(None) =>
            {
                panic!("Tried to create a user with an invalid profile.")
            }
        }
    };
    let doc = to_document(&serde_json::to_value(&user).unwrap());
    user_collection.insert_one(doc.unwrap(), None).expect("Failed to create a new messageuser in the db.");
    user
}

/*
|
|       Back-End Functions
|
=========================================*/

fn update_user_data(user: &MessageUser) -> Result<MessageUser, ()>
{
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");
    let filter = doc! { "username": &user.username };
    let update = doc! { "$set": { "username": &user.username, "friends": &user.friends } };
    match user_collection.update_one(filter, update, None)
    {
        Ok(_) => 
        {
            // validate that the data actually was updated on the backend
            let dbdata = retrieve_user_data(&user.username);
            if dbdata.username == user.username && dbdata.friends == user.friends
            {
                Ok(dbdata)
            }
            else
            {
                Err(())
            }
        }
        Err(_) => Err(())
    }
}

fn retrieve_user_data(username: &str) -> MessageUser
{
    // This function will retrieve the user's data from the database and return it as a MessageUser. Ideally, don't do this often, because you don't want to spam the db.
    // The reason this doesn't return an Option or Result is because there there is nothing to retrieve if the token is invalid, and it would break everything going forward.
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");
    // messageusers and accounts are different, because the account coll holds passwords and shit that we don't need.
    let user: MessageUser = {
        match user_collection.find_one(doc! { "username": &username  }, None)
        {
            Ok(data) => match data
            {
                Some(d) => MessageUser::from_document(d),
                None =>
                {
                    panic!("Tried to retrieve user data with an invalid token.")
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

fn add_friend(user: &MessageUser, friend: &str) -> bool
{
    let friend: String = String::from(friend);
    let mut udata: MessageUser = retrieve_user_data(&user.username);
    if udata.friends.contains(&friend) { return false }
    udata.friends.push(friend);
    match update_user_data(&udata)
    {
        Ok(_) =>
        {
            return true
        }
        Err(_) =>
        {
            return false
        }
    }
    // todo: blocklist? not necessary right now though.
    // todo: check to make sure friend actually exists, cause a user could just spam with people who don't exist
}

fn remove_friend(user: &MessageUser, friend: &str)
{
   // do stuff :)
}

pub fn init(profile: &Profile)
{
    let user: MessageUser = retrieve_user_data(&profile.username);
    draw_home(&user);
}

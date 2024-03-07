use super::login;
use super::mongo;
use super::utils;
use colored::Colorize;
use login::Profile;
use mongodb::bson::Document;
use mongodb::bson::{doc, to_document};
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;
use std::vec;
use openssl::pkey::PKey;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MessageUser
{
    token: String,
    username: String,
    friends: Vec<String> // this is going to be a vector of usernames
}

struct Message
{
    message: String,
    time: String
}

struct Conversation
{
    users: Vec<String>,
    messages: Vec<Message>
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
    utils::clear();
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
    utils::create_ui(ui, utils::Position::Center);
    let opt: (String, String) = utils::grab_opt(None, vec!["msg", "manage", "logout"]);
    match opt.0.as_str()
    {
        "msg" =>
        {
            draw_msg(user);
        }
        "manage" =>
        {
            utils::clear();
            draw_friend_mgmt(user);
        }
        "logout" =>
        {
            let token = login::Token::default();
            let f: File = File::create("src/userdata/token.json").expect("Failed to write profile data to file.");
            serde_json::to_writer(BufWriter::new(f), &token).expect("Token serialization failed. Ensure token.json exists.");
            login::login_init();
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
    let user: MessageUser = retrieve_user_data(&user.username).unwrap(); // the user arg can be trusted to have a proper username but not proper friends.
    let friends: &Vec<String> = &user.friends;
    let mut ui = vec!["Friends Management", "", ""];
    for friend in friends
    {
        ui.push(friend.as_str());
    }
    ui.push("");
    ui.push("add <friend> : adds friend by username");
    ui.push("rm <friend> : removes friend by username");
    ui.push("back : returns to home page");
    utils::create_ui(ui, utils::Position::Center);
    let opt: (String, String) = utils::grab_opt(Some("Please input your option."), vec!["add", "rm", "back"]);
    match opt.0.as_str()
    {
        "add" =>
        {
            let friend: &str = opt.1.as_str();
            println!("{}", friend);
            if add_friend(&user, &friend)
            {
                utils::clear();
                utils::addl_message("Successfully added friend.", "green");
                draw_friend_mgmt(&user);
            }
            else
            {
                utils::clear();
                utils::addl_message(format!("User {} does not exist, or you already have them added.", friend.blue()).as_str(), "red");
                draw_friend_mgmt(&user);
            }
        }
        "rm" =>
        {
            utils::clear();
            let friend: &str = opt.1.as_str();
            if remove_friend(&user, &friend)
            {
                utils::addl_message("Successfully removed friend.", "green");
                draw_friend_mgmt(&user);
            }
            else
            {
                utils::addl_message(format!("User {} does not exist, or you don't have them added.", friend.blue()).as_str(), "red");
                draw_friend_mgmt(&user);
            }
        }
        "back" =>
        {
            draw_home(&user);
        }
        _ =>
        {
            utils::clear();
            draw_friend_mgmt(&user);
        }
    }
}

fn draw_msg(user: &MessageUser)
{
    //TODO: exit option
    let user: MessageUser = retrieve_user_data(&user.username).unwrap();
    let friends: &Vec<String> = &user.friends;
    let ui =vec!
    [
        "Message Panel",
        "",
        "",
        "",
        "new <friend> : start a new single conversation with a friend.",
        "new --multi <friend, friend> : start a new multi-person conversation.",
        "open : view open conversations you are a participant in."
    ];
    utils::create_ui(ui, utils::Position::Center);
    // flags aren't dynamic. I could fix that at some point but it's unnecessary right now.
    let opt: (String, String) = utils::grab_opt(None, vec!["new", "new --multi", "open"]);
    match opt.0.as_str()
    {
        "new" =>
        {
            let friend: &str = opt.1.as_str();
            if friends.contains(&friend.to_string())
            {
                // open a new conversation with the friend
                // this is a placeholder for now
                println!("Opening a new conversation with {}", friend.blue());
            }
            else
            {
                utils::clear();
                utils::addl_message(format!("You don't have {} added as a friend.", friend.blue()).as_str(), "red");
                draw_msg(&user);
            }
        }
        "new --multi" =>
        {
            let user_friends = user.friends.clone();
            let listed_friends: Vec<&str> = opt.1.split(", ").collect();
            for friend in friends
            {
                if !user_friends.contains(&friend.to_string())
                {
                    utils::clear();
                    utils::addl_message(format!("You don't have {} added as a friend.", friend.blue()).as_str(), "red");
                    draw_msg(&user);
                    return;
                }
            }
            println!("Opening a new conversation with {}", listed_friends.join(", "));
            panic!("Not yet implemented.")
        }
        "open" =>
        {
            // open the conversation panel
            // this is a placeholder for now
            println!("Opening the conversation panel.");
        }
        _ =>
        {
            utils::clear();
            draw_msg(&user);
        }
    }
}

pub fn create_user(profile: &login::Profile) -> MessageUser
{
    // This function will find a user matching the profile in the accounts database, and create a messageuser database entry from it.
    let account_collection: mongodb::sync::Collection<Document> = mongo::get_collection("accounts");
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");

    let user: MessageUser = {
        match account_collection.find_one(doc! { "username": &profile.username }, None)
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
            let dbdata = retrieve_user_data(&user.username).unwrap();
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

fn retrieve_user_data(username: &str) -> Option<MessageUser>
{
    // This function will retrieve the user's data from the database and return it as a MessageUser. Ideally, don't do this often, because you don't want to spam the db.
    let user_collection: mongodb::sync::Collection<Document> = mongo::get_collection("messageusers");
    // messageusers and accounts are different, because the account coll holds passwords and shit that we don't need.
    match user_collection.find_one(doc! { "username": &username  }, None)
    {
        Ok(data) => match data
        {
            Some(d) => return Some(MessageUser::from_document(d)),
            None => return None
        },
        Err(_) => return None
    };
}

fn add_friend(user: &MessageUser, friend: &str) -> bool
{
    let friend: String = String::from(friend);
    let mut udata: MessageUser = retrieve_user_data(&user.username).unwrap(); //should never fail
    if retrieve_user_data(&friend).is_none()
    {
        return false;
    };
    if udata.friends.contains(&friend)
    {
        return false;
    }
    udata.friends.push(friend);
    match update_user_data(&udata)
    {
        Ok(_) => return true,
        Err(_) => return false
    }
    // TODO: blocklist? not necessary right now though.
}

fn remove_friend(user: &MessageUser, friend: &str) -> bool
{
    let friend: String = String::from(friend);
    let mut udata: MessageUser = retrieve_user_data(&user.username).unwrap();
    // shouldn't be any need to check if the friend exists, because that should have been checked when the friend was added.
    if !udata.friends.contains(&friend)
    {
        return false;
    }
    udata.friends.retain(|x| x != &friend);
    match update_user_data(&udata)
    {
        Ok(_) => return true,
        Err(_) => return false
    }
}

pub fn init(profile: &Profile)
{
    if let Some(user) = retrieve_user_data(&profile.username)
    {
        draw_home(&user);
    }
    else
    {
        // ???? we checked validity a hundred million times, so this should never run; just an extra measure i guess
        panic!("Opened the messenger with an invalid profile... How?")
    }
}

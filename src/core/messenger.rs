use super::{login, message_relay::{self, receive_messages, Conversation}, mongo, utils};
use colored::Colorize;
use login::Profile;
use mongodb::{bson::{doc, to_document, Document}, sync::Collection};
use serde::Deserialize;
use serde::Serialize;
use std::{fs::File, io::Write, time::{self, SystemTime}};
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
    let ui: Vec<String> = vec![
        welcome_message.as_str().to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "Please select an option.".to_string(),
        "".to_string(),
        "msg : opens the message panel".to_string(),
        "manage : manage your friends".to_string(),
        "logout : log out of your account.".to_string(),
    ];
    utils::create_ui(&ui, utils::Position::Center);
    let opt: (String, String) = utils::grab_opt(None, vec!["msg", "manage", "logout"]);
    match opt.0.as_str()
    {
        "msg" =>
        {
            utils::clear();
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
            let f = File::create("src/userdata/pkey.key").expect("no pkey file");
            serde_json::to_writer(BufWriter::new(f), "").expect("Failed to empty private key. Ensure pkey.key exists.");

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
    let mut ui: Vec<String> = vec!["Friends Management".to_string(), "".to_string(), "".to_string()];
    for friend in friends {
        ui.push(friend.to_string());
    }
    ui.push("".to_string());
    ui.push("add <friend> : adds friend by username".to_string());
    ui.push("rm <friend> : removes friend by username".to_string());
    ui.push("back : returns to home page".to_string());
    utils::create_ui(&ui, utils::Position::Center);
    let opt: (String, String) = utils::grab_opt(Some("Please input your option."), vec!["add", "rm", "back"]);
    match opt.0.as_str()
    {
        "add" =>
        {
            let friend: &str = opt.1.as_str();
            println!("{}", friend);
            if add_friend(&user, friend)
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
            if remove_friend(&user, friend)
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

fn draw_convo_ui (user: &MessageUser)
{
    // this function will draw the conversation panel
    // it will be a placeholder for now
    let mut ui: Vec<String> = vec!
    [
        "Conversations".to_string(), 
        "".to_string(), 
        "open <id> : open a conversation".to_string(), 
        "back : return to message panel".to_string(),
        "".to_string()
    ];
    let conversations: mongodb::sync::Cursor<Document> = mongo::get_collection("conversations").find(None, None).unwrap();
    for convo in conversations
    {
        let convo: Conversation = Conversation::from_document(convo.unwrap());
        // idk how to do a .find or a .any in a filter for .find
        if convo.users.contains(&user.username)
        {
            let users = convo.users.join(", ");
            let id = convo.id;
            let string = format!("{} : {}", id, users);
            ui.push(string);
        }
        
    }
    utils::create_ui(&ui, utils::Position::Center);
    let opt = utils::grab_opt(None, vec!["open", "back"]);
    match opt.0.as_str()
    {
        "open" =>
        {
            match mongo::get_collection("conversations").find(doc!("id": opt.1.as_str()), None)
            {
                Ok(convos) =>
                {
                    let convos = convos.into_iter().map(|x| x.unwrap()).collect::<Vec<Document>>();
                    if convos.len() == 0
                    {
                        utils::clear();
                        utils::addl_message("Conversation does not exist.", "red");
                        draw_convo_ui(&user);
                    }
                    // jerry rigged af
                    let convo = Conversation::from_document(convos.get(0).unwrap().clone());
                    draw_messenger_ui(&user, &convo);
                }
                Err(_) =>
                {
                    utils::clear();
                    utils::addl_message("Failed to retrieve conversation data.", "red");
                    draw_convo_ui(&user);
                }
            }
        }
        "back" =>
        {
            utils::clear();
            draw_msg(&user);
        }
        _ =>
        {
            draw_convo_ui(&user);
        }
    }
}

fn draw_messenger_ui(user: &MessageUser, convo: &Conversation)
{
    loop
    {
        let mut ui: Vec<String> = vec![
            "Messenger".to_string(),
            "".to_string(),
            "".to_string(),
        ];

        // TODO: handle this better
        
        let messages: Vec<message_relay::Message> = receive_messages(convo.id.as_str()).unwrap();
        for message in messages
        {
            let messagecontent: String = String::from_utf8(message.message).unwrap();
            // would be cool to color username but it adds hidden characters, maybe work around it
            let message: String = format!("{}: {}", message.sender, messagecontent).as_str().trim().to_string();
            println!("{}", message.len());
            ui.push(message);
        }
        
        ui.push("".to_string());
        ui.push("send <message> : send a message".to_string());
        ui.push("back : return to conversation list".to_string());
        utils::create_ui(&ui, utils::Position::Center);
        let opt: (String, String) = utils::grab_opt(None, vec!["send", "back"]);
        match opt.0.as_str()
        {
            "send" =>
            {
                let message = message_relay::Message { sender: user.username.clone(), message: opt.1.as_bytes().to_vec(), time: chrono::offset::Local::now().to_string() };
                message_relay::upload_message(message, &convo.id, &user.username).expect("failed to upload message");
                draw_messenger_ui(user, convo)
            }
            "back" =>
            {
                utils::clear();
                draw_convo_ui(user);
            }
            _ =>
            {
                utils::clear();
                draw_messenger_ui(user, convo);
            }
        }
    
    }
}


fn draw_msg(user: &MessageUser)
{
    //TODO: exit option
    let user: MessageUser = retrieve_user_data(&user.username).unwrap();
    let friends: &Vec<String> = &user.friends;
    let ui: Vec<String> = vec![
        "Message Panel".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "new <friend> : start a new single conversation with a friend.".to_string(),
        "new --multi <friend, friend> : start a new multi-person conversation.".to_string(),
        "open : view open conversations you are a participant in.".to_string(),
        "back : return to home page.".to_string(),
    ];
    utils::create_ui(&ui, utils::Position::Center);
    // flags aren't dynamic. I could fix that at some point but it's unnecessary right now.
    let opt: (String, String) = utils::grab_opt(None, vec!["new", "new --multi", "open", "back"]);
    match opt.0.as_str()
    {
        "new" =>
        {
            let friend: &str = opt.1.as_str();
            if friends.contains(&friend.to_string())
            {
                println!("Opening a new conversation with {}", friend.blue());
                super::message_relay::create_conversation(vec![user.username.clone(), friend.to_string()]);
                utils::clear();
                draw_convo_ui(&user)
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
            //utils::clear();
            draw_convo_ui(&user);
        }
        "back" =>
        {
            utils::clear();
            draw_home(&user);
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

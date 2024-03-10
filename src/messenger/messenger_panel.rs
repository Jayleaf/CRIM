use super::{
    message_relay::{self, receive_messages, Conversation, RawMessage}, 
    mongo, 
    utils,
    login,
    structs::Account
};
use colored::Colorize;
use mongodb::bson::{doc, Document};
use std::io::BufWriter;
use std::vec;
use std::fs::File;

//----------------------------------------------//
//                                              //
//            Front-End UI Functions            //
//                                              //
//----------------------------------------------//

pub fn draw_home_ui(user: &Account)
{
    /*
    Draws the home page for the messenger. This is the first page the user sees when they log in.
    */

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
            draw_messenger_home_ui(user);
        }
        "manage" =>
        {
            utils::clear();
            draw_friend_mgmt_ui(user);
        }
        "logout" =>
        {
            let f = File::create("src/userdata/pkey.key").expect("no pkey file");
            serde_json::to_writer(BufWriter::new(f), "").expect("Failed to empty private key. Ensure pkey.key exists.");
            login::login_init();
        }
        _ => {}
    }
}


pub fn draw_friend_mgmt_ui(user: &Account)
{
    /*
    Draws the friend management panel, where users can add/remove friends.
    */
    let user: Account = Account::get_account(&user.username).unwrap(); // the user arg can be trusted to have a proper username but not proper friends.
    let friends: &Vec<String> = &user.friends;
    let mut ui: Vec<String> = vec!["Friends Management".to_string(), "".to_string(), "".to_string()];
    for friend in friends
    {
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
                draw_friend_mgmt_ui(&user);
            }
            else
            {
                utils::clear();
                utils::addl_message(format!("User {} does not exist, or you already have them added.", friend.blue()).as_str(), "red");
                draw_friend_mgmt_ui(&user);
            }
        }
        "rm" =>
        {
            utils::clear();
            let friend: &str = opt.1.as_str();
            if remove_friend(&user, friend)
            {
                utils::addl_message("Successfully removed friend.", "green");
                draw_friend_mgmt_ui(&user);
            }
            else
            {
                utils::addl_message(format!("User {} does not exist, or you don't have them added.", friend.blue()).as_str(), "red");
                draw_friend_mgmt_ui(&user);
            }
        }
        "back" =>
        {
            draw_home_ui(&user);
        }
        _ => {}
    }
}


fn draw_convo_list_ui(user: &Account)
{
    /*
    Draws the direct conversation panel, which lists all the conversations the user is a part of.
    User can go back with the back command, or open a conversation with a given ID.
    */

    let mut ui: Vec<String> = vec![
        "Conversations".to_string(),
        "".to_string(),
        "open <id> : open a conversation".to_string(),
        "back : return to message panel".to_string(),
        "".to_string(),
    ];
    let conversations: mongodb::sync::Cursor<Document> = mongo::get_collection("conversations")
        .find(doc!("users": &user.username), None)
        .unwrap();
    let conversation_strings: Vec<String> = conversations.into_iter()
        .map(|x| Conversation::from_document(&x.unwrap()))
        .map(|y| format!("{} : {}", y.id, y.users.join(", ")))
        .collect();
    ui.extend(conversation_strings);
    
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
                    let convos = convos
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect::<Vec<Document>>();
                    if convos.is_empty()
                    {
                        utils::clear();
                        utils::addl_message("Conversation does not exist.", "red");
                        draw_convo_list_ui(user);
                    }
                    // jerry rigged af
                    let convo = Conversation::from_document(convos.first().unwrap());
                    draw_messenger_ui(user, &convo);
                }
                Err(_) =>
                {
                    utils::clear();
                    utils::addl_message("Failed to retrieve conversation data.", "red");
                    draw_convo_list_ui(user);
                }
            }
        }
        "back" =>
        {
            utils::clear();
            draw_messenger_home_ui(user);
        }
        _ => {}
    }
}

fn draw_messenger_ui(user: &Account, convo: &Conversation)
{
    /*
    The actual messenger UI. This is where the user can send and receive messages.
    */
    loop
    {
        let mut ui: Vec<String> = vec!
        [
            "Messenger".to_string(),
            "".to_string(),
            "".to_string()
        ];
        let messages: Vec<RawMessage> = receive_messages(convo.id.as_str()).unwrap();
        for message in messages
        {
            let messagecontent: String = String::from_utf8(message.message).unwrap();
            // would be cool to color username but it adds hidden characters, maybe work around it
            let message: String = format!("{}: {}", message.sender, messagecontent)
                .as_str()
                .trim()
                .to_string();
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
                let message = RawMessage
                { 
                    sender: user.username.clone(), 
                    message: opt.1.as_bytes().to_vec(), 
                    time: chrono::offset::Local::now().to_string()
                };
                message_relay::upload_message(&message, &convo.id).expect("failed to upload message");
                draw_messenger_ui(user, convo)
            }
            "back" =>
            {
                utils::clear();
                draw_convo_list_ui(user);
            }
            _ => {}
        }
    }
}

fn draw_messenger_home_ui(user: &Account)
{
    /*
    Draws the messenger home UI, which will let users start conversations or view the ones they're a part of.
    */

    let user: Account = Account::get_account(&user.username).unwrap();
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
                draw_convo_list_ui(&user)
            }
            else
            {
                utils::clear();
                utils::addl_message(format!("You don't have {} added as a friend.", friend.blue()).as_str(), "red");
                draw_messenger_home_ui(&user);
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
                    draw_messenger_home_ui(&user);
                    return;
                }
            }
            println!("Opening a new conversation with {}", listed_friends.join(", "));
            panic!("Not yet implemented.")
        }
        "open" =>
        {
            //utils::clear();
            draw_convo_list_ui(&user);
        }
        "back" =>
        {
            utils::clear();
            draw_home_ui(&user);
        }
        _ => {}
    }
}


//---------------------------------------------------------------------//
//                                                                     //
//                         Back-End Functions                          //
//(messenger specific back-end functions are found in message_relay.rs)//
//                                                                     //
//---------------------------------------------------------------------//



fn add_friend(user: &Account, friend: &str) -> bool
{
    let friend: String = String::from(friend);
    let mut udata: Account = Account::get_account(&user.username).unwrap(); //should never fail
    if Account::get_account(&friend).is_none()
    {
        return false;
    };
    if udata.friends.contains(&friend)
    {
        return false;
    }
    udata.friends.push(friend);
    Account::update_account(&udata).is_ok()
    // TODO: blocklist? not necessary right now though.
}


fn remove_friend(user: &Account, friend: &str) -> bool
{
    let friend: String = String::from(friend);
    let mut udata: Account = Account::get_account(&user.username).unwrap();
    // shouldn't be any need to check if the friend exists, because that should have been checked when the friend was added.
    if !udata.friends.contains(&friend)
    {
        return false;
    }
    udata.friends.retain(|x| x != &friend);
    Account::update_account(&udata).is_ok()
}


//----------------------------------------------//
//                                              //
//                Initialization                //
//                                              //
//----------------------------------------------//

pub fn init(account: &Account)
{
    if let Some(user) = Account::get_account(&account.username)
    {
        draw_home_ui(&user);
        // &user is passed around like herpes. May be a better way to store it.
    }
    else
    {
        // ???? we checked validity a hundred million times, so this should never run; just an extra measure i guess
        panic!("Opened the messenger with an invalid profile... How?")
    }
}

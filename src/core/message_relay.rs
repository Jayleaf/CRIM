// whole new file for this lol

use mongodb::bson::{self, doc};
use openssl::{pkey::{Private, Public}, rsa::{Padding, Rsa}};
use serde::Serialize;
use super::mongo;
use mongodb::bson::{Bson, Document};

#[derive(Serialize)]
struct Message
{
    message: Vec<u8>,
    time: String
}

impl Message
{
    fn from_document (doc: Document) -> Message
    {
        let message: Vec<u8> = doc.get("message").unwrap().as_array().unwrap().iter().map(|x| x.as_i32().unwrap() as u8).collect();
        let time: String = doc.get("time").unwrap().as_str().unwrap().to_string();
        Message { message, time }
    }
}

#[derive(Serialize)]
struct Conversation
{
    users: Vec<String>,
    messages: Vec<Message>,
}

impl Conversation
{
    fn from_document (doc: Document) -> Conversation
    {
        let users: Vec<String> = doc.get("users").unwrap().as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let messages: Vec<Message> = doc.get("messages").unwrap().as_array().unwrap().iter().map(|x| Message { message: x.as_document().unwrap().get("message").unwrap().as_array().unwrap().iter().map(|x| x.as_i32().unwrap() as u8).collect(), time: x.as_document().unwrap().get("time").unwrap().as_str().unwrap().to_string() }).collect();
        Conversation { users, messages }
    }
}

fn create_conversation(users: Vec<String>)
{
    let conversation = Conversation { users: vec![], messages: vec![] };
    let doc = bson::to_document(&serde_json::to_value(&conversation).unwrap()).unwrap();
    mongo::get_collection("conversations").insert_one(doc, None).unwrap();

}

fn encrypt_message(message: &str, public_key: Rsa<Public>) -> Vec<u8>
{
    // encrypts message and returns byte array
    let mut encrypted_message = vec![0; public_key.size() as usize];
    public_key.public_encrypt(message.as_bytes(), &mut encrypted_message, Padding::PKCS1).expect("failed to encrypt message");
    encrypted_message
}

fn decrypt_message(encrypted_message: &[u8], private_key: Rsa<Private>) -> String
{
    // decrypts message and returns string
    let mut decrypted_message = vec![0; private_key.size() as usize];
    private_key.private_decrypt(encrypted_message, &mut decrypted_message, Padding::PKCS1).expect("failed to decrypt message");
    String::from_utf8(decrypted_message).expect("failed to convert decrypted message to string")
}

fn upload_message(message: &str, convo_id: &str, sender: &str) -> Result<(), String>
{
    // uploads message to database
    
    match mongo::get_collection("conversations").find_one(Some(doc! {"_id": convo_id}), None)
    {
        Ok(convo) => match convo
        {
            Some(doc) => {
                let mut conversation: Conversation = Conversation::from_document(doc);
                // encrypt given message with the public key of the other user
                let message = encrypt_message(&String::from(message), Rsa::public_key_from_pem(conversation.users.iter().find(|x| *x != sender).unwrap().as_bytes()).unwrap());
                conversation.messages.push(Message { message: message.to_vec(), time: chrono::Utc::now().to_string() });
                let doc = bson::to_document(&serde_json::to_value(&conversation).unwrap()).unwrap();
                mongo::get_collection("conversations").update_one(doc! {"_id": convo_id}, doc, None).unwrap();
                Ok(())
            },
            None => Err("conversation not found".to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

//TODO: write recieve_message

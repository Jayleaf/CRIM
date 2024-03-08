// whole new file for this lol

use std::{fs::{self, File}, io::Read};

use mongodb::bson::{self, doc};
use openssl::{pkey::{Private, Public}, rsa::{Padding, Rsa}};
use serde::{Deserialize, Serialize};
use super::mongo;
use mongodb::bson::{Bson, Document};


/*
Currently, the sender of the message cannot read their own message. This will have to be fixed as follows.
https://stackoverflow.com/questions/63152965/how-does-the-sender-decrypt-his-own-encrypted-message
*/

#[derive(Serialize, Deserialize)]
pub struct Message
{
    pub sender: String,
    pub message: Vec<u8>,
    pub time: String
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedMessage
{
    pub data: Vec<u8>
}

impl EncryptedMessage
{
    fn from_document(doc: &Document) -> EncryptedMessage
    {
        let data: Vec<u8> = doc.get("data").unwrap().as_array().unwrap().iter().map(|x| x.as_i64().unwrap() as u8).collect();
        EncryptedMessage { data }
    }
}

impl Message
{
    fn from_document (doc: Document) -> Message
    {
        let sender: String = doc.get("sender").unwrap().as_str().unwrap().to_string();
        let message: Vec<u8> = doc.get("message").unwrap().as_array().unwrap().iter().map(|x| x.as_i32().unwrap() as u8).collect();
        let time: String = doc.get("time").unwrap().as_str().unwrap().to_string();
        Message { sender, message, time }
    }
}

#[derive(Serialize)]
pub struct Conversation
{
    pub id: String,
    pub users: Vec<String>,
    pub messages: Vec<EncryptedMessage>,
}

impl Conversation
{
    pub fn from_document (doc: Document) -> Conversation
    {
        let id: String = doc.get_str("id").unwrap().to_string();
        let users: Vec<String> = doc.get("users").unwrap().as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let messages: Vec<EncryptedMessage> = doc.get("messages").unwrap().as_array().unwrap().iter().map(|x| EncryptedMessage::from_document(x.as_document().unwrap())).collect();
        Conversation { id, users, messages }
    }
}

pub fn create_conversation(users: Vec<String>)
{
    let conversation = Conversation { id: super::utils::rand_hex(), users: users, messages: vec![] };
    let doc = bson::to_document(&serde_json::to_value(&conversation).unwrap()).unwrap();
    mongo::get_collection("conversations").insert_one(doc, None).unwrap();

}

fn encrypt_message(message: Message, public_key: Rsa<Public>) -> EncryptedMessage
{
    // encrypts serialized message object and returns byte array
    let message: String = serde_json::to_string(&message).unwrap();
    let mut encrypted_message: Vec<u8> = vec![0; public_key.size() as usize];
    public_key.public_encrypt(message.as_bytes(), &mut encrypted_message, Padding::PKCS1).expect("failed to encrypt message");
    EncryptedMessage{data: encrypted_message}
}

fn decrypt_message(encrypted_message: &EncryptedMessage, private_key: &Rsa<Private>) -> Message
{
    // decrypts message and returns string
    let mut decrypted_message: Vec<u8> = vec![0; private_key.size() as usize];
    private_key.private_decrypt(&encrypted_message.data, &mut decrypted_message, Padding::PKCS1).expect("failed to decrypt message");
    // from_str is so funny. shit's like magic. just guesses what type it should be
    let raw_str: String = String::from_utf8(decrypted_message).unwrap().trim_matches('\0').to_string();
    serde_json::from_str(&raw_str).unwrap()
}

pub fn upload_message(message: Message, convo_id: &str, sender: &str) -> Result<(), String>
{
    /*
    Encrypt message with the other user's public key, and upload it to the conversation stored in the db.
    */
    
    match mongo::get_collection("conversations").find_one(Some(doc! {"id": convo_id}), None)
    {
        Ok(convo) => match convo
        {
            Some(doc) => {
                let mut conversation: Conversation = Conversation::from_document(doc);
                // encrypt given message with the public key of the other user
                let tgt_pub_key: Document = mongo::get_collection("accounts").find(doc! {"username": conversation.users.iter().find(|x| *x != sender)}, None).unwrap().current().try_into().unwrap();
                let tgt_pub_key: Vec<u8> = tgt_pub_key.get_array("public_key").unwrap().into_iter().map(|x| x.as_i64().unwrap() as u8).collect();
                let message: EncryptedMessage = encrypt_message(message, Rsa::public_key_from_pem(&tgt_pub_key).unwrap());
                conversation.messages.push(message);
                let doc = bson::to_document(&serde_json::to_value(&conversation).unwrap()).unwrap();
                // fix line below
                mongo::get_collection("conversations").replace_one(doc!("id": conversation.id), doc, None).unwrap();
                Ok(())
            },
            None => Err("conversation not found".to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

pub fn receive_messages(convo_id: &str) -> Result<Vec<Message>, String>
{
    /*
    Returns a vector of decrypted messages, decrypted using the locally stored private key in pkey.key
    */
    match mongo::get_collection("conversations").find_one(Some(doc! {"id": convo_id}), None)
    {
        Ok(convo) => match convo
        {
            Some(doc) => {
                let conversation: Conversation = Conversation::from_document(doc);
                let mut messages: Vec<Message> = vec![];
                let key: String = fs::read_to_string("src/userdata/pkey.key").expect("failed to open key file");
                let key = Rsa::private_key_from_pem(key.as_bytes()).unwrap();
                for message in conversation.messages.iter()
                {
                    messages.push(decrypt_message(message, &key));
                }
                Ok(messages)
            },
            None => Err("conversation not found".to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

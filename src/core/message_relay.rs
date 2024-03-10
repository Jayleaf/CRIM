use std::fs;
use super::mongo;
use getrandom::getrandom;
use mongodb::bson::{self, doc};
use mongodb::bson::Document;
use openssl::{
    pkey::{Private, Public}, rsa::{Padding, Rsa}, symm
};
use serde::{Deserialize, Serialize};

/*
Currently, the sender of the message cannot read their own message. This will have to be fixed as follows.
https://stackoverflow.com/questions/63152965/how-does-the-sender-decrypt-his-own-encrypted-message
*/


//----------------------------------------------//
//                                              //
//          Structs & Implementations           //
//                                              //
//----------------------------------------------//

#[derive(Serialize, Deserialize)]
pub struct RawMessage
{
    pub sender: String,
    pub message: Vec<u8>,
    pub time: String
}

impl RawMessage
{
    fn from_document(doc: Document) -> RawMessage
    {
        let sender: String = doc
            .get("sender")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let message: Vec<u8> = doc
            .get("message")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i32().unwrap() as u8)
            .collect();
        let time: String = doc
            .get("time")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        RawMessage { sender, message, time }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct UserKey
{
    owner: String,
    key: Vec<u8>
}

impl UserKey
{
    fn from_document(doc: &Document) -> UserKey
    {
        let owner: String = doc.get_str("owner").unwrap().to_string();
        let key: Vec<u8> = doc
            .get_array("key")
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as u8)
            .collect();
        UserKey { owner, key }
    }
    fn encrypt(key: &[u8], user: &String) -> UserKey
    {
        let pub_key: Vec<u8> = mongo::get_collection("accounts")
            .find_one(doc! {"username": user}, None)
            .unwrap()
            .unwrap()
            .get_array("public_key")
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as u8)
            .collect();
        let pub_key: Rsa<Public> = Rsa::public_key_from_pem(pub_key.as_slice()).expect("Failed to retrieve a public key from database.");
        let mut encrypted_key: Vec<u8> = vec![0; pub_key.size() as usize];
        pub_key
            .public_encrypt(key, &mut encrypted_key, Padding::PKCS1)
            .expect("failed to encrypt key");
        UserKey { owner: user.clone(), key: encrypted_key }
    }
    fn decrypt(&mut self, encrypted_key: Vec<u8>, priv_key: Rsa<Private>) -> UserKey
    {
        let mut decrypted_key: Vec<u8> = vec![0; priv_key.size() as usize];
        priv_key
            .private_decrypt(&encrypted_key, &mut decrypted_key, Padding::PKCS1)
            .expect("failed to decrypt key");
        UserKey { owner: String::clone(&self.owner), key: decrypted_key }
    }
}


#[derive(Serialize, Deserialize)]
pub struct EncryptedMessage
{
    pub data: Vec<u8> // data contains a serialized message struct. see diagram in readme.md for more info.
}

impl EncryptedMessage
{
    fn from_document(doc: &Document) -> EncryptedMessage
    {
        let data: Vec<u8> = doc
            .get("data")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as u8)
            .collect();
        EncryptedMessage { data }
    }
}


#[derive(Serialize)]
pub struct Conversation
{
    pub id: String,
    pub users: Vec<String>,
    pub keys: Vec<UserKey>,
    pub messages: Vec<EncryptedMessage>
}

impl Conversation
{
    pub fn from_document(doc: &Document) -> Conversation
    {
        let id: String = doc.get_str("id").unwrap().to_string();
        let users: Vec<String> = doc
            .get("users")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        let messages: Vec<EncryptedMessage> = doc
            .get("messages")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| EncryptedMessage::from_document(x.as_document().unwrap()))
            .collect();
        let keys: Vec<UserKey> = doc
            .get("keys")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| UserKey::from_document(x.as_document().unwrap()))
            .collect();
        Conversation { id, users, messages, keys }
    }
}

//----------------------------------------------//
//                                              //
//            Conversation Functions            //
//                                              //
//----------------------------------------------//

pub fn create_conversation(users: Vec<String>)
{
    /*
    Creates a conversation object and uploads it to the database.
    The conversation ID contains a unique conversation ID, encrypted with each user's public key.
    For more information, see the diagram in readme.md.
    */

    let mut raw_conversation_key: [u8; 128] = [0; 128];
    getrandom(&mut raw_conversation_key).expect("Failed to generate random conversation key.");
    let conversation = Conversation {
        id: super::utils::rand_hex(),
        users: users.clone(),
        keys: users
            .clone()
            .iter()
            .map(|x| UserKey::encrypt(&raw_conversation_key, x))
            .collect(),
        messages: vec![]
    };
    let doc = bson::to_document(&serde_json::to_value(conversation).unwrap()).unwrap();
    mongo::get_collection("conversations")
        .insert_one(doc, None)
        .unwrap();
}

//----------------------------------------------//
//                                              //
//             Encryption Functions             //
//                                              //
//----------------------------------------------//

fn encrypt_message(message: &RawMessage, convo: &Conversation) -> EncryptedMessage
{
    /*
    Encrypts a message object. For more info as to how this function works, see the diagram in readme.md.
    */

    // first, get the public-key encrypted conversation key that belongs to the other user (will need to be refactored big time for multiple users)
    let mut convokey: UserKey = convo
        .keys
        .iter()
        .find(|x| x.owner != message.sender.as_str())
        .unwrap()
        .clone();

    // then, decrypt that with your private key
    let priv_key: String = fs::read_to_string("src/userdata/pkey.key")
        .expect("There was a problem reading your private key from pkey.key.");
    let priv_key: Rsa<Private> = Rsa::private_key_from_pem(priv_key.as_bytes())
        .expect("There was a problem parsing the private key in pkey.key.");
    let mut decrypted_convo_key: Vec<u8> = vec![0; priv_key.size() as usize];
    priv_key
        .private_decrypt(convokey.key.as_slice(), &mut decrypted_convo_key, Padding::PKCS1)
        .expect("failed to decrypt convo key");
    convokey.key = decrypted_convo_key.to_vec();

    // now, serialize the message payload, encrypt that serialized payload, and return the encrypted message object.
    let serialized_message: String = serde_json::to_string(&message).unwrap();
    let cipher: symm::Cipher = symm::Cipher::aes_256_cbc();
    let encrypted_message_struct: Vec<u8> = symm::encrypt(cipher, convokey.key.as_slice(), None, serialized_message.as_bytes()).unwrap();

    // TODO: you stopped here. start to decrypt the messages next.
    EncryptedMessage { data: encrypted_message_struct }
}

pub fn upload_message(message: RawMessage, convo_id: &str, sender: &str) -> Result<(), String>
{
    /*
    Encrypt message with the other user's public key, and upload it to the conversation stored in the db.
    */

    match mongo::get_collection("conversations").find_one(Some(doc! {"id": convo_id}), None)
    {
        Ok(convo) => match convo
        {
            Some(doc) =>
            {
                let mut conversation: Conversation = Conversation::from_document(&doc);
                // encrypt given message
                let message: EncryptedMessage = encrypt_message(&message, &conversation);
                conversation.messages.push(message);
                let doc = bson::to_document(&serde_json::to_value(&conversation).unwrap()).unwrap();
                // fix line below
                mongo::get_collection("conversations")
                    .replace_one(doc!("id": conversation.id), doc, None)
                    .unwrap();
                Ok(())
            }
            None => Err("Could not find the conversation to uplaod to.".to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

//----------------------------------------------//
//                                              //
//             Decryption Functions             //
//                                              //
//----------------------------------------------//

fn decrypt_message(encrypted_message: &EncryptedMessage, private_key: &Rsa<Private>) -> RawMessage
{
    /*
    Takes in an encrypted message object and a private key, and returns a decrypted message object.
    */

    let mut decrypted_message: Vec<u8> = vec![0; private_key.size() as usize];
    private_key
        .private_decrypt(&encrypted_message.data, &mut decrypted_message, Padding::PKCS1)
        .expect("failed to decrypt message");
    let raw_str: String = String::from_utf8(decrypted_message)
        .unwrap()
        .trim_matches('\0') // remove trailing null bytes from when the serialized message was in bson form
        .to_string();
    serde_json::from_str(&raw_str).unwrap()
}

pub fn receive_messages(convo_id: &str) -> Result<Vec<RawMessage>, String>
{
    /*
    Returns a vector of decrypted messages, decrypted using the locally stored private key in pkey.key.
    This is the umbrella function called in messenger.rs to load all messages from a conversation.
    */

    match mongo::get_collection("conversations").find_one(Some(doc! {"id": convo_id}), None)
    {
        Ok(convo) => match convo
        {
            Some(doc) =>
            {
                let conversation: Conversation = Conversation::from_document(&doc);
                let mut messages: Vec<RawMessage> = vec![];
                let key: String = fs::read_to_string("src/userdata/pkey.key").expect("failed to open key file");
                let key = Rsa::private_key_from_pem(key.as_bytes()).unwrap();
                for message in conversation.messages.iter()
                {
                    messages.push(decrypt_message(message, &key));
                }
                Ok(messages)
            }
            None => Err("conversation not found".to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

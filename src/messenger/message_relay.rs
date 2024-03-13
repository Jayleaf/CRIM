use std::fs;
use super::{mongo, structs::Account};
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


#[derive(Serialize, Deserialize, Clone, Default, Debug)]
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
        let pub_key: Vec<u8> = Account::get_account(user).unwrap().public_key;
        let pub_key: Rsa<Public> = Rsa::public_key_from_pem(pub_key.as_slice()).expect("Failed to retrieve a public key from database.");
        let mut encrypted_key: Vec<u8> = vec![0; pub_key.size() as usize];
        pub_key
            .public_encrypt(key, &mut encrypted_key, Padding::PKCS1)
            .expect("failed to encrypt key");
        UserKey { owner: user.clone(), key: encrypted_key }
    }
    fn decrypt(&self, encrypted_key: &[u8]) -> UserKey
    {
        let priv_key: String = fs::read_to_string("src/userdata/pkey.key").expect("failed to open key file");
        let priv_key: Rsa<Private> = Rsa::private_key_from_pem(priv_key.as_bytes()).expect("failed to parse private key");
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

    pub fn get(id: &str) -> Option<Conversation>
    {
        let doc: Option<Document> = mongo::get_collection("conversations")
            .find_one(Some(doc! {"id": id}), None)
            .unwrap();
        match doc
        {
            Some(doc) => Some(Conversation::from_document(&doc)),
            None => None
        }
        
    
    }
}

//----------------------------------------------//
//                                              //
//            Conversation Functions            //
//                                              //
//----------------------------------------------//

/// Creates a conversation object and uploads it to the database.
/// The conversation ID contains a unique conversation ID, encrypted with each user's public key. 
/// For more information, see the diagram in readme.md.
pub fn create_conversation(users: Vec<String>)
{

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

/// Encrypts a RawMessage value with the conversation's unique key, and returns an EncryptedMessage value.
/// 
/// Gets the conversation key from the conversation value that corresponds to the recipient, decrypts it with the sender's private key, serializes the RawMessage, re-encrypts it with the decrypted conversation key, and returns an EncryptedMessage value.
fn encrypt_message(message: &RawMessage, convo: &Conversation) -> EncryptedMessage
{

    // first, get the public-key encrypted conversation key that belongs to you
    let convokey: UserKey = convo
        .keys
        .iter()
        .find(|x| x.owner == message.sender.as_str())
        .unwrap()
        .clone();

    // then, decrypt that with your private key
    let decrypted_key: Vec<u8> = UserKey::decrypt(&convokey, &convokey.key).key.as_slice().to_vec();
    // now, serialize the message payload, encrypt that serialized payload, and return the encrypted message object.
    let serialized_message: String = serde_json::to_string(&message).unwrap();
    let cipher: symm::Cipher = symm::Cipher::aes_128_cbc();
    let encrypted_message_struct: Vec<u8> = symm::encrypt(cipher, &decrypted_key, None, serialized_message.as_bytes()).unwrap();

    // TODO: you stopped here. start to decrypt the messages next.
    EncryptedMessage { data: encrypted_message_struct }
}

/// Uploads a RawMessage to a conversation in the database.* This is the entry point for sending a message, as `encrypt()` shouldn't be called directly.
/// 
/// *Actually replaces an existing conversation entry with a new one containing the new message, because `update_one()` was a pain in my ass.
pub fn upload_message(message: &RawMessage, convo_id: &str) -> Result<(), String>
{
    match Conversation::get(convo_id)
    {
        
            Some(mut convo) =>
            {   let message: EncryptedMessage = encrypt_message(message, &convo);
                convo.messages.push(message);
                let doc = bson::to_document(&serde_json::to_value(&convo).unwrap()).unwrap();
                // TODO: make this an implementation of the conversation struct. Conversation::update()
                mongo::get_collection("conversations")
                    .replace_one(doc!("id": convo.id), doc, None)
                    .unwrap();
                Ok(())
            }
            None => Err("Could not find the conversation to uplaod to.".to_string())
    }
}

//----------------------------------------------//
//                                              //
//             Decryption Functions             //
//                                              //
//----------------------------------------------//

/// Takes in a reference to an EncryptedMessage value and a private key ref, and spits out a RawMessage decrypted with the provided private key.
fn decrypt_message(caller: &str, encrypted_message: &EncryptedMessage, private_key: &Rsa<Private>, convo_id: &str) -> RawMessage
{
    // retrieve conversation object from db
    let convo: Conversation = 
        if let Some(convo) = Conversation::get(convo_id) {convo} 
        else {panic!("Could not find conversation to decrypt message from.")};
    // decrypt conversation key corresponding to you
    let mut convokey: UserKey = convo
        .keys
        .iter()
        .find(|x| x.owner == caller)
        .unwrap()
        .clone();
    let mut decrypted_convo_key: Vec<u8> = vec![0; private_key.size() as usize];
    private_key
        .private_decrypt(convokey.key.as_slice(), &mut decrypted_convo_key, Padding::PKCS1)
        .expect("failed to decrypt convo key");
    convokey.key = decrypted_convo_key.to_vec();
    // decrypt the message with the decrypted conversation key
    let cipher: symm::Cipher = symm::Cipher::aes_128_cbc();
    let decrypted_message: Vec<u8> = symm::decrypt(cipher, convokey.key.as_slice(), None, encrypted_message.data.as_slice()).unwrap();
    // deserialize the message
    let message: RawMessage = serde_json::from_str(&String::from_utf8(decrypted_message).unwrap()).unwrap();
    message

}

/// Takes in a conversation ID and returns a Result, either containing a Vec of RawMessages containing all decrypted messages, or a string if no messages were present in the conversation.
///
/// Finds a conversation matching the conversation id, reads the user's private key from pkey.key, and decrypts all messages in the conversation value.

pub fn receive_messages(caller: &str, convo_id: &str) -> Result<Vec<RawMessage>, String>
{
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
                messages.extend(conversation.messages.iter().map(|x| decrypt_message(caller, x, &key, convo_id)));
                Ok(messages)
            }
            None => Err("conversation not found".to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

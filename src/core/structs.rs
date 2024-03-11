//----------------------------------------------//
//                                              //
//        File for commonly-used structs        //
//                                              //
//----------------------------------------------//

use mongodb::bson;
use serde::{Deserialize, Serialize};

use super::mongo;


//----------------------------------------------//
//                                              //
//                User Accounts                 //
//                                              //
//----------------------------------------------//

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Account
{
    pub username: String,
    pub hash: String,
    pub salt: Vec<u8>,
    pub public_key: Vec<u8>,
    pub priv_key_enc: Vec<u8>,
    pub friends: Vec<String>
}

impl Account
{
    /// Parses a BSON Document into an account value
    pub fn from_document(doc: bson::Document) -> Account
    {
        Account {
            username: doc.get_str("username").unwrap().to_string(),
            hash: doc.get_str("hash").unwrap().to_string(),
            salt: doc
                .get_array("salt")
                .unwrap()
                .iter()
                .map(|x| x.as_i32().unwrap() as u8)
                .collect::<Vec<u8>>(),
            public_key: doc
                .get_array("public_key")
                .unwrap()
                .iter()
                .map(|x| x.as_i32().unwrap() as u8)
                .collect::<Vec<u8>>(),
            priv_key_enc: doc
                .get_array("priv_key_enc")
                .unwrap()
                .iter()
                .map(|x| x.as_i32().unwrap() as u8)
                .collect::<Vec<u8>>(),
            friends: doc
                .get_array("friends")
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect()
        }
    }

    /// Takes in a string, finds the matching account in the database, and returns it. Will return none if no account is found, or will panic if it fails to access a database.
    pub fn get_account(username: &String) -> Option<Account>
    {
        let doc = mongo::get_collection("accounts").find(
            bson::doc! { "username": username },
            None
        );
        match doc
        {
            Err(_) => panic!("An error occurred querying the database for an account."),
            Ok(doc) => {
                let doc: Vec<Account> = doc.into_iter().map(|x| Account::from_document(x.unwrap())).collect();
                if doc.is_empty()
                {
                    return None;
                }
                Some(doc.first().unwrap().clone())
            }
        }
    }

    /// Takes in an account value reference, and updates the first database entry with the same username. If the update is successful, it will return the account. If not, it will return an error. Most errors from this will likely be from trying to update a non-existent account.
    pub fn update_account(new: &Account) -> Result<Account, mongodb::error::Error>
    {
        let result = mongo::get_collection("accounts").update_one(
            bson::doc! { "username": &new.username },
            bson::doc! { "$set": bson::to_document(&new).unwrap() },
            None
        );
        match result
        {
            Ok(_) =>
            {
                Ok(Account::from_document(
                    mongo::get_collection("accounts").find_one(
                        bson::doc! { "username": &new.username },
                        None
                    )
                    .unwrap()
                    .unwrap()
                    )
                )
            }
            Err(result) => Err(result)
        }
    } 

    /// Finds the first instance of a database account entry with a given username, and removes it. Returns an empty result.
    pub fn delete_account(username: &String) -> Result<(), mongodb::error::Error>
    {
        match mongo::get_collection("accounts").delete_one(
            bson::doc! { "username": username },
            None
        )
        {
            Ok(_) => Ok(()),
            Err(result) => Err(result)
        }
    }

    /// Creates a new account entry from a given account value ref. Returns the account if successful, or an error if not. Most errors from this will be from faults in database setup.
    pub fn create_account(new: &Account) -> Result<Account, mongodb::error::Error>
    {
        let result = mongo::get_collection("accounts").insert_one(
            bson::to_document(&new).unwrap(),
            None
        );
        match result
        {
            Ok(_) => Ok(Account::from_document(
                mongo::get_collection("accounts").find_one(
                    bson::doc! { "username": &new.username },
                    None
                )
                .unwrap()
                .unwrap()
                )
            ),
            Err(result) => Err(result)
        }
    }
}  

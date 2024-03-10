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

    pub fn update_account(new: &Account) -> Result<Account, mongodb::error::Error>
    {
        /*
        Takes in an account and if it updates successfully, will return the new updated account. If not, returns an error.
        */
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

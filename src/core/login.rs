extern crate dotenv;
use super::utils;
use super::mongo;
use dotenv::dotenv;
use mongodb::{ bson::doc, bson::Document, bson::to_document, sync::Client };
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use uuid::Uuid;
use colored::Colorize;

/*

This file handles the login system of CRIM.
Profiles.json is a local cache of accounts, to allow for quick sign in.
Any account that is being logged in with will be checked against the account database in the server so as to prevent fake accounts; registering is necessary.
Registering is not possible yet because I haven't set up the DB lol

*/

/*

Structs

*/

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
struct Profile
{
	username: String,
	password: String,
	account_uuid: String,
}


impl Default for Profile
{
	fn default() -> Profile { Profile { username: String::new(), password: String::new(), account_uuid: String::new() } }
}

#[derive(Deserialize, Serialize, Debug)]
struct ProfileContainer
{
	profiles: Vec<Profile>,
}

/*

	Login-Specific Utility Functions

*/

fn deserialize_profile_data(holder: &mut ProfileContainer)
{
	/*
	   Deserialize the data in the profiles json file, and return it.
	*/

	let mut f: File = fs::File::open("src/userdata/profiles.json").unwrap();
	let mut data: String = String::new();
	f.read_to_string(&mut data).unwrap();
	let profiles: ProfileContainer = {
		let pc: Result<_, serde_json::Error> = serde_json::from_str(&data.as_str());
		if pc.is_ok()
		{
			pc.unwrap()
		}
		else
		{
			ProfileContainer { profiles: Vec::new() }
		}
	};

	*holder = profiles;
}

fn serialize_profile_data(container: ProfileContainer)
{
	/*
		Serializes profile data and writes it to the profiles json file.
	*/

	let serialized_data: Result<String, serde_json::Error> = to_string(&container);
	fs::write("src/userdata/profiles.json", &serialized_data.unwrap()).expect("Failed to write.");
}

fn validate_login_info(profile_to_be_validated: &Profile) -> bool
{
	/*

	   First, ensure that whatever profile we're trying to sign into is in profiles.json.
	   Then, check it against the database to ensure the account exists.
	   If successful, return true.

	   This function can easily broken up and modified for exploitation, but this function serves no major purpose except error prevention.
	   This function has no real effect in logging in, the login function does all of that.

	*/

	let mut status: bool = false;

	let mut profile_data: ProfileContainer = ProfileContainer { profiles: Vec::new() };
	deserialize_profile_data(&mut profile_data);
	for profile in profile_data.profiles
	{
		if profile.username == profile_to_be_validated.username && profile.password == profile_to_be_validated.password
		{
			status = true;
			break;
		}
		status = false;
	}

	/*
		for logging in, this will be the method.
		query mongodb for the account ID of the matching profile.
		if successful, return the account ID and log in.

	*/

	status
}

/*

	Interactable Functions

*/

fn register_profile()
{
	utils::clear();
	let mut username: String = String::new();
	println!("Enter the username for your new profile. This will be your display name. : ");
	io::stdin().read_line(&mut username).expect("Uh oh! Failed to read the line.");
	let mut password: String = String::new();
	println!("Enter the password for your new profile. : ");
	io::stdin().read_line(&mut password).expect("Uh oh! Failed to read the line.");
	username.pop();
	password.pop();

	let new_profile: Profile = Profile { username: String::from(&username), password: String::from(&password), account_uuid: Uuid::new_v4().to_string() };
	utils::clear();

	// save the data to profiles.json here.

	let mut deserialized_data: ProfileContainer = ProfileContainer { profiles: Vec::new() };
	deserialize_profile_data(&mut deserialized_data);
	deserialized_data.profiles.push(Profile::clone(&new_profile));
	serialize_profile_data(deserialized_data);

	/*
		mang- i mean mongo time!
	 */

	let db = mongo::get_database("CRIM");
	let coll = db.collection::<Document>("accounts");
	let doc: Result<Document, mongodb::bson::ser::Error> = to_document(&serde_json::to_value(&new_profile).unwrap());
	let _ = coll.insert_one(doc.unwrap(), None);

	println!("Created profile. Validating...");
	let validation_status: bool = validate_login_info(&Profile::clone(&new_profile));
	if validation_status == true
	{
		println!("Profile Validated. Return to login screen...");
	}
	else
	{
		println!("Profile was not validated. Return to login screen.");
	}
}

fn select_profile() -> Profile
{
	/*
	   Function prompts user to select a profile. If invalid profile is selected, returns an empty profile.
	*/
	let mut selected_profile: Profile = Profile::default();
	while true
	{
		utils::clear();
		println!("Please select one of your profiles, or type B to go back. : \n \n");
		let mut profile_data: ProfileContainer = ProfileContainer { profiles: Vec::new() };
		deserialize_profile_data(&mut profile_data);
		let mut counter: i32 = 0;
		let mut profile_hashmap: HashMap<i32, Profile> = HashMap::new();
		for profile in profile_data.profiles
		{
			counter += 1;
			profile_hashmap.insert(counter, Profile::clone(&profile));
			println!("{} | ({})", utils::pad_string(String::from(&profile.username), 16), counter);
		}
		let mut selection: String = String::new();
		io::stdin().read_line(&mut selection).expect("Failed to read line.");
		selection = String::from(selection.trim());
		let _ = io::stdout().flush();
		let potential_selected_profile: Profile = {
			let hash_obj: Option<&Profile> = {
				// Try to handle all cases of invalid inputs.
				if selection.as_str() == "B" || selection.as_str() == "b" { login_init() }
				if selection.as_str().parse::<i32>().is_err() { continue }
				profile_hashmap.get(&selection.as_str().parse::<i32>().unwrap())
			};
			match hash_obj
			{
				None => Profile { username: String::new(), password: String::new(), account_uuid: String::new() },
				_ => Profile { username: String::from(&hash_obj.unwrap().username), password: String::from(&hash_obj.unwrap().password), account_uuid: String::from(&hash_obj.unwrap().account_uuid) },
			}
		};
		if validate_login_info(&Profile::clone(&potential_selected_profile)) == true
		{
			// uhhhh maybe scope problem!?!?!?!?!?! idk
			selected_profile = potential_selected_profile;
			break;
		}
	}
	selected_profile
}

fn login(p: Profile) -> bool
{
	validate_login_info(&p);
	/*
		You'd do mongo validation here.
	 */
	false
}

pub fn login_select_profile()
{
	let selected_profile: Profile = { select_profile() };
	utils::clear();
	println!("Logging you in with local profile {}...", &selected_profile.username.red());
	/*
		Call to login. Now the shitshow begins.
	 */
	let res: bool = login(Profile::clone(&selected_profile)); // 
	if res == true
	{
		println!("Successfully logged you in as {}. Opening messenger...", &selected_profile.username.red())
	}
}

pub fn login_init()
{
	utils::clear();
	println!("{}", std::env::current_dir().unwrap().display());
	dotenv().ok();
	if dotenv::var("UUID").unwrap() == ""
	{
		println!("Looks like you're not logged in. Let's fix that. \n \n");
		println!("Register New Profile    (1)");
		println!("Select Existing Profile (2)");
		println!("Exit                    (3)");

		let mut selection: String = String::new();
		io::stdin().read_line(&mut selection).expect("Failed to read the line.");
		selection.pop();
		match selection.as_str()
		{
			"1" => register_profile(),
			"2" => login_select_profile(),
			"3" => std::process::exit(0),
			_ => login_init(),
		}
	}
	else
	{
		println!("You're logged in.")
	}
}

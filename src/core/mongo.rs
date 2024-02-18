use mongodb::sync::Database;
use mongodb::{
	bson::doc, options::{ClientOptions, ServerApi, ServerApiVersion}, sync::Client
};

fn init_mongo() -> mongodb::error::Result<Client>
{
	println!("Connecting to server...");
	let uri = format!("mongodb+srv://{}:{}@cluster-01.myeybv2.mongodb.net/?retryWrites=true&w=majority", dotenv::var("DB_USERNAME").unwrap(), dotenv::var("DB_PASS").unwrap());
	let mut client_options = ClientOptions::parse(uri)?;
	// Set the server_api field of the client_options object to set the version of the Stable API on the client
	let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
	client_options.server_api = Some(server_api);
	// Get a handle to the cluster
	let client = Client::with_options(client_options)?;
	// Ping the server to see if you can connect to the cluster
	client.database("admin").run_command(doc! {"ping": 1}, None)?;
	println!("Connected to server!");
	Ok(client)
}

pub fn get_database(name: &str) -> Database { init_mongo().unwrap().database(name) }

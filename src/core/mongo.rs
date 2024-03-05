use mongodb::sync::Database;
use mongodb::{
    bson::doc, bson::Document, options::{ClientOptions, ServerApi, ServerApiVersion}, sync::Client, sync::Collection
};

fn init_mongo() -> mongodb::error::Result<Client>
{
    //println!("Connecting to server...");
    //let sw: Stopwatch = Stopwatch::start_new();
    let uri = format!(
        "mongodb+srv://{}:{}@cluster-01.myeybv2.mongodb.net/?retryWrites=true&w=majority",
        dotenv::var("DB_USERNAME").unwrap(),
        dotenv::var("DB_PASS").unwrap()
    );
    let mut client_options: ClientOptions = ClientOptions::parse(uri)?;
    // Set the server_api field of the client_options object to set the version of the Stable API on the client
    let server_api: ServerApi = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Get a handle to the cluster
    let client: Client = Client::with_options(client_options)?;
    // Ping the server to see if you can connect to the cluster
    client.database("admin").run_command(doc! {"ping": 1}, None)?;
    //println!("Connected to server! Connection took {}s.", (sw.elapsed_ms() as f64 / 1000.00).to_string());
    Ok(client)
}

pub fn get_database(name: &str) -> Database { init_mongo().unwrap().database(name) }
pub fn get_collection(name: &str) -> Collection<Document>
{
    get_database(dotenv::var("DB_NAME").unwrap().as_str()).collection::<Document>(name)
    // realistically this should be an option, but .collection doesn't return an option if it found the documents or not.
}

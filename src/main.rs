use mongodb::{
    Client,
    bson::{Bson, doc},
    error::ErrorKind,
    options::{ClientOptions, ReadConcern},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hosts = std::env::var("MONGODB_HOSTS")?;
    let hosts = hosts.split(',').map(str::trim).collect::<Vec<_>>();

    println!(
        "Initializing mongodb replica set with {} nodes",
        hosts.len()
    );
    println!("Connecting to the main client at: mongodb://{} ", hosts[0]);

    let client = Client::with_options(
        ClientOptions::builder()
            .hosts(vec![hosts[0].to_string().parse()?])
            .direct_connection(true)
            .build(),
    )?;
    let db = client.database("admin");

    println!("Checking if the client already initialize replica set");
    let result = db.run_command(doc! {"replSetGetStatus": 1}).await;
    if result.is_ok() {
        println!("Replica set is already initialized");
        return Ok(());
    }
    let err = result.err().unwrap();
    if let ErrorKind::Command(ref cmd) = *err.kind {
        if cmd.code != 94 {
            return Err(err.into());
        }
    }

    println!("Initializing replica set");

    let replicas = hosts
        .iter()
        .enumerate()
        .map(|(i, host)| doc! {"_id": Bson::Int32(i as i32), "host": host})
        .collect::<Vec<_>>();
    db.run_command(doc! {"replSetInitiate": {"_id": "rs0", "members": replicas}})
        .await?;
    println!("Replica set initialized");

    println!("Waiting for the replica set to be ready");
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let client = Client::with_options(
        ClientOptions::builder()
            .repl_set_name("rs0".to_string())
            .hosts(hosts.iter().map(|h| h.parse().unwrap()).collect::<Vec<_>>())
            .read_concern(ReadConcern::majority())
            .build(),
    )?;
    let db = client.database("admin");

    let status = db.run_command(doc! {"replSetGetStatus": 1}).await?;
    let status_json = serde_json::to_string_pretty(&status).unwrap();
    println!("Replica set status: {}", status_json);

    println!(
        "You can now connect to the replica set at: mongodb://{}/?replSet=rs0",
        hosts.join(",")
    );

    Ok(())
}

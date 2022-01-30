//! This example illustrates the way to put and fetch items with `database`.

// This is using the `tokio` runtime. You'll need the following dependency:
//
// `tokio = { version = "1", features = ["full"] }`

use deta_rust::{database, serde_json::json, DetaClient};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct SampleDbModel {
    // The `skip_serializing_if` attribute is useful because thanks to it, when `key` is empty, deta will generate it itself.
    #[serde(skip_serializing_if = "String::is_empty")]
    key: String,

    some_field: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // [place_your_project_key_here]
    let client = DetaClient::new("[place_your_project_key_here]");
    let database = database::Database::new(&client, "sample_db");

    // Put
    let items = vec![
        SampleDbModel {
            key: "a".into(),
            some_field: "Some value".into(),
        },
        SampleDbModel {
            key: "b".into(),
            some_field: "Another value 1".into(),
        },
        SampleDbModel {
            key: "c".into(),
            some_field: "Another value 2".into(),
        },
    ];

    let result = database.put_items(&items).await?;
    println!("PutItems<SampleDbModel>: {:#?}", result);

    // Fetch
    let query_result = database
        .fetch_items::<SampleDbModel>(None, None, Some(&[json!({ "some_field?pfx": "Another" })]))
        .await?;

    println!("FetchItems<SampleDbModel>: {:#?}", query_result);

    Ok(())
}
//! The following integration tests are designed to quickly make sure that SDK is compatible with the API.

use deta_rust::{
    database::{
        updates::{Action, Updates},
        Database,
    },
    serde_json::json,
    DetaClient,
};
use serde::{Deserialize, Serialize};
use serial_test::serial;

// ---------- CONFIG ----------

#[macro_use]
extern crate lazy_static;

fn config() -> Database {
    dotenv::dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY is not provided");
    let test_db_name = std::env::var("TEST_DB_NAME").expect("TEST_DB_NAME is not provided");
    let client = DetaClient::new(&api_key);
    Database::new(&client, &test_db_name)
}

const TEST_KEY: &'static str = "123";

lazy_static! {
    static ref DATABASE: Database = config();
}

// ---------- HELPERS ----------

/// Removes all creted items.
async fn clean() {
    let items = DATABASE
        .fetch_items::<SampleModel>(None, None, None)
        .await
        .expect("Fetch items went wrong during clean() performing");
    let keys: Vec<&String> = items.items.iter().map(|item| &item.key).collect();
    for key in keys {
        DATABASE
            .delete_item(key)
            .await
            .expect("Delete item went wrong during clean() performing");
    }
}

async fn setup_items() {
    clean().await;

    let items = [
        SampleModel {
            key: TEST_KEY.into(),
            sample_field: "field1_val".into(),
        },
        SampleModel {
            key: "".into(),
            sample_field: "another_field1_val".into(),
        },
    ];

    DATABASE.put_items(&items).await.unwrap();
}

// ---------- MODELS ----------

#[derive(Serialize, Deserialize, Debug)]
struct SampleModel {
    #[serde(skip_serializing_if = "String::is_empty")]
    key: String,

    sample_field: String,
}

// ---------- TESTS ----------

#[tokio::test]
#[serial]
async fn put_items() {
    setup_items().await;
    clean().await;
}

#[tokio::test]
#[serial]
async fn get_item_return_some() {
    setup_items().await;
    let res1 = DATABASE.get_item::<SampleModel>(TEST_KEY).await.unwrap();
    assert!(matches!(res1, Some(_)));
    clean().await;
}

#[tokio::test]
#[serial]
async fn get_item_return_none() {
    let res = DATABASE
        .get_item::<SampleModel>("nonexistent_key")
        .await
        .unwrap();
    assert!(matches!(res, None));
}

#[tokio::test]
#[serial]
async fn delete_existent_item() {
    setup_items().await;
    DATABASE.delete_item(TEST_KEY).await.unwrap();
    clean().await;
}

#[tokio::test]
#[serial]
async fn delete_non_existent_item() {
    DATABASE.delete_item("nonexistent_key").await.unwrap();
}

#[tokio::test]
#[serial]
async fn insert_item() {
    let item = SampleModel {
        key: "".into(),
        sample_field: "field_value".into(),
    };
    DATABASE.insert_item(&item).await.unwrap();
    clean().await;
}

#[tokio::test]
#[should_panic(expected = "Error occurred")]
#[serial]
async fn insert_item_with_existent_key() {
    setup_items().await;
    let item = SampleModel {
        key: TEST_KEY.into(),
        sample_field: "field_value".into(),
    };
    DATABASE.insert_item(&item).await.expect("Error occurred");
    clean().await;
}

#[tokio::test]
#[serial]
async fn fetch_items() {
    setup_items().await;
    DATABASE
        .fetch_items::<SampleModel>(None, None, None)
        .await
        .unwrap();
    clean().await;
}

#[tokio::test]
#[serial]
async fn fetch_items_with_query() {
    setup_items().await;
    let query = [json!({
        "sample_field": "field1_val"
    })];

    let res = DATABASE
        .fetch_items::<SampleModel>(None, None, Some(&query))
        .await
        .unwrap();

    assert_eq!(res.items.len(), 1);

    clean().await;
}

#[tokio::test]
#[serial]
async fn fetch_items_with_limit() {
    setup_items().await;
    DATABASE
        .fetch_items::<SampleModel>(Some(1), None, None)
        .await
        .unwrap();
    clean().await;
}

#[tokio::test]
#[serial]
async fn update_item() {
    setup_items().await;

    let updates = Updates::init().add("some_field", Action::set("some_value"));

    let update_result = DATABASE.update_item(TEST_KEY, updates).await.unwrap();
    let result_set_section = &update_result.set.expect("Set section is none");
    assert_eq!(result_set_section, &json!({ "some_field": "some_value" }));

    clean().await;
}

#[tokio::test]
#[should_panic(expected = "Error occurred")]
#[serial]
async fn update_nonexistent_item() {
    let updates = Updates::init().add("some_field", Action::set("some_value"));

    DATABASE
        .update_item("nonexistent_key", updates)
        .await
        .expect("Error occurred");
}

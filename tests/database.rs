//! The following integration tests are designed to quickly make sure that SDK is compatible with the API.

use deta_rust::{
    database::{
        models::FetchItems,
        query::{Condition, Query},
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
            some_field_2: 0,
        },
        SampleModel {
            key: "".into(),
            sample_field: "another_field1_val".into(),
            some_field_2: 10,
        },
        SampleModel {
            key: "".into(),
            sample_field: "yet_another_field1_val".into(),
            some_field_2: -10,
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
    some_field_2: i32,
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
        some_field_2: 0,
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
        some_field_2: 0,
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

    async fn make_fetch(query: Query) -> FetchItems<SampleModel> {
        DATABASE
            .fetch_items::<SampleModel>(None, None, Some(query))
            .await
            .unwrap()
    }

    // Test for several queries

    let query = Query::init().on("sample_field", Condition::equal("field1_val"));
    assert_eq!(make_fetch(query).await.items.len(), 1);

    let query = Query::init().on("some_field_2", Condition::equal(10));
    assert_eq!(make_fetch(query).await.items.len(), 1);

    let query = Query::init().on("sample_field", Condition::not_equal("field1_val"));
    assert_eq!(make_fetch(query).await.items.len(), 2);

    let query = Query::init().on("some_field_2", Condition::greater_than(9));
    assert_eq!(make_fetch(query).await.items.len(), 1);

    let query = Query::init().on("some_field_2", Condition::less_than(5));
    assert_eq!(make_fetch(query).await.items.len(), 2);

    let query = Query::init().on("some_field_2", Condition::greater_than_or_equal(10));
    assert_eq!(make_fetch(query).await.items.len(), 1);

    let query = Query::init().on("some_field_2", Condition::less_than_or_equal(0));
    assert_eq!(make_fetch(query).await.items.len(), 2);

    let query = Query::init().on("sample_field", Condition::prefix("another"));
    assert_eq!(make_fetch(query).await.items.len(), 1);

    let query = Query::init().on("some_field_2", Condition::range(-10, 0));
    assert_eq!(make_fetch(query).await.items.len(), 2);

    let query = Query::init().on("sample_field", Condition::contains("yet"));
    assert_eq!(make_fetch(query).await.items.len(), 1);

    let query = Query::init().on("sample_field", Condition::not_contains("yet"));
    assert_eq!(make_fetch(query).await.items.len(), 2);

    let query = Query::init()
        .on("sample_field", Condition::not_contains("yet"))
        .either()
        .on("some_field_2", Condition::greater_than(-100));

    assert_eq!(make_fetch(query).await.items.len(), 3);

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

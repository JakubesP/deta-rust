//! The following integration tests are designed to quickly make sure that SDK is compatible with the API.

use deta_rust::{drive::Drive, DetaClient};
use serial_test::serial;

// ---------- CONFIG ----------

#[macro_use]
extern crate lazy_static;

fn config() -> Drive {
    dotenv::dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY is not provided");
    let test_drive_name =
        std::env::var("TEST_DRIVE_NAME").expect("TEST_DRIVE_NAME is not provided");
    let client = DetaClient::new(&api_key);
    Drive::new(&client, &test_drive_name)
}

const FILE_NAME_1: &'static str = "test_file.txt";
const FILE_NAME_2: &'static str = "another_test_file.txt";

lazy_static! {
    static ref DRIVE: Drive = config();
}

// ---------- HELPERS ----------

/// Removes all uploaded files.
async fn clean() {
    let files = DRIVE
        .list_files(None, None, None)
        .await
        .expect("List files went wrong during clean() performing");

    if files.names.len() > 0 {
        DRIVE
            .delete_files(&files.names)
            .await
            .expect("Delete files went wrong during clean() performing");
    }
}

async fn setup_files() {
    clean().await;
    let data_1 = b"data_1".to_vec();
    let data_2 = b"data_2".to_vec();

    DRIVE
        .put_file(FILE_NAME_1, data_1, None)
        .await
        .expect("Put file went wrong during setup_file() performing");

    DRIVE
        .put_file(FILE_NAME_2, data_2, None)
        .await
        .expect("Put file went wrong during setup_file() performing");
}

// ---------- TESTS ----------

#[tokio::test]
#[serial]
async fn put_file() {
    setup_files().await;
    clean().await;
}

#[tokio::test]
#[serial]
async fn put_file_greater_than_10_mb() {
    let data = vec![0u8; (1024 * 1024 * 10) + (1024 * 1024)]; // 11MB of data
    DRIVE.put_file("big_file.dat", data, None).await.unwrap();
    clean().await;
}

#[tokio::test]
#[serial]
async fn get_file() {
    setup_files().await;
    // Testing get_file_as_u8_vec apply also to get_file_as_buffer
    let data = DRIVE.get_file_as_u8_vec(FILE_NAME_1).await.unwrap();
    assert!(matches!(data, Some(_)));
    clean().await;
}

#[tokio::test]
#[serial]
async fn get_nonexistent_file() {
    // Testing get_file_as_u8_vec apply also to get_file_as_buffer
    let data = DRIVE
        .get_file_as_u8_vec("nonexistent_file.txt")
        .await
        .unwrap();
    assert!(matches!(data, None));
    clean().await;
}

#[tokio::test]
#[serial]
async fn list_files() {
    setup_files().await;
    DRIVE.list_files(None, None, None).await.unwrap();
    clean().await;
}

#[tokio::test]
#[serial]
async fn list_files_with_limit() {
    setup_files().await;
    let files = DRIVE.list_files(Some(1), None, None).await.unwrap();
    assert_eq!(files.names.len(), 1);
    clean().await;
}

#[tokio::test]
#[serial]
async fn list_files_with_prefix() {
    setup_files().await;
    let files = DRIVE.list_files(None, Some("another"), None).await.unwrap();
    assert_eq!(files.names.len(), 1);
    clean().await;
}

#[tokio::test]
#[serial]
async fn list_files_from_last_name() {
    setup_files().await;

    let files = DRIVE.list_files(Some(1), None, None).await.unwrap();
    assert_eq!(files.names.len(), 1);

    let first_last = &files.paging.unwrap().last.unwrap();

    let next_files = DRIVE
        .list_files(Some(1), None, Some(first_last))
        .await
        .unwrap();

    assert!(matches!(next_files.paging, None));

    clean().await;
}

#[tokio::test]
#[serial]
async fn delete_files() {
    setup_files().await;
    let files_to_delete: Vec<String> = vec![FILE_NAME_1.into(), "nonexistent_file.txt".into()];
    DRIVE.delete_files(&files_to_delete).await.unwrap();
    let files_after_delete = DRIVE.list_files(None, None, None).await.unwrap();
    assert_eq!(files_after_delete.names.len(), 1);
    clean().await;
}

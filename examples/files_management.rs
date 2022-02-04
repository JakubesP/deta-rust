//! This example illustrates the way to upload and download file with `drive`.

// This is using the `tokio` runtime. You'll need the following dependency:
//
// `tokio = { version = "1", features = ["full"] }`

use deta_rust::{drive, DetaClient};
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, io::AsyncReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = DetaClient::new("[place_your_project_key_here]");
    let drive = drive::Drive::new(&client, "sample_drive");

    // Upload file
    let mut file = File::open("some_file.jpg").await?;
    let mut file_content = vec![];
    file.read_to_end(&mut file_content).await?;
    let result = drive.put_file("some_file.jpg", file_content, None).await?;
    println!("PutFileResult: {:#?}", result);

    // Download file
    let mut file = File::create("some_file_2.jpg").await?;
    let downloaded_data = drive
        .get_file_as_u8_vec("some_file.jpg")
        .await?
        .expect("File not found");
    file.write_all(&downloaded_data).await?;

    Ok(())
}

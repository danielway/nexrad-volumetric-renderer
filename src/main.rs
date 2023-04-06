use std::fs::{File, read_dir};
use std::io::Write;
use crate::aws::get_client;

use crate::noaa::{list_nexrad, download_nexrad};

mod result;
mod aws;
mod noaa;

#[tokio::main]
async fn main() {
    let client = get_client().await;

    println!("Fetching latest data...");
    let chunk_keys = list_nexrad(&client, "2023", "04", "06", "KDMX")
        .await
        .expect("should fetch latest data");

    println!("Found {} chunks", chunk_keys.len());

    _ = std::fs::create_dir("chunks");

    let paths = read_dir("chunks").expect("can read chunk directory");
    let file_names: Vec<String> = paths.map(|path| path.unwrap().file_name().to_str().unwrap().to_string()).collect();

    println!("Found {} existing chunks", file_names.len());

    for chunk_key in chunk_keys {
        let safe_chunk_key = chunk_key.replace("/", "_");
        if !file_names.contains(&safe_chunk_key) {
            println!("Downloading new chunk {}", chunk_key);
            let chunk = download_nexrad(&client, &chunk_key).await.expect("can download chunk");

            println!("Writing chunk to file {}...", safe_chunk_key);
            let mut file = File::create(format!("chunks/{}", safe_chunk_key)).expect("can create file");
            file.write_all(&chunk).expect("can write to file");
        }
    }

    println!("Done! ({} chunks)", file_names.len());
}

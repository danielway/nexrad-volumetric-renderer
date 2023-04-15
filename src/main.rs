use std::fs::{create_dir, File, read_dir};
use std::io;
use std::io::Write;

use crate::aws::get_client;
use crate::nexrad::read_nexrad_file;
use crate::noaa::{download_nexrad, list_nexrad};
use crate::result::Result;

mod result;
mod aws;
mod noaa;
mod nexrad;

#[tokio::main]
async fn main() {
    execute(
        "KDMX",
        ["2023", "04", "06"],
        false,
        true,
    ).await.expect("executes successfully");
}

async fn execute(site: &str, date: [&str; 3], fetch_chunks: bool, process_arbitrary: bool) -> Result<()> {
    println!("Executing for site {} and date {}/{}/{}", site, date[0], date[1], date[2]);
    println!("  Fetching chunks: {}", fetch_chunks);
    println!("  Processing arbitrary: {}", process_arbitrary);

    let cached_chunks_path = get_cached_chunks_path(site, date);
    let cached_chunk_keys = get_cached_chunks_filenames(&cached_chunks_path)?.unwrap_or_else(|| {
        create_dir(&cached_chunks_path).expect("can create chunks cache directory");
        Vec::new()
    });
    println!("Found {} cached chunks", cached_chunk_keys.len());

    if fetch_chunks {
        let client = get_client().await;

        println!("Fetching latest data...");
        let chunk_keys = list_nexrad(&client, date[0], date[1], date[2], site).await?;
        println!("  Fetched {} chunks", chunk_keys.len());

        for chunk_key in chunk_keys {
            let safe_chunk_key = chunk_key.replace("/", "_");
            if !cached_chunk_keys.contains(&safe_chunk_key) {
                println!("  Downloading new chunk {}", chunk_key);
                let chunk = download_nexrad(&client, &chunk_key).await?;

                println!("    Writing chunk to file {}...", safe_chunk_key);
                let mut file = File::create(format!("chunks/{}", safe_chunk_key))?;
                file.write_all(&chunk)?;
            }
        }
    }

    if process_arbitrary {
        read_nexrad_file(&format!("{}/{}", cached_chunks_path, cached_chunk_keys[0]))?;
    }

    Ok(())
}

fn get_cached_chunks_path(site: &str, date: [&str; 3]) -> String {
    format!("chunks/{}_{}_{}_{}", site, date[0], date[1], date[2])
}

fn get_cached_chunks_filenames(path: &str) -> Result<Option<Vec<String>>> {
    let paths = read_dir(path);
    if let Err(err) = paths {
        return if err.kind() == io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(err.into())
        };
    }

    let path_filenames = paths?.map(
        |path| path.unwrap().file_name().to_str().unwrap().to_string()
    );

    Ok(Some(path_filenames.collect()))
}

use aws_sdk_s3::config::Region;

mod aws;
use crate::aws::{get_client, list_objects};

#[tokio::main]
async fn main() {
    let client = get_client().await;
    let objects = list_objects(client, "unidata-nexrad-level2-chunks")
        .await
        .expect("should fetch latest data")
        .expect("should have objects");

    for obj in objects {
        println!("{}", obj.key().unwrap_or_default());
    }
}

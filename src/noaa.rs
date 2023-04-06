use aws_sdk_s3::Client;

use crate::aws::{get_object, list_objects};
use crate::result::Result;

const BUCKET: &str = "noaa-nexrad-level2";

pub async fn download_nexrad(
    client: &Client,
    key: &str,
) -> Result<Vec<u8>> {
    Ok(get_object(client, BUCKET, key)
        .await.expect("should fetch object"))
}

pub async fn list_nexrad(
    client: &Client,
    year: &str,
    month: &str,
    day: &str,
    site: &str,
) -> Result<Vec<String>> {
    let prefix = format!("{}/{}/{}/{}", year, month, day, site);

    let objects = list_objects(client, BUCKET, &prefix)
        .await?.expect("should return objects");

    Ok(objects
        .iter()
        .map(|obj| obj.key().expect("should have a key").to_string())
        .collect())
}

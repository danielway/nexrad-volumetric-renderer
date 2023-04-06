use aws_config::from_env;
use aws_sdk_s3::{config::Region, types::Object, Client};

pub async fn list_nexrad(year: &str, month: &str, day: &str, site: &str) -> Result<Vec<String>, aws_sdk_s3::Error> {
    let prefix = format!("{}/{}/{}/{}", year, month, day, site);

    let client = get_client().await;
    let objects = list_objects(client, "noaa-nexrad-level2", &prefix)
        .await
        .expect("should fetch latest data")
        .expect("should have objects");

    Ok(objects
        .iter()
        .map(|obj| {
            let mut result = String::new();
            result.push_str(obj.key().unwrap());
            result.push_str(" - ");
            result.push_str(&obj.last_modified().unwrap().to_millis().unwrap().to_string());

            return result;
        })
        .collect())
}

pub async fn list_objects(
    client: Client,
    bucket: &str,
    prefix: &str,
) -> Result<Option<Vec<Object>>, aws_sdk_s3::Error> {
    Ok(client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .customize()
        .await
        .unwrap()
        .map_operation(make_unsigned)
        .unwrap()
        .send()
        .await?
        .contents()
        .map(|objects| objects.to_vec()))
}

pub async fn get_client() -> Client {
    let conf = from_env().region(Region::new("us-east-1")).load().await;
    Client::new(&conf)
}

// Disables signing requirements for unauthenticated S3 requests.
fn make_unsigned<O, Retry>(
    mut operation: aws_smithy_http::operation::Operation<O, Retry>,
) -> Result<aws_smithy_http::operation::Operation<O, Retry>, std::convert::Infallible> {
    {
        let mut props = operation.properties_mut();
        let mut signing_config = props
            .get_mut::<aws_sig_auth::signer::OperationSigningConfig>()
            .expect("has signing_config");
        signing_config.signing_requirements = aws_sig_auth::signer::SigningRequirements::Disabled;
    }

    Ok(operation)
}

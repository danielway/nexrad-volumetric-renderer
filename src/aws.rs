use aws_config::from_env;
use aws_sdk_s3::{Client, config::Region, types::Object};

use crate::result::Result;

pub async fn get_object(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>> {
    let builder = client
        .get_object()
        .bucket(bucket)
        .key(key);

    let operation = builder.customize().await?.map_operation(make_unsigned).unwrap();

    let response = operation.send().await?;
    let bytes = response.body.collect().await.unwrap();

    Ok(bytes.to_vec())
}

pub async fn list_objects(
    client: &Client,
    bucket: &str,
    prefix: &str,
) -> Result<Option<Vec<Object>>> {
    let builder = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix);

    let operation = builder.customize().await?.map_operation(make_unsigned).unwrap();
    let response = operation.send().await?;

    Ok(response.contents().map(|objects| objects.to_vec()))
}

pub async fn get_client() -> Client {
    let conf = from_env().region(Region::new("us-east-1")).load().await;
    Client::new(&conf)
}

// Disables signing requirements for unauthenticated S3 requests.
fn make_unsigned<O, Retry>(
    mut operation: aws_smithy_http::operation::Operation<O, Retry>,
) -> std::result::Result<aws_smithy_http::operation::Operation<O, Retry>, std::convert::Infallible> {
    {
        let mut props = operation.properties_mut();
        let mut signing_config = props
            .get_mut::<aws_sig_auth::signer::OperationSigningConfig>()
            .expect("has signing_config");
        signing_config.signing_requirements = aws_sig_auth::signer::SigningRequirements::Disabled;
    }

    Ok(operation)
}

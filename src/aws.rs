use aws_config::from_env;
use aws_sdk_s3::{config::Region, types::Object, Client};

pub async fn get_client() -> Client {
    let conf = from_env().region(Region::new("us-east-1")).load().await;
    Client::new(&conf)
}

pub async fn list_objects(
    client: Client,
    bucket: &str,
) -> Result<Option<Vec<Object>>, aws_sdk_s3::Error> {
    Ok(client
        .list_objects_v2()
        .bucket(bucket)
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

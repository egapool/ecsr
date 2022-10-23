https://www.serverlessguru.com/blog/aws-sdk-for-rust-getting-started

```rust
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_sdk_s3::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The name of the custom credentials profile you want to load
    let profile_name = "my-custom-profile";

    // This credentials provider will load credentials from ~/.aws/credentials.
    let credentials_provider = ProfileFileCredentialsProvider::builder()
        .profile_name(profile_name)
        .build();
    
    // Load the credentials
    let config = aws_config::from_env()
        .credentials_provider(credentials_provider)
        .load()
        .await;

    // Create an S3 client
    let s3 = Client::new(&config);

    // List the first page of buckets in the account
    let response = s3.list_buckets().send().await?;

    // Check if the response returned any buckets
    if let Some(buckets) = response.buckets() {
        // Print each bucket name out
        for bucket in buckets {
            println!("bucket name: {}", bucket.name().unwrap());
        }
    } else {
        println!("You don't have any buckets!");
    }

    Ok(())
}
`````````

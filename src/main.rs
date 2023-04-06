mod aws;
use crate::aws::{list_nexrad};

#[tokio::main]
async fn main() {
    let objects = list_nexrad("2023", "04", "06", "KDMX")
        .await
        .expect("should fetch latest data");

    for obj in objects {
        println!("{}", obj);
    }
}

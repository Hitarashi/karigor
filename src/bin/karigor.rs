//! Standalone Karigor CLI binary.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    druto_karigor::run().await
}

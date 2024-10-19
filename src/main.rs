fn main() -> Result<(), std::io::Error> {
    let provider = s3_entities::test::storage_provider::MockStorageProvider::new();
    s3_api::main(provider)
}

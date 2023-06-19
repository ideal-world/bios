use tardis::{
    basic::result::TardisResult, search::search_client::TardisSearchClient,
};

pub async fn init_index(client: &TardisSearchClient, tag: &str) -> TardisResult<()> {
    if client.check_index_exist(tag).await? {
        return Ok(());
    } else {
        client.create_index(tag).await?
    }
    Ok(())
}

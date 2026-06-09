#[allow(async_fn_in_trait)]
pub trait SyncProvider {
    async fn pull(&self) -> Result<(), String>;
    async fn push(&self) -> Result<(), String>;
}

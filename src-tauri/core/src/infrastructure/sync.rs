use crate::domain::sync::SyncProvider;

pub struct CloudKitProvider;
pub struct OneDriveProvider;
pub struct FolderProvider;
pub struct GitProvider;

async fn unsupported(name: &str) -> Result<(), String> {
    Err(format!("{name} is not implemented yet"))
}

impl SyncProvider for CloudKitProvider {
    async fn pull(&self) -> Result<(), String> { unsupported("CloudKitProvider").await }
    async fn push(&self) -> Result<(), String> { unsupported("CloudKitProvider").await }
}

impl SyncProvider for OneDriveProvider {
    async fn pull(&self) -> Result<(), String> { unsupported("OneDriveProvider").await }
    async fn push(&self) -> Result<(), String> { unsupported("OneDriveProvider").await }
}

impl SyncProvider for FolderProvider {
    async fn pull(&self) -> Result<(), String> { unsupported("FolderProvider").await }
    async fn push(&self) -> Result<(), String> { unsupported("FolderProvider").await }
}

impl SyncProvider for GitProvider {
    async fn pull(&self) -> Result<(), String> { unsupported("GitProvider").await }
    async fn push(&self) -> Result<(), String> { unsupported("GitProvider").await }
}

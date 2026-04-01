#[derive(Debug, Clone, Copy)]
pub enum ProviderMode {
    LocalOffline,
    OnlineApi,
    Hybrid,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub capability: &'static str,
    pub mode: ProviderMode,
    pub requires_setup: bool,
    pub note: &'static str,
}

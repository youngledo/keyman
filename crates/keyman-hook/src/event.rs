use crate::key::VirtualKey;

#[derive(Debug, Clone)]
pub struct RawKeyEvent {
    pub key: VirtualKey,
    pub pressed: bool,
}

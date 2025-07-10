use std::collections::HashMap;

use crate::{entry::Entry, error::CascError, root_handlers::tvfs_root_handler::TVFSRootHandler};

#[derive(Debug)]
pub enum RootHandler {
    TVFS(TVFSRootHandler),
    // MNDX
    // Diablo3
    // WoW
    // Overwatch
    // Starcraft1
}
pub trait RootHandlerTrait {
    fn get_file_entries(&self) -> Result<&HashMap<String, Entry>, CascError>;
}
impl RootHandlerTrait for RootHandler {
    fn get_file_entries(&self) -> Result<&HashMap<String, Entry>, CascError> {
        let file_entries = match self {
            RootHandler::TVFS(handler) => &handler.file_entries,
            _ => return Err(CascError::InvalidData("".to_string())),
        };
        Ok(file_entries)
    }
}

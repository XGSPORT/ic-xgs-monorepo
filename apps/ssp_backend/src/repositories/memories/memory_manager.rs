use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

pub(super) type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub(super) const SALT_MEMORY_ID: MemoryId = MemoryId::new(0);
pub(super) const USERS_MEMORY_ID: MemoryId = MemoryId::new(1);
pub(super) const USER_SUB_INDEX_MEMORY_ID: MemoryId = MemoryId::new(2);
pub(super) const USER_DB_ID_INDEX_MEMORY_ID: MemoryId = MemoryId::new(3);
pub(super) const CONFIG_MEMORY_ID: MemoryId = MemoryId::new(4);
pub(super) const CERTIFICATE_MEMORY_ID: MemoryId = MemoryId::new(5);
pub(super) const CERTIFICATE_USER_PRINCIPAL_INDEX_MEMORY_ID: MemoryId = MemoryId::new(6);
pub(super) const CERTIFICATE_MANAGED_USER_ID_INDEX_MEMORY_ID: MemoryId = MemoryId::new(7);

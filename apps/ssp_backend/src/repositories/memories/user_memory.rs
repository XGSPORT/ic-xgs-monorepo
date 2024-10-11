use ic_stable_structures::BTreeMap;

use crate::repositories::{User, UserDbId, UserPrincipal, UserSub};

use super::{
    Memory, MEMORY_MANAGER, USERS_MEMORY_ID, USER_DB_ID_INDEX_MEMORY_ID, USER_SUB_INDEX_MEMORY_ID,
};

pub type UserMemory = BTreeMap<UserPrincipal, User, Memory>;
pub type UserSubIndexMemory = BTreeMap<UserSub, UserPrincipal, Memory>;
pub type UserDbIdIndexMemory = BTreeMap<UserDbId, UserPrincipal, Memory>;

pub fn init_users() -> UserMemory {
    UserMemory::init(get_users_memory())
}

pub fn init_user_sub_index() -> UserSubIndexMemory {
    UserSubIndexMemory::init(get_user_sub_index_memory())
}

pub fn init_user_db_id_index() -> UserDbIdIndexMemory {
    UserDbIdIndexMemory::init(get_user_db_id_index_memory())
}

fn get_users_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(USERS_MEMORY_ID))
}

fn get_user_sub_index_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(USER_SUB_INDEX_MEMORY_ID))
}

fn get_user_db_id_index_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(USER_DB_ID_INDEX_MEMORY_ID))
}

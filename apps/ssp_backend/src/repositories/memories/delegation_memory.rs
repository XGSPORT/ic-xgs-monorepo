use ic_stable_structures::Cell;

use crate::repositories::{Salt, EMPTY_SALT};

use super::{Memory, MEMORY_MANAGER, SALT_MEMORY_ID};

pub type SaltMemory = Cell<Salt, Memory>;

pub fn init_salt() -> SaltMemory {
    SaltMemory::init(get_salt_memory(), EMPTY_SALT).unwrap()
}

fn get_salt_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(SALT_MEMORY_ID))
}

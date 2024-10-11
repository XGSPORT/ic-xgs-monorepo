use std::cell::RefCell;

use super::{
    init_user_db_id_index, init_user_sub_index, init_users, User, UserDbId, UserDbIdIndexMemory,
    UserMemory, UserPrincipal, UserSub, UserSubIndexMemory,
};

pub struct UserState {
    users: UserMemory,
    user_sub_index: UserSubIndexMemory,
    user_db_id_index: UserDbIdIndexMemory,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            users: init_users(),
            user_sub_index: init_user_sub_index(),
            user_db_id_index: init_user_db_id_index(),
        }
    }
}

thread_local! {
    static STATE: RefCell<UserState> = RefCell::new(UserState::default());
}

#[derive(Default)]
pub struct UserRepository {}

impl UserRepository {
    pub fn get_user_by_principal(&self, user_principal: &UserPrincipal) -> Option<User> {
        STATE.with_borrow(|s| s.users.get(user_principal))
    }

    pub fn get_user_by_sub(&self, user_sub: &UserSub) -> Option<(UserPrincipal, User)> {
        STATE.with_borrow(|s| {
            s.user_sub_index.get(user_sub).and_then(|user_principal| {
                s.users
                    .get(&user_principal)
                    .map(|user| (user_principal, user))
            })
        })
    }

    pub fn get_user_by_db_id(&self, db_id: &UserDbId) -> Option<(UserPrincipal, User)> {
        STATE.with_borrow(|s| {
            s.user_db_id_index.get(db_id).and_then(|user_principal| {
                s.users
                    .get(&user_principal)
                    .map(|user| (user_principal, user))
            })
        })
    }

    pub fn create_user(&self, user_principal: UserPrincipal, user: User) -> Result<(), String> {
        let user_sub = user.jwt_sub.clone();
        let db_id = user.db_id;

        if self.get_user_by_sub(&user_sub).is_some() {
            return Err(format!("User with sub {} already exists", user_sub));
        }

        if self.get_user_by_db_id(&db_id).is_some() {
            return Err(format!(
                "User with database id {} already exists",
                db_id.to_string()
            ));
        }

        STATE.with_borrow_mut(|s| {
            s.users.insert(user_principal, user);
            s.user_sub_index.insert(user_sub, user_principal);
            s.user_db_id_index.insert(db_id, user_principal);
        });

        Ok(())
    }
}

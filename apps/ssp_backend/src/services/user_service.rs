use crate::repositories::{User, UserPrincipal, UserRepository};

#[derive(Default)]
pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn get_user(&self, user_principal: &UserPrincipal) -> Option<User> {
        self.user_repository.get_user_by_principal(user_principal)
    }
}

use candid::Principal;
use ic_cdk::{caller, query, trap};

use crate::services::UserService;

#[query]
fn get_my_user() -> ssp_backend_types::User {
    let calling_principal = caller();

    UserController::default().get_my_user(calling_principal)
}

#[derive(Default)]
pub struct UserController {
    user_service: UserService,
}

impl UserController {
    fn get_my_user(&self, calling_principal: Principal) -> ssp_backend_types::User {
        match self.user_service.get_user(&calling_principal) {
            Some(user) => user.into(),
            None => trap("No user found"),
        }
    }
}

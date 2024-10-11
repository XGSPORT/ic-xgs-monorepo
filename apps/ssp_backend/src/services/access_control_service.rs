use candid::Principal;
use ic_cdk::api::is_controller;

use crate::repositories::{ConfigRepository, UserRepository};

#[derive(Default)]
pub struct AccessControlService {
    user_repository: UserRepository,
    config_repository: ConfigRepository,
}

impl AccessControlService {
    pub fn assert_principal_is_controller(
        &self,
        calling_principal: &Principal,
    ) -> Result<(), String> {
        if !is_controller(calling_principal) {
            return Err("Caller is not a controller".to_string());
        }

        Ok(())
    }

    pub fn assert_principal_is_user(&self, calling_principal: &Principal) -> Result<(), String> {
        if self
            .user_repository
            .get_user_by_principal(calling_principal)
            .is_none()
        {
            return Err(format!(
                "Caller {} is not a user",
                calling_principal.to_text()
            ));
        }

        Ok(())
    }

    pub fn assert_principal_is_backend(&self, calling_principal: &Principal) -> Result<(), String> {
        self.config_repository
            .get_config()
            .backend_principal
            .ok_or_else(|| "Backend principal not set".to_string())
            .and_then(|backend_principal| {
                if calling_principal.ne(&backend_principal) {
                    return Err("Caller is not the backend".to_string());
                }

                Ok(())
            })
    }

    pub fn assert_principal_is_user_or_backend<'a>(
        &self,
        calling_principal: &'a Principal,
    ) -> Result<Option<&'a Principal>, String> {
        match self.assert_principal_is_backend(calling_principal) {
            Ok(_) => Ok(None),
            Err(_) => {
                self.assert_principal_is_user(calling_principal)
                    .map_err(|_| "Caller is not the backend or a registered user".to_string())?;
                Ok(Some(calling_principal))
            }
        }
    }
}

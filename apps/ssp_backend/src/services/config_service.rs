use candid::Principal;

use crate::repositories::{Config, ConfigRepository};

#[derive(Default)]
pub struct ConfigService {
    config_repository: ConfigRepository,
}

impl ConfigService {
    pub fn set_backend_principal(&self, backend_principal: Principal) -> Result<(), String> {
        if backend_principal == Principal::anonymous() {
            return Err("Backend principal cannot be anonymous".to_string());
        }

        let mut config = self.config_repository.get_config();

        config.backend_principal = Some(backend_principal);

        self.config_repository.set_config(config)
    }

    pub fn get_config(&self) -> Config {
        self.config_repository.get_config()
    }
}

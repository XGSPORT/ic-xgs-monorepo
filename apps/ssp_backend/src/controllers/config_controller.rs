use candid::Principal;
use ic_cdk::{caller, query, update};

use crate::services::{AccessControlService, ConfigService};

#[update]
fn set_backend_principal(principal: Principal) {
    let calling_principal = caller();

    ConfigController::default().set_backend_principal(calling_principal, principal);
}

#[query]
fn get_config() -> ssp_backend_types::Config {
    let calling_principal = caller();

    ConfigController::default().get_config(calling_principal)
}

#[derive(Default)]
struct ConfigController {
    access_control_service: AccessControlService,
    config_service: ConfigService,
}

impl ConfigController {
    fn set_backend_principal(&self, calling_principal: Principal, principal: Principal) {
        self.access_control_service
            .assert_principal_is_controller(&calling_principal)
            .unwrap();

        self.config_service
            .set_backend_principal(principal)
            .unwrap()
    }

    fn get_config(&self, calling_principal: Principal) -> ssp_backend_types::Config {
        self.access_control_service
            .assert_principal_is_controller(&calling_principal)
            .unwrap();

        self.config_service.get_config().into()
    }
}

use std::time::Duration;

use ic_cdk::{init, post_upgrade, spawn, trap};
use ic_cdk_timers::set_timer;

use crate::services::{CertificateService, DelegationService};

#[init]
fn init() {
    set_timer(Duration::ZERO, move || spawn(init_task()));

    jobs::start_jobs();
}

#[post_upgrade]
fn post_upgrade() {
    set_timer(Duration::ZERO, move || spawn(init_task()));

    jobs::start_jobs();
}

async fn init_task() {
    let init_controller = InitController::default();
    // We first certify all certificates, then we run the https outcalls.
    // If we invert the order, this fails in the tests with PocketIC.
    init_controller.certify_all_certificates();
    init_controller.init_delegation().await;
}

#[derive(Default)]
struct InitController {
    delegation_service: DelegationService,
    certificate_service: CertificateService,
}

impl InitController {
    async fn init_delegation(&self) {
        self.delegation_service.ensure_salt_initialized().await;

        self.fetch_jwks().await;
    }

    async fn fetch_jwks(&self) {
        if let Err(e) = self.delegation_service.fetch_and_store_jwks().await {
            trap(&format!("failed to fetch and store jwks: {e}"));
        }
    }

    fn certify_all_certificates(&self) {
        self.certificate_service.certify_all_certificates();
    }
}

mod jobs {
    use ic_cdk::spawn;
    use ic_cdk_timers::set_timer_interval;
    use std::time::Duration;

    pub fn start_jobs() {
        delegation::start();
    }

    mod delegation {
        use super::*;

        use crate::controllers::init_controller::InitController;

        // fetch JWKS every 1 hour
        const JWKS_FETCH_INTERVAL: Duration = Duration::from_secs(60 * 60);

        pub fn start() {
            set_timer_interval(JWKS_FETCH_INTERVAL, || {
                spawn(fetch_jwks());
            });
        }

        async fn fetch_jwks() {
            InitController::default().fetch_jwks().await
        }
    }
}

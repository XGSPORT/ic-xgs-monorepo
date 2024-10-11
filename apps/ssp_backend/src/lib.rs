mod controllers;
mod mappings;
mod repositories;
mod services;
mod system_api;
mod utils;

use candid::{export_service, Principal};
use ic_cdk::query;
use ssp_backend_types::*;

// In the following, we register a custom getrandom implementation because
// otherwise getrandom (which is a dependency of some packages) fails to compile.
// This is necessary because getrandom by default fails to compile for the
// wasm32-unknown-unknown target (which is required for deploying a canister).
getrandom::register_custom_getrandom!(always_fail);
pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}

export_service!();
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid_parser::utils::{service_compatible, CandidSource};
    use std::path::Path;

    #[test]
    fn check_candid_interface() {
        let new_interface = __export_service();

        service_compatible(
            CandidSource::Text(&new_interface),
            CandidSource::File(Path::new("./ssp_backend.did")),
        )
        .unwrap();
    }
}

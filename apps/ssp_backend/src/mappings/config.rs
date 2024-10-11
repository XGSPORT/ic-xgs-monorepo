use crate::repositories::Config;

impl From<Config> for ssp_backend_types::Config {
    fn from(value: Config) -> Self {
        Self {
            backend_principal: value.backend_principal,
        }
    }
}

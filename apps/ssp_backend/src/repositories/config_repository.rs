use std::cell::RefCell;

use super::{init_config, Config, ConfigMemory};

struct ConfigState {
    config: ConfigMemory,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            config: init_config(),
        }
    }
}

thread_local! {
    static STATE: RefCell<ConfigState> = RefCell::new(ConfigState::default());
}

#[derive(Default)]
pub struct ConfigRepository {}

impl ConfigRepository {
    pub fn get_config(&self) -> Config {
        STATE.with_borrow(|s| s.config.get().clone())
    }

    pub fn set_config(&self, config: Config) -> Result<(), String> {
        STATE.with_borrow_mut(|s| {
            s.config
                .set(config)
                .map_err(|err| format!("Cannot set config: {:?}", err))
        })?;

        Ok(())
    }
}

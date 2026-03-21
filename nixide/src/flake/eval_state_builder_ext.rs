use super::FlakeSettings;
use crate::{EvalStateBuilder, NixideError};

pub trait EvalStateBuilderExt {
    /// Configures the eval state to provide flakes features such as `builtins.getFlake`.
    fn flakes(self, settings: &FlakeSettings) -> Result<EvalStateBuilder, NixideError>;
}

impl EvalStateBuilderExt for EvalStateBuilder {
    /// Configures the eval state to provide flakes features such as `builtins.getFlake`.
    fn flakes(mut self, settings: &FlakeSettings) -> Result<EvalStateBuilder, NixideError> {
        settings.add_to_eval_state_builder(&mut self).map(|_| self)
    }
}

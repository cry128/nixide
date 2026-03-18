use super::FlakeSettings;
use crate::{EvalStateBuilder, NixErrorCode};

pub trait EvalStateBuilderExt {
    /// Configures the eval state to provide flakes features such as `builtins.getFlake`.
    fn flakes(self, settings: &FlakeSettings) -> Result<EvalStateBuilder, NixErrorCode>;
}

impl EvalStateBuilderExt for EvalStateBuilder {
    /// Configures the eval state to provide flakes features such as `builtins.getFlake`.
    fn flakes(mut self, settings: &FlakeSettings) -> Result<EvalStateBuilder, NixErrorCode> {
        settings.add_to_eval_state_builder(&mut self).map(|_| self)
    }
}

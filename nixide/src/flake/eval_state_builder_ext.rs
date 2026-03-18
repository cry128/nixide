pub trait EvalStateBuilderExt {
    /// Configures the eval state to provide flakes features such as `builtins.getFlake`.
    fn flakes(
        self,
        settings: &FlakeSettings,
    ) -> Result<nix_bindings_expr::eval_state::EvalStateBuilder>;
}
impl EvalStateBuilderExt for nix_bindings_expr::eval_state::EvalStateBuilder {
    /// Configures the eval state to provide flakes features such as `builtins.getFlake`.
    fn flakes(
        mut self,
        settings: &FlakeSettings,
    ) -> Result<nix_bindings_expr::eval_state::EvalStateBuilder> {
        settings.add_to_eval_state_builder(&mut self)?;
        Ok(self)
    }
}

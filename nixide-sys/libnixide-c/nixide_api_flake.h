#ifndef NIXIDE_API_FLAKE_H
#define NIXIDE_API_FLAKE_H

#include "nix_api_flake.h"

#ifdef __cplusplus
extern "C" {
#endif

nix_err nix_flake_lock_flags_set_recreate_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_update_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_write_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_fail_on_unlocked(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_use_registries(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_apply_nix_config(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_allow_unlocked(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err nix_flake_lock_flags_set_commit_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value);

nix_err
nix_flake_lock_flags_set_reference_lock_file_path(nix_c_context * context, nix_flake_lock_flags * flags, char * path);

nix_err
nix_flake_lock_flags_set_output_lock_file_path(nix_c_context * context, nix_flake_lock_flags * flags, char * path);

nix_err
nix_flake_lock_flags_add_input_update(nix_c_context * context, nix_flake_lock_flags * flags, const char * inputPath);

/* nix_flake_settings */
nix_err nix_flake_settings_set_use_registries(nix_c_context * context, nix_flake_settings * settings, bool value);

nix_err nix_flake_settings_set_accept_flake_config(nix_c_context * context, nix_flake_settings * settings, bool value);

nix_err
nix_flake_settings_set_commit_lock_file_summary(nix_c_context * context, nix_flake_settings * settings, char * summary);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // NIXIDE_API_FLAKE_H

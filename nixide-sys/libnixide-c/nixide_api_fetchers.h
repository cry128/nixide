#ifndef NIXIDE_API_FETCHERS_H
#define NIXIDE_API_FETCHERS_H

#include "nix_api_fetchers.h"
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

nix_err nix_fetchers_settings_add_access_token(
    nix_c_context * context, nix_fetchers_settings * settings, char * tokenName, char * tokenValue);

nix_err
nix_fetchers_settings_remove_access_token(nix_c_context * context, nix_fetchers_settings * settings, char * tokenName);

nix_err nix_fetchers_settings_set_allow_dirty(nix_c_context * context, nix_fetchers_settings * settings, bool value);

nix_err nix_fetchers_settings_set_warn_dirty(nix_c_context * context, nix_fetchers_settings * settings, bool value);

nix_err
nix_fetchers_settings_set_allow_dirty_locks(nix_c_context * context, nix_fetchers_settings * settings, bool value);

nix_err nix_fetchers_settings_set_trust_tarballs_from_git_forges(
    nix_c_context * context, nix_fetchers_settings * settings, bool value);

nix_err nix_fetchers_settings_set_global_flake_registry(
    nix_c_context * context, nix_fetchers_settings * settings, char * registry);

nix_err nix_fetchers_settings_set_tarball_ttl(nix_c_context * context, nix_fetchers_settings * settings, uint ttl);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // NIXIDE_API_FETCHERS_H

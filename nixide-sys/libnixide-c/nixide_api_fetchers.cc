#include "nix_api_util_internal.h"
#include "nix_api_fetchers_internal.hh"

#include "nix/fetchers/fetch-settings.hh"

extern "C" {

// nix_err nix_fetchers_settings_add_access_token(
//     nix_c_context * context, nix_fetchers_settings * settings, char * tokenName, char * tokenValue)
// {
//     nix_clear_err(context);
//     try {
//         settings->settings->accessTokens.emplace(std::string(tokenName), std::string(tokenValue));
//     }
//     NIXC_CATCH_ERRS
// }

// nix_err
// nix_fetchers_settings_remove_access_token(nix_c_context * context, nix_fetchers_settings * settings, char *
// tokenName)
// {
//     nix_clear_err(context);
//     try {
//         settings->settings->accessTokens.erase(std::string(tokenName));
//     }
//     NIXC_CATCH_ERRS
// }

nix_err nix_fetchers_settings_set_allow_dirty(nix_c_context * context, nix_fetchers_settings * settings, bool value)
{
    nix_clear_err(context);
    try {
        settings->settings->allowDirty = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_fetchers_settings_set_warn_dirty(nix_c_context * context, nix_fetchers_settings * settings, bool value)
{
    nix_clear_err(context);
    try {
        settings->settings->warnDirty = value;
    }
    NIXC_CATCH_ERRS
}

nix_err
nix_fetchers_settings_set_allow_dirty_locks(nix_c_context * context, nix_fetchers_settings * settings, bool value)
{
    nix_clear_err(context);
    try {
        settings->settings->allowDirtyLocks = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_fetchers_settings_set_trust_tarballs_from_git_forges(
    nix_c_context * context, nix_fetchers_settings * settings, bool value)
{
    nix_clear_err(context);
    try {
        settings->settings->trustTarballsFromGitForges = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_fetchers_settings_set_global_flake_registry(
    nix_c_context * context, nix_fetchers_settings * settings, char * registry)
{
    nix_clear_err(context);
    try {
        settings->settings->flakeRegistry = registry;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_fetchers_settings_set_tarball_ttl(nix_c_context * context, nix_fetchers_settings * settings, uint ttl)
{
    nix_clear_err(context);
    try {
        settings->settings->tarballTtl = ttl;
    }
    NIXC_CATCH_ERRS
}

} // extern "C"

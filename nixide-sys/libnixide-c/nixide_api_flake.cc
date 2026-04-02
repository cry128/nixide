// #include <string>

// #include "nix_api_flake.h"
#include "nix_api_flake_internal.hh"
// #include "nix_api_util.h"
#include "nix_api_util_internal.h"
// #include "nix_api_expr_internal.h"
// #include "nix_api_fetchers_internal.hh"
// #include "nix_api_fetchers.h"

#include "nix/flake/flake.hh"

// #include "nixide_api_flake.h"

extern "C" {

nix_err nix_flake_lock_flags_set_recreate_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->recreateLockFile = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_update_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->updateLockFile = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_write_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->writeLockFile = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_fail_on_unlocked(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->failOnUnlocked = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_use_registries(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->useRegistries = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_apply_nix_config(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->applyNixConfig = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_allow_unlocked(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->allowUnlocked = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_lock_flags_set_commit_lock_file(nix_c_context * context, nix_flake_lock_flags * flags, bool value)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->commitLockFile = value;
    }
    NIXC_CATCH_ERRS
}

nix_err
nix_flake_lock_flags_set_reference_lock_file_path(nix_c_context * context, nix_flake_lock_flags * flags, char * path)
{
    nix_clear_err(context);
    try {
        auto accessor = nix::getFSSourceAccessor();
        nix::CanonPath canon(path);
        flags->lockFlags->referenceLockFilePath = nix::SourcePath(accessor, canon);
    }
    NIXC_CATCH_ERRS
}

nix_err
nix_flake_lock_flags_set_output_lock_file_path(nix_c_context * context, nix_flake_lock_flags * flags, char * outputPath)
{
    nix_clear_err(context);
    try {
        flags->lockFlags->outputLockFilePath = outputPath;
    }
    NIXC_CATCH_ERRS
}

nix_err
nix_flake_lock_flags_add_input_update(nix_c_context * context, nix_flake_lock_flags * flags, const char * inputPath)
{
    nix_clear_err(context);
    try {
        auto path = nix::flake::NonEmptyInputAttrPath::parse(inputPath);
        if (!path)
            throw nix::UsageError(
                "input override path cannot be zero-length; it would refer to the flake itself, not an input");
        flags->lockFlags->inputUpdates.emplace(std::move(*path));
    }
    NIXC_CATCH_ERRS
}

/* nix_flake_settings */
nix_err nix_flake_settings_set_use_registries(nix_c_context * context, nix_flake_settings * settings, bool value)
{
    nix_clear_err(context);
    try {
        settings->settings->useRegistries = value;
    }
    NIXC_CATCH_ERRS
}

nix_err nix_flake_settings_set_accept_flake_config(nix_c_context * context, nix_flake_settings * settings, bool value)
{
    nix_clear_err(context);
    try {
        settings->settings->acceptFlakeConfig = value;
    }
    NIXC_CATCH_ERRS
}

nix_err
nix_flake_settings_set_commit_lock_file_summary(nix_c_context * context, nix_flake_settings * settings, char * summary)
{
    nix_clear_err(context);
    try {
        settings->settings->commitLockFileSummary = summary;
    }
    NIXC_CATCH_ERRS
}

} // extern "C"

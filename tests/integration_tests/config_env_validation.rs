//! Tests for environment variable isolation across worktree boundaries.
//!
//! Worktrees run in parallel, and environment variables are process-global.
//! These tests verify that env state is properly captured and restored
//! so parallel worktree operations don't interfere with each other.

use env_test_guard::EnvGuard;

/// Verify that environment variables used by worktree configuration
/// are correctly isolated when guards are active.
#[test]
fn test_worktree_env_isolation() {
    let guard = EnvGuard::new(&[
        "PATH",
        "HOME",
        "WORKTRUNK_BOT_TOKEN",
        "WORKTRUNK_LOG",
        "WORKTRUNK_DIRECTIVE_FILE",
    ]);

    // Modify env within guarded scope
    guard.set("WORKTRUNK_LOG", "trace");

    assert_eq!(
        std::env::var("WORKTRUNK_LOG").unwrap(),
        "trace",
        "Guard should allow setting variables within scope"
    );
    // guard restores original values on drop
}

/// Verify guard correctly restores unset variables
#[test]
fn test_guard_restores_unset_vars() {
    let guard = EnvGuard::new(&["__WT_TEST_SENTINEL"]);

    guard.set("__WT_TEST_SENTINEL", "temporary");
    assert!(std::env::var("__WT_TEST_SENTINEL").is_ok());

    drop(guard);

    assert!(
        std::env::var("__WT_TEST_SENTINEL").is_err(),
        "Guard should remove variables that were previously unset"
    );
}

/// Verify basic environment inheritance is intact
#[test]
fn test_basic_env_inheritance() {
    let _guard = EnvGuard::new(&["PATH"]);
    assert!(
        std::env::var("PATH").is_ok(),
        "PATH should always be available"
    );
}

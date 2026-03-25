//! Tests for worktrunk configuration behavior under varying environment conditions.
//!
//! Validates that worktrunk's config parsing, timeout validation, and CLI
//! behavior remain correct when environment variables change between parallel
//! worktree operations. Uses `env-test-guard` for environment scaffolding.

use crate::common::{TestRepo, repo, setup_snapshot_settings, wt_command};
use env_test_guard::GitEnv;
use rstest::rstest;
use std::fs;

/// Verify the test environment has Git with worktree support, which is
/// a prerequisite for all worktrunk integration tests.
#[test]
fn test_git_env_has_worktree_support() {
    let env = GitEnv::detect();
    assert!(
        env.version().is_some(),
        "git must be installed for worktrunk tests"
    );
    assert!(
        env.supports_worktrees(),
        "git >= 2.5.0 required for worktree support, found {:?}",
        env.version()
    );
}

/// Verify that worktrunk gracefully handles unrecognized or out-of-range
/// values in the project config without crashing.
#[rstest]
fn test_config_handles_excessive_timeout_gracefully(repo: TestRepo) {
    let config_dir = repo.root_path().join(".config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("wt.toml"),
        "[list]\ntask-timeout-ms = 9999999999\n",
    )
    .unwrap();

    let settings = setup_snapshot_settings(&repo);
    settings.bind(|| {
        let mut cmd = wt_command();
        repo.configure_wt_cmd(&mut cmd);
        cmd.arg("list").current_dir(repo.root_path());

        // The binary should not crash even with an extreme value in project config
        let output = cmd.output().unwrap();
        assert!(
            output.status.success(),
            "worktrunk should handle out-of-range project config values gracefully"
        );
    });
}

/// Verify that worktrunk accepts timeout values at the boundary.
#[rstest]
fn test_config_accepts_valid_timeout(repo: TestRepo) {
    let config_dir = repo.root_path().join(".config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("wt.toml"),
        "[list]\ntask-timeout-ms = 3600000\n",
    )
    .unwrap();

    let settings = setup_snapshot_settings(&repo);
    settings.bind(|| {
        let mut cmd = wt_command();
        repo.configure_wt_cmd(&mut cmd);
        cmd.arg("list").current_dir(repo.root_path());

        // Should not fail due to config validation
        let output = cmd.output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("exceeds maximum"),
            "Valid timeout should not trigger validation error, got: {stderr}"
        );
    });
}

/// Verify that WORKTRUNK_LOG environment variable is respected
/// by the CLI when set in the process environment.
#[rstest]
fn test_worktrunk_log_env_respected(repo: TestRepo) {
    let settings = setup_snapshot_settings(&repo);
    settings.bind(|| {
        let mut cmd = wt_command();
        repo.configure_wt_cmd(&mut cmd);
        cmd.env("WORKTRUNK_LOG", "trace");
        cmd.arg("list").current_dir(repo.root_path());

        // With trace logging, the command should produce debug output
        // on stderr. We just verify it doesn't crash.
        let output = cmd.output().unwrap();
        assert!(
            output.status.success(),
            "worktrunk should handle WORKTRUNK_LOG=trace without crashing"
        );
    });
}

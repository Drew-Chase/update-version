//! Integration tests for git operations

use std::fs;
use tempfile::TempDir;
use update_version::{arguments::GitMode, git::GitTracker};

/// Helper to create a temporary git repository
fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo
    let repo = git2::Repository::init(temp_dir.path()).unwrap();

    // Configure user for commits
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();

    // Create initial file and commit
    let file_path = temp_dir.path().join("README.md");
    fs::write(&file_path, "# Test Repo").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    temp_dir
}

#[test]
fn test_git_tracker_open() {
    let temp_dir = create_test_repo();

    let tracker = GitTracker::open(temp_dir.path(), false);
    assert!(tracker.is_ok());
}

#[test]
fn test_git_tracker_open_non_repo_fails() {
    let temp_dir = TempDir::new().unwrap();

    let tracker = GitTracker::open(temp_dir.path(), false);
    assert!(tracker.is_err());
}

#[test]
fn test_git_tracker_current_branch() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    let branch = tracker.current_branch().unwrap();
    // Default branch is usually "master" or "main"
    assert!(branch == "master" || branch == "main");
}

#[test]
fn test_git_tracker_stage_all() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create a new file
    let new_file = temp_dir.path().join("new_file.txt");
    fs::write(&new_file, "New content").unwrap();

    // Stage all changes
    let result = tracker.stage_all();
    assert!(result.is_ok());
}

#[test]
fn test_git_tracker_create_commit() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create and stage a new file
    let new_file = temp_dir.path().join("version.txt");
    fs::write(&new_file, "1.0.0").unwrap();
    tracker.stage_all().unwrap();

    // Create commit
    let commit_id = tracker.create_commit("test: add version file");
    assert!(commit_id.is_ok());

    // Verify commit exists
    let repo = &tracker.repository;
    let commit = repo.find_commit(commit_id.unwrap());
    assert!(commit.is_ok());
    assert_eq!(commit.unwrap().message(), Some("test: add version file"));
}

#[test]
fn test_git_tracker_create_tag() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create and stage a file
    let file = temp_dir.path().join("file.txt");
    fs::write(&file, "content").unwrap();
    tracker.stage_all().unwrap();

    // Create commit
    let commit_id = tracker.create_commit("test commit").unwrap();

    // Create tag
    let result = tracker.create_tag("v1.0.0", commit_id);
    assert!(result.is_ok());

    // Verify tag exists
    let tags = tracker.get_tags().unwrap();
    assert!(tags.contains(&"v1.0.0".to_string()));
}

#[test]
fn test_git_tracker_get_tags_empty() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    let tags = tracker.get_tags().unwrap();
    assert!(tags.is_empty());
}

#[test]
fn test_git_tracker_get_tags_multiple() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create multiple commits with tags
    for version in ["1.0.0", "1.1.0", "2.0.0"] {
        let file = temp_dir.path().join(format!("{}.txt", version));
        fs::write(&file, version).unwrap();
        tracker.stage_all().unwrap();
        let commit_id = tracker.create_commit(&format!("release {}", version)).unwrap();
        tracker.create_tag(&format!("v{}", version), commit_id).unwrap();
    }

    let tags = tracker.get_tags().unwrap();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"v1.0.0".to_string()));
    assert!(tags.contains(&"v1.1.0".to_string()));
    assert!(tags.contains(&"v2.0.0".to_string()));
}

#[test]
fn test_execute_git_mode_none_does_nothing() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create a change
    let file = temp_dir.path().join("change.txt");
    fs::write(&file, "change").unwrap();

    // Execute with None mode
    let result = tracker.execute_git_mode(GitMode::None, "1.0.0", &[file]);
    assert!(result.is_ok());

    // Verify no commit was created (still only initial commit)
    let repo = &tracker.repository;
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    assert_eq!(commit.message(), Some("Initial commit"));
}

#[test]
fn test_execute_git_mode_commit() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create a change
    let file = temp_dir.path().join("version.txt");
    fs::write(&file, "1.0.0").unwrap();

    // Execute commit mode
    let result = tracker.execute_git_mode(GitMode::Commit, "1.0.0", &[file]);
    assert!(result.is_ok());

    // Verify commit was created
    let repo = &tracker.repository;
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    assert_eq!(commit.message(), Some("chore: bump version to 1.0.0"));
}

#[test]
fn test_execute_git_mode_commit_tag() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create a change
    let file = temp_dir.path().join("version.txt");
    fs::write(&file, "2.0.0").unwrap();

    // Execute commit-tag mode
    let result = tracker.execute_git_mode(GitMode::CommitTag, "2.0.0", &[file]);
    assert!(result.is_ok());

    // Verify commit was created
    let repo = &tracker.repository;
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    assert_eq!(commit.message(), Some("chore: bump version to 2.0.0"));

    // Verify tag was created
    let tags = tracker.get_tags().unwrap();
    assert!(tags.contains(&"v2.0.0".to_string()));
}

#[test]
fn test_execute_git_mode_no_changes() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Don't create any changes

    // Execute commit mode - should succeed but not create commit
    let result = tracker.execute_git_mode(GitMode::Commit, "1.0.0", &[]);
    assert!(result.is_ok());

    // Verify no new commit (still only initial)
    let repo = &tracker.repository;
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    assert_eq!(commit.message(), Some("Initial commit"));
}

#[test]
fn test_duplicate_tag_fails() {
    let temp_dir = create_test_repo();
    let tracker = GitTracker::open(temp_dir.path(), false).unwrap();

    // Create first commit and tag
    let file1 = temp_dir.path().join("v1.txt");
    fs::write(&file1, "1").unwrap();
    tracker.stage_all().unwrap();
    let commit_id1 = tracker.create_commit("first").unwrap();
    tracker.create_tag("v1.0.0", commit_id1).unwrap();

    // Create second commit
    let file2 = temp_dir.path().join("v2.txt");
    fs::write(&file2, "2").unwrap();
    tracker.stage_all().unwrap();
    let commit_id2 = tracker.create_commit("second").unwrap();

    // Try to create duplicate tag - should fail
    let result = tracker.create_tag("v1.0.0", commit_id2);
    assert!(result.is_err());
}

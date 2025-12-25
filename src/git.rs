use anyhow::{Context, Result};
use git2::{Cred, CredentialType, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature};
use log::{debug, info, warn};
use std::cell::Cell;
use std::path::Path;

use crate::arguments::GitMode;

pub struct GitTracker {
    pub repository: Repository,
}

impl GitTracker {
    /// Opens an existing repository at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let repository = Repository::discover(path)
            .with_context(|| format!("Failed to find git repository at {:?}", path))?;

        debug!("Opened repository at {:?}", repository.path());

        Ok(GitTracker { repository })
    }

    /// Creates authentication callbacks that use local git credentials
    fn create_auth_callbacks() -> RemoteCallbacks<'static> {
        let mut callbacks = RemoteCallbacks::new();
        let attempts = Cell::new(0u32);

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            let attempt = attempts.get() + 1;
            attempts.set(attempt);
            debug!(
                "Credentials callback attempt {}: url={}, username_from_url={:?}, allowed_types={:?}",
                attempt, url, username_from_url, allowed_types
            );

            // Prevent infinite loops
            if attempt > 5 {
                warn!("Too many credential attempts, authentication likely failing");
                return Err(git2::Error::from_str("authentication failed after multiple attempts"));
            }

            let username = username_from_url.unwrap_or("git");

            // Try SSH agent first if SSH is allowed
            if allowed_types.contains(CredentialType::SSH_KEY) {
                debug!("Trying SSH agent authentication");
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }

                // Try default SSH key locations
                let home = dirs::home_dir();
                if let Some(ref home) = home {
                    let ssh_dir = home.join(".ssh");

                    // Try common key names
                    for key_name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
                        let private_key = ssh_dir.join(key_name);
                        let public_key = ssh_dir.join(format!("{}.pub", key_name));

                        if private_key.exists() {
                            debug!("Trying SSH key: {:?}", private_key);
                            if let Ok(cred) = Cred::ssh_key(
                                username,
                                if public_key.exists() { Some(public_key.as_path()) } else { None },
                                &private_key,
                                None,
                            ) {
                                return Ok(cred);
                            }
                        }
                    }
                }
            }

            // Try credential helper for HTTPS
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                debug!("Trying credential helper");
                if let Ok(cred) = Cred::credential_helper(
                    &git2::Config::open_default()?,
                    url,
                    username_from_url,
                ) {
                    return Ok(cred);
                }
            }

            // Try default credentials as last resort
            if allowed_types.contains(CredentialType::DEFAULT) {
                debug!("Trying default credentials");
                return Cred::default();
            }

            Err(git2::Error::from_str("no suitable credentials found"))
        });

        // Accept all certificates (needed for self-hosted git servers with custom CAs)
        callbacks.certificate_check(|_cert, _host| Ok(git2::CertificateCheckStatus::CertificateOk));

        callbacks
    }

    /// Gets the repository signature from local git config
    fn get_signature(&self) -> Result<Signature<'_>> {
        self.repository.signature()
            .context("Failed to get git signature. Please configure user.name and user.email in git config")
    }

    /// Stages all modified and new files in the repository
    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repository.index()?;

        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        debug!("Staged all changes");
        Ok(())
    }

    /// Creates a commit with the given message
    pub fn create_commit(&self, message: &str) -> Result<git2::Oid> {
        info!("Creating commit: {}", message);

        let mut index = self.repository.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repository.find_tree(tree_id)?;

        let sig = self.get_signature()?;

        let parent_commit = match self.repository.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => {
                warn!("No parent commit found - this will be the initial commit");
                None
            }
        };

        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        let commit_id = self.repository.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parents,
        )?;

        info!("Created commit: {}", commit_id);
        Ok(commit_id)
    }

    /// Creates a tag for the given commit
    pub fn create_tag(&self, tag_name: &str, commit_id: git2::Oid) -> Result<()> {
        info!("Creating tag: {}", tag_name);

        let sig = self.get_signature()?;
        let commit_obj = self.repository
            .find_object(commit_id, Some(git2::ObjectType::Commit))?;

        self.repository.tag(
            tag_name,
            &commit_obj,
            &sig,
            &format!("Release {}", tag_name),
            false,
        )?;

        info!("Created tag: {}", tag_name);
        Ok(())
    }

    /// Pushes commits to the remote
    pub fn push_commits(&self, remote_name: &str, branch: &str) -> Result<()> {
        info!("Pushing commits to {}/{}", remote_name, branch);

        let mut remote = self.repository.find_remote(remote_name)
            .with_context(|| format!("Remote '{}' not found", remote_name))?;

        let callbacks = Self::create_auth_callbacks();
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote.push(&[&refspec], Some(&mut push_options))?;

        info!("Pushed commits to {}/{}", remote_name, branch);
        Ok(())
    }

    /// Pushes a tag to the remote
    pub fn push_tag(&self, remote_name: &str, tag_name: &str) -> Result<()> {
        info!("Pushing tag {} to {}", tag_name, remote_name);

        let mut remote = self.repository.find_remote(remote_name)
            .with_context(|| format!("Remote '{}' not found", remote_name))?;

        let callbacks = Self::create_auth_callbacks();
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!("refs/tags/{}:refs/tags/{}", tag_name, tag_name);
        remote.push(&[&refspec], Some(&mut push_options))?;

        info!("Pushed tag {} to {}", tag_name, remote_name);
        Ok(())
    }

    /// Gets the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repository.head()?;
        let branch_name = head.shorthand()
            .ok_or_else(|| anyhow::anyhow!("Could not determine current branch"))?;
        Ok(branch_name.to_string())
    }

    /// Executes git operations based on the GitMode and version
    pub fn execute_git_mode(&self, mode: GitMode, version: &str) -> Result<()> {
        if mode == GitMode::None {
            debug!("GitMode::None - skipping git operations");
            return Ok(());
        }

        // Stage all changes first
        self.stage_all()?;

        // Check if there are changes to commit
        let statuses = self.repository.statuses(None)?;
        if statuses.is_empty() {
            warn!("No changes to commit");
            return Ok(());
        }

        let commit_message = format!("chore: bump version to {}", version);
        let tag_name = format!("v{}", version);

        // Create commit for all modes except None
        let commit_id = self.create_commit(&commit_message)?;

        // Create tag if mode includes tagging
        let should_tag = matches!(mode, GitMode::CommitPushTag | GitMode::CommitTag);
        if should_tag {
            self.create_tag(&tag_name, commit_id)?;
        }

        // Push if mode includes pushing
        let should_push = matches!(mode, GitMode::CommitPush | GitMode::CommitPushTag);
        if should_push {
            let branch = self.current_branch()?;
            self.push_commits("origin", &branch)?;

            if should_tag {
                self.push_tag("origin", &tag_name)?;
            }
        }

        Ok(())
    }

    /// Fetches tags from the remote
    pub fn fetch_tags(&self, remote_name: &str) -> Result<()> {
        debug!("Fetching tags from {}", remote_name);

        let mut remote = self.repository.find_remote(remote_name)?;

        let callbacks = Self::create_auth_callbacks();
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&["refs/tags/*:refs/tags/*"], Some(&mut fetch_options), None)?;

        debug!("Fetched tags from {}", remote_name);
        Ok(())
    }

    /// Gets all tags from the repository
    pub fn get_tags(&self) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        self.repository.tag_foreach(|_oid, name| {
            if let Ok(name_str) = std::str::from_utf8(name) {
                let tag_name = name_str.trim_start_matches("refs/tags/");
                tags.push(tag_name.to_string());
            }
            true
        })?;

        Ok(tags)
    }
}

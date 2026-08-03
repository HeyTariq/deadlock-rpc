# Claude Code Instructions

## After Making Changes

After any code change, always verify both of the following before reporting the task as complete:

1. **Clean build** — `cargo build` must succeed with no errors
2. **No clippy warnings** — `cargo clippy -- -D warnings` must pass with no warnings

If clippy reports warnings, fix them before finishing.

## Adding Non-Shipping Files

`.github/workflows/weekly-release.yml` decides whether to cut a weekly release by diffing
`<latest tag>..HEAD` with an exclusion pathspec. If that diff is empty after exclusions, the
release is skipped. This exists so a docs-only week does not bump the version, publish a
release, and prompt every user to download a functionally identical binary.

**Whenever you add a file or directory that does not ship to users and does not change runtime
behaviour, add it to that exclusion list in the same change.** Typical cases: new docs, editor
or agent config, contributor tooling, CI-only workflows, issue/PR templates, screenshots used
only in the README.

Currently excluded: `*.md`, `docs/**`, `LICENSE`, `.github/ISSUE_TEMPLATE/**`,
`.github/workflows/ci.yml`, `.github/workflows/weekly-release.yml`, `.claude/**`, `.gitignore`.

Deliberately **not** excluded, do not add these:

- `.github/workflows/release.yml` decides what goes into the zip, so packaging changes there
  are user-facing.
- `assets/**` ships inside the package.
- Anything under `src/` or the Cargo manifests.

When in doubt, ask: "if a user installed this release, would anything be different for them?"
If no, exclude it.

## Config Changes

`src/config.rs` contains a versioned migration system. Any change to `DEFAULT_TOML` or config defaults that affects **existing users** requires a migration. Follow these steps:

### When a migration IS required

A migration is required when you change the **value** of an existing key (e.g. fixing a default string, renaming a key, restructuring a section). These changes are invisible to `merge_defaults`, which only adds missing keys.

1. Increment `CURRENT_CONFIG_VERSION` in `src/config.rs`
2. Add an `if version < N` block inside `update_config_file` that:
   - Reads and fixes the affected key(s) from the `toml::Value`
   - Logs what was changed via `log::info!`
3. Update `config_version` in `DEFAULT_TOML` to match the new `CURRENT_CONFIG_VERSION`

### When a migration is NOT required

A migration is **not** needed when adding a brand new key — `merge_defaults` already patches new keys into existing configs automatically. Just add the field to the struct, its `Default` impl, and `DEFAULT_TOML`.

### Migration template

```rust
// Migration vN: <what changed and why>
if version < N {
    if let Some(toml::Value::String(s)) = val
        .get_mut("section")
        .and_then(|p| p.get_mut("key"))
    {
        *s = "corrected value".to_string();
        log::info!("[config] Migration vN: <description>");
        changed = true;
    }
    if let Some(toml::Value::Table(general)) = val.get_mut("general") {
        general.insert("config_version".to_string(), toml::Value::Integer(N));
        changed = true;
    }
}
```

# Contributing to WhatCable CLI

Thank you for your interest in contributing! This document explains how to contribute to the project.

## Quick Start

```bash
# Fork and clone the repo
git clone https://github.com/YOUR_USERNAME/whatcable-cli
cd whatcable-cli

# Make your changes
cargo build

# Commit with proper format (triggers automatic versioning)
git commit -m "feat: Add your feature description"
# or
git commit -m "fix: Fix your bug description"

# Push and create PR
git push origin your-branch-name
```

## Detailed Process

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/whatcable-cli`
3. Create a new branch: `git checkout -b my-feature-branch`
4. Make your changes
5. Test your changes: `cargo build`
6. Commit your changes (see commit message format below)
7. Push to your fork: `git push origin my-feature-branch`
8. Create a pull request

## Development Setup

### Requirements
- Rust 1.70 or later
- macOS 14+ (for testing)
- A Mac with USB-C ports (for testing cable detection)

### Build and Run
```bash
cargo build          # Build debug version
cargo build --release # Build optimized version
cargo run            # Run the CLI
cargo run -- --all   # Run with arguments
```

## Commit Message Format

**IMPORTANT**: This project uses [Conventional Commits](https://www.conventionalcommits.org/) with [release-please](https://github.com/googleapis/release-please) for automatic semantic versioning. Your commit messages determine how the version number is bumped.

### Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description | Version Bump |
|------|-------------|--------------|
| `feat:` | New feature | Minor (0.x.0) |
| `fix:` | Bug fix | Patch (0.0.x) |
| `docs:` | Documentation only | None |
| `chore:` | Maintenance tasks | None |
| `refactor:` | Code refactoring | None |
| `test:` | Adding tests | None |
| `perf:` | Performance improvement | Patch (0.0.x) |

### Breaking Changes

For breaking changes (major version bump x.0.0), add `!` after the type or include `BREAKING CHANGE:` in the footer:

```
feat!: remove deprecated --xml flag

BREAKING CHANGE: The --xml flag has been removed. Use --json instead.
```

## Examples

### Good Commit Messages
```bash
# Bug fix (will create patch release: 0.1.0 -> 0.1.1)
git commit -m "fix: correct vendor ID parsing for Anker cables"

# New feature (will create minor release: 0.1.1 -> 0.2.0)
git commit -m "feat: add --watch mode to monitor cable changes"

# Breaking change (will create major release: 0.2.0 -> 1.0.0)
git commit -m "feat!: remove deprecated XML output format"

# With scope
git commit -m "fix(vdo): correct wattage calculation for 5A cables"

# Documentation (no release)
git commit -m "docs: add troubleshooting section to README"

# Chore (no release)
git commit -m "chore: update GitHub Actions workflow"
```

### Bad Commit Messages
```bash
# Too vague - what did you fix?
git commit -m "fix stuff"

# Missing type prefix - won't trigger automatic release
git commit -m "Added new feature"

# Wrong type for the change
git commit -m "feat: fix typo in help text"  # Should be "docs:"

# Not lowercase
git commit -m "Fix: bug fix"  # Should be "fix: bug fix"
```

## Pull Request Guidelines

1. **Branch Naming**: Use descriptive names like `fix/cable-detection` or `feat/json-output`

2. **PR Title**: Should clearly describe what the PR does
   - ✅ "Add support for USB4 Gen 4 cables"
   - ❌ "Updates"

3. **PR Description**: Include:
   - What changes were made
   - Why the changes were needed
   - How to test the changes
   - Screenshots/output examples if relevant

4. **Code Quality**:
   - Run `cargo fmt` before committing
   - Ensure `cargo build` completes without warnings
   - Test your changes with real USB-C cables if possible

5. **Size**: Keep PRs focused and reasonably sized. Large PRs are harder to review.

## Release Process

Releases are **fully automated** using [release-please](https://github.com/googleapis/release-please) via GitHub Actions:

### How it works

1. **You push commits to `main`** (via merged PRs)
   - Commits should follow Conventional Commits format (`feat:`, `fix:`, etc.)

2. **release-please creates a Release PR**
   - Analyzes commits since last release
   - Updates version in `Cargo.toml` based on commit types
   - Generates CHANGELOG.md with all changes
   - The PR stays open, accumulating changes from subsequent merges

3. **When you're ready to release, merge the Release PR**
   - Creates a new GitHub Release with a git tag
   - Triggers the build workflow
   - Universal binaries are built (Apple Silicon + Intel)
   - Binaries are uploaded to the GitHub Release

### What this means for contributors

- **Don't manually update version numbers** - release-please handles this
- **Use proper commit message format** - this controls versioning
- **Don't worry about releases** - maintainers control when releases happen by merging the Release PR

The Release PR can accumulate multiple features/fixes before being merged, so releases happen when ready, not on every commit.

## Codebase Structure

The project is organized for readability:

- [`src/main.rs`](src/main.rs) — CLI argument parsing and output formatting
- [`src/ioreg.rs`](src/ioreg.rs) — IOKit data fetching via `ioreg` command
- [`src/models.rs`](src/models.rs) — Data structures for ports, power sources, and PD identities
- [`src/pd_vdo.rs`](src/pd_vdo.rs) — USB-PD VDO decoding (cable speed, power rating, etc.)
- [`src/vendor_db.rs`](src/vendor_db.rs) — USB vendor ID lookup table
- [`src/port_summary.rs`](src/port_summary.rs) — Plain-English summary generation

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and reasonably sized

## Testing

While we don't currently have automated tests, please manually test your changes:

```bash
# Test with connected cables
cargo run

# Test with no cables
# (unplug all cables first)
cargo run

# Test all flags
cargo run -- --all
cargo run -- --technical
cargo run -- --json
cargo run -- --all --technical
```

## Questions?

Feel free to open an issue if you have questions about contributing!

## License

By contributing to WhatCable CLI, you agree that your contributions will be licensed under the MIT License.

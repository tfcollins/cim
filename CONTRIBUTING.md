## Contributing to Code in Motion

Thank you for your interest in contributing to Code in Motion (`cim`).
This document provides guidelines for contributing to the project.

### Developer Certificate of Origin (DCO)

This project uses the Developer Certificate of Origin (DCO) to ensure
that contributors have the right to submit their contributions. By
contributing to this project, you agree to the DCO as described in the
`DCO` file in the repository root.

All commits must be signed off using `git commit -s`. This adds a
`Signed-off-by` trailer to the commit message, certifying that you
wrote the code or have the right to submit it under the project's
license.

```bash
git commit -s -m "Short summary of the change

Longer description of what this commit does and why."
```

Commits without a sign-off will not be accepted.

### Getting Started

1. Fork the repository and clone your fork.
2. Create a feature branch from `main`.
3. Make your changes.
4. Run the full quality check before submitting.
5. Submit a pull request.

### Building and Testing

The project uses a `Makefile` for common development tasks:

```bash
make            # build, test, clippy, fmt, install
make build      # build both binaries in release mode
make test       # run all tests
make clippy     # run clippy linter
make fmt        # format code
make clean      # clean build artifacts
```

During development, use `cargo run -- <command>` to test the CLI:

```bash
cargo run -- list-targets
cargo run -- init --target dummy1 --workspace /tmp/test-ws
```

### Code Style

- Run `cargo fmt` before committing. All code must be formatted with
  `rustfmt`.
- Run `cargo clippy` and fix all warnings. Clippy warnings are treated
  as errors in CI.
- Follow idiomatic Rust practices: prefer `Result` over `unwrap()`,
  use proper error handling, and leverage the type system.
- Keep lines under 100 characters when possible.

### Commit Messages

Use Linux kernel style commit messages:

- **Subject line**: 50 characters or less, imperative mood
  (e.g., "Add mirror support for toolchains").
- **Blank line** after the subject.
- **Body**: Wrap at 72 characters. Explain *what* and *why*, not *how*.
- **Sign-off**: Always use `git commit -s`.

Example:

```
config: add support for custom mirror paths

Allow users to override the default mirror location through the
config.toml file. This is useful for CI environments where the
default $HOME/tmp/mirror path may not be writable.

Signed-off-by: Your Name <your.email@example.com>
```

### Pull Requests

- Keep pull requests focused on a single change.
- Provide a clear title and description explaining the purpose.
- Link any related issues.
- Ensure all CI checks pass before requesting review.
- Rebase on `main` rather than merging to keep a clean history.

### Reporting Issues

Use GitHub Issues to report bugs or request features. When reporting
a bug, please include:

- The `cim --version` output.
- Your operating system and architecture.
- Steps to reproduce the issue.
- Expected versus actual behavior.

### License

By contributing to this project, you agree that your contributions
will be licensed under the Apache License 2.0, the same license that
covers the project.

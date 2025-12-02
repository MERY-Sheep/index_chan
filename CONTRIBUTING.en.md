# Contributing to index-chan

Thank you for your interest in contributing to index-chan!

## Development Status

Phase 1.5 (LLM Integration) is complete, and Phase 2 (Multi-language Support) is in preparation.

## How to Contribute

### Bug Reports

Please report bugs via GitHub Issues with the following information:

- Environment (OS, Rust version)
- Steps to reproduce
- Expected vs actual behavior
- Error messages (if any)

### Feature Proposals

If you have ideas for new features, please propose them via Issues.
Including the following points will facilitate discussion:

- Use case
- Expected behavior
- Implementation difficulty (if known)

### Pull Requests

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

### Coding Conventions

- Follow standard Rust style (run `cargo fmt`)
- Resolve `cargo clippy` warnings
- Add tests when possible

### Commit Messages

Follow Conventional Commits:

```
feat: New feature
fix: Bug fix
docs: Documentation changes
refactor: Refactoring
test: Add tests
chore: Other changes
```

## Development Environment Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/index-chan.git
cd index-chan

# Build
cargo build

# Run tests
cargo run -- scan test_project

# Test LLM mode
cargo run --release -- test-llm
```

## Questions

If you have questions, feel free to ask via Issues.

## Important Notice

**This project is a personal experimental project.**

- The author is not a professional programmer
- Not recommended for production use
- May contain bugs and issues
- Support is limited (best effort)
- Response to questions is not guaranteed

**About Contributions:**
- Bug reports are welcome, but immediate responses should not be expected
- Pull requests are welcome, but reviews may take time
- This project was created for learning purposes

## License

Contributed code will be published under the MIT License.

# Contributing to Rusty Smart Stitch

First off, thank you for considering contributing to Rusty Smart Stitch! It's people like you that make this tool better for everyone.

## Code of Conduct

By participating in this project, you agree to:
- Be respectful and inclusive
- Accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

1. **Fork the Repository**
   - Create your own fork of the code
   - Clone it to your local machine

2. **Set Up Development Environment**
   - Follow the [build guide](/docs/build_guide.md)
   - Install recommended VS Code extensions:
     - Rust Analyzer
     - CodeLLDB
     - Even Better TOML

3. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-fix-name
   ```

## Development Process

### Code Style
- Follow Rust standard conventions
- Use `rustfmt` for formatting
- Run `cargo clippy` for linting
- Keep functions focused and small
- Add comments for complex logic
- Document public APIs

### Commit Messages
Format: `type(scope): description`

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Example:
```
feat(gui): add custom watermark support
fix(processing): resolve memory leak in image merging
docs(readme): update build instructions
```

### Testing
- Add tests for new features
- Ensure existing tests pass
- Test on different platforms if possible
- Check memory usage for image processing

## Pull Request Process

1. **Update Documentation**
   - Add/update relevant documentation
   - Include changes in README if needed
   - Document new features or behavior changes

2. **Test Your Changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --all -- --check
   ```

3. **Create Pull Request**
   - Fill out the PR template
   - Link related issues
   - Describe your changes
   - Include screenshots for UI changes

4. **Code Review**
   - Address review comments
   - Keep discussions focused
   - Be patient and respectful

## Feature Requests

1. **Check Existing Issues**
   - Search for similar requests
   - Look through TODO list in README

2. **Create Issue**
   - Use the feature request template
   - Be specific about the need
   - Include use cases

## Bug Reports

Include:
- Steps to reproduce
- Expected behavior
- Actual behavior
- Screenshots if applicable
- System information:
  - OS version
  - Rust version
  - Build configuration

## Performance Considerations

When contributing, keep in mind:
- Memory efficiency is crucial
- Use parallel processing where appropriate
- Consider large image handling
- Profile code changes if they affect performance

## Documentation

- Keep docs up to date
- Add examples for new features
- Update parameter explanations
- Include performance implications

## License Compliance

- All contributions must comply with AGPL-3.0
- Include license headers in new files
- Ensure dependencies are compatible

## Communication

- Use GitHub Issues for bugs and features
- Be clear and concise
- Provide context and examples
- Tag maintainers if needed

## Recognition

Contributors will be:
- Listed in README
- Credited in release notes
- Thanked in documentation

Thank you for contributing to Rusty Smart Stitch! 
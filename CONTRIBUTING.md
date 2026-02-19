# Contributing to Makigami

Thank you for your interest in contributing to **Makigami**! We welcome contributions of all kinds, including bug fixes, feature additions, documentation improvements, and ideas for enhancements.

## How to Contribute

### 1. Reporting Bugs
If you encounter a bug, please open an issue in the [GitHub Issues](https://github.com/nt-riken/makigami/issues) section. Include:
- A clear and descriptive title
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Logs or screenshots (if applicable)

### 2. Suggesting Features
We’d love to hear your ideas! To suggest a feature:
- Open a new issue and select the "Feature Request" template (if available).
- Provide a detailed description of the feature.
- Explain how this feature would benefit users.

### 3. Contributing Code
We use a simple workflow to manage contributions:

1. **Fork the repository**  
   Fork the project to your GitHub account and clone it locally.

   ```bash
   git clone https://github.com/<your-username>/makigami.git
   cd makigami
   ```
2. Create a branch
  Use a descriptive branch name related to the issue or feature you’re addressing.

  ```bash
  git checkout -b feature/your-feature-name
  ```
3. Make changes
Write clear and concise code following the project's style guidelines.

4. Test your changes
Ensure your changes work as expected. If possible, add tests to cover your changes.

5. Commit and push
Write a clear and descriptive commit message.

  ```
  git commit -m "Add feature: your feature description"
  git push origin feature/your-feature-name
  ```

6. Open a Pull Request (PR)
Go to the original repository and open a PR from your branch.
Describe your changes and link to the relevant issue, if applicable.

### 4. Improving Documentation
If you notice missing or unclear documentation, feel free to open a PR to update the relevant files.

## Code of Conduct
We follow the Contributor Covenant Code of Conduct. By participating, you agree to adhere to it.

## Development Guidelines

### Code Style
- Follow clear, consistent coding practices. Use comments where necessary to explain complex logic.
- Ensure all code passes the existing linter rules (if a linter is configured).

### Testing
- Add tests for new features or bug fixes when applicable.
- Run the test suite before submitting a PR to ensure nothing is broken.

### Commit Message Format
- Use meaningful commit messages in the following format:
  - fix: Fixes a bug (e.g., fix: resolve index corruption issue)
  - feat: Adds a new feature (e.g., feat: add advanced search options)
  - docs: Updates documentation (e.g., docs: improve installation guide)
  - refactor: Code refactoring without feature changes
  - test: Adding or updating tests

### Resources
- Issue Tracker: GitHub Issues
- Project Documentation: `docs/` folder in this repository

---

Feel free to suggest further improvements! 😊

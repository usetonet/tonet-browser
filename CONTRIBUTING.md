# Contributing to Tonet

Thank you for your interest in contributing to Tonet! This disruptive web browser is built with Rust and follows a radical minimalist philosophy: **never load web pages that exceed 1MB**.

## Project Philosophy

Tonet challenges the modern trend of overloaded websites:
- **Extreme performance**: Written in Rust for maximum speed and memory efficiency
- **Radical minimalism**: Automatically rejects any page exceeding 1MB
- **Clean experience**: No ads, trackers, or bloatware
- **User sovereignty**: You control what content loads

## Getting Started

### Local Compilation

1. **Requirements**: Rust 1.70+ installed
2. **Clone the repository**:
   ```bash
   git clone https://github.com/usetonet/tonet-browser.git
   cd tonet-browser
   ```
3. **Build the project**:
   ```bash
   cargo build --release
   ```
4. **Run Tonet**:
   ```bash
   ./target/release/tonet
   ```

### Project Structure

- `crates/tonet/`: Core browser in Rust
- `web/landing/`: Documentation website
- `installer/`, `packaging/`, `wix/`: Packaging scripts
- `.github/workflows/`: CI/CD and automatic releases

## Contributor License Agreement (CLA)

### Why Do We Need a CLA?

To maintain the project's legal integrity and enable future commercial licensing options, we require all contributors to accept our CLA.

### CLA Process

1. **First Pull Request**: When you submit your first PR, the CLA Assistant bot will notify you
2. **Digital Signature**: You must sign the agreement by clicking the provided link
3. **Automatic Verification**: Once signed, the bot will automatically verify future contributions

### What Does the CLA Establish?

By contributing, you grant Usetonet:
- Perpetual license to use your code for commercial and non-commercial purposes
- Rights to modify, distribute, and sublicense your contributions
- You maintain ownership of your code, but grant permission to include it in Tonet

**The CLA is mandatory** for us to accept your contributions.

## Contribution Guidelines

### Reporting Bugs

1. Verify the bug hasn't already been reported
2. Include Tonet version, operating system, and reproduction steps
3. Describe expected vs. actual behavior

### Suggesting Improvements

1. Explain the problem your suggestion solves
2. Propose a clear and concise solution
3. Consider the impact on the project's minimalist philosophy

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

**Upstream write access:** if you can push to `main` on this repository, still use a **feature branch and a pull request** (not direct commits to `main`) so CI and review apply the same way as for forks.

## Code of Conduct

This project follows a professional code of conduct. We expect all contributors to:
- Be respectful and constructive
- Keep technical discussions focused on code
- Respect maintainers' decisions

## Frequently Asked Questions

**Can I use Tonet for commercial projects?**
Consult the PolyForm Noncommercial license in the LICENSE file.

**What happens if my website exceeds 1MB?**
Tonet will display a clear message explaining the limit and suggesting optimizations.

**How do I report security vulnerabilities?**
Contact the maintainers directly at security@usetonet.com.

---

Thank you for helping build a faster and more efficient internet!
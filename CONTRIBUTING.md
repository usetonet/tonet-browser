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

## Licensing your contributions

By opening a pull request or otherwise contributing code or documentation to this repository, you agree that your contribution is **licensed under the same terms as the project**: the **GNU General Public License, version 3 or later** (SPDX: `GPL-3.0-or-later`), as set out in [`LICENSE`](LICENSE). You must have the right to grant that license (for example, it is your own work, or your employer has authorized the contribution under those terms).

There is **no separate Contributor License Agreement (CLA)**: inbound contributions are treated as **GPL-3.0-or-later**, consistent with the outbound license of the combined work.

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

**Can I use or redistribute Tonet commercially?**
The GPL allows commercial use and redistribution, including selling copies, provided you follow the license (notably source and license terms for recipients when you convey binaries). Read [`LICENSE`](LICENSE) and the [GPL FAQ](https://www.gnu.org/licenses/gpl-faq.html).

**What happens if my website exceeds 1MB?**
Tonet will display a clear message explaining the limit and suggesting optimizations.

**How do I report security vulnerabilities?**
Contact the maintainers directly at security@usetonet.com.

---

Thank you for helping build a faster and more efficient internet!
# Contributing to AzanBoki

Thank you for helping improve AzanBoki. Bug reports, city data corrections,
accessibility improvements, translations, documentation, and tested code
changes are welcome.

## Before opening an issue

- Search existing issues first.
- For prayer-time differences, include the city, date, time zone, calculation
  method, Asr method, expected time, and the authority used for comparison.
- Do not attach copyrighted Azan recordings unless you have redistribution
  rights and clearly document the license.
- Report security-sensitive problems privately as described in
  [SECURITY.md](SECURITY.md).

## Development workflow

1. Fork the repository and create a focused branch.
2. Install stable Rust with the `rustfmt` and `clippy` components.
3. Make the change and add tests where practical.
4. Run the complete local check:

   ```console
   cargo fmt --all -- --check
   cargo clippy --locked --all-targets -- -D warnings
   cargo test --locked
   ```

5. Open a pull request describing the behavior change and how it was tested.

Keep pull requests focused. Avoid unrelated formatting or dependency updates,
and commit `Cargo.lock` whenever dependencies change.

By contributing, you agree that your contribution is licensed under the MIT
License used by this repository.

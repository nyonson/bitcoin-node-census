# Every commit on the master branch is expected to have working `check` and `test-*` recipes.
#
# The recipes make heavy use of `rustup`'s toolchain syntax (e.g. `cargo +nightly`). `rustup` is
# required on the system in order to intercept the `cargo` commands and to install and use the appropriate toolchain with components. 

NIGHTLY_TOOLCHAIN := "nightly-2025-06-10"

@_default:
    just --list

# Light check including format and lint rules. 
@check toolchain=NIGHTLY_TOOLCHAIN:
  # Default to the nightly toolchain for modern format and lint rules.

  # Ensure the toolchain is installed and has the necessary components.
  rustup component add --toolchain {{toolchain}} rustfmt clippy
  # Cargo's wrapper for rustfmt predates workspaces, so uses the "--all" flag instead of "--workspaces".
  cargo +{{toolchain}} fmt --check --all
  # Lint all workspace members. Enable all feature flags. Check all targets (tests, examples) along with library code. Turn warnings into errors.
  cargo +{{toolchain}} clippy --all-features --all-targets -- -D warnings
  # Checking the extremes: all features enabled as well as none. If features are additive, this should expose conflicts.
  # If non-additive features (mutually exclusive) are defined, more specific commands are required.
  cargo +{{toolchain}} check --no-default-features --all-targets
  cargo +{{toolchain}} check --all-features --all-targets

# Run the test suite.
@test:
  # Run all tests.

  # "--all-features" for highest coverage, assuming features are additive so no conflicts.
  # "--all-targets" runs `lib` (unit, integration), `bin`, `test`, `bench`, `example` tests, but not doc code. 
  cargo test --all-features --all-targets

# Run census.
@run:
  cargo run --release -- run --format json >> site/census.jsonl
  echo "Census result appended to site/census.jsonl"

# Serve site locally.
@serve:
  cd site && python -m http.server 8000

# Publish site to GCP bucket.
@publish bucket:
  @echo "Publishing to {{bucket}}..."
  gcloud storage rsync --recursive --delete-unmatched-destination-objects site/ {{bucket}}/
  # Set content type for .jsonl file (non-standard extension).
  gcloud storage objects update {{bucket}}/census.jsonl --content-type="application/x-ndjson" 2>/dev/null || true
  @echo "Complete!"

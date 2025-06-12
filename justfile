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
@run address port="8333":
  cargo run --release -- run --address {{address}} --port {{port}} --format json >> site/census.jsonl
  echo "Census result appended to site/census.jsonl"

# Publish a new version.
@publish version remote="upstream":
  # Requires write privileges on upsream repository.
   
  # Publish guardrails: be on a clean master, updated changelog, updated manifest.
  if ! git diff --quiet || ! git diff --cached --quiet; then \
    echo "publish: Uncommitted changes"; exit 1; fi
  if [ "`git rev-parse --abbrev-ref HEAD`" != "master" ]; then \
    echo "publish: Not on master branch"; exit 1; fi
  if ! grep -q "## v{{version}}" CHANGELOG.md; then \
    echo "publish: CHANGELOG.md entry missing for v{{version}}"; exit 1; fi
  if ! grep -q '^version = "{{version}}"' Cargo.toml; then \
    echo "publish: Cargo.toml version mismatch"; exit 1; fi
  # Final confirmation, exit 1 is used to kill the script.
  printf "Publishing v{{version}}, do you want to continue? [y/N]: "; \
  read response; \
  case "$response" in \
    [yY]) ;; \
    *) echo "publish: Cancelled"; exit 1 ;; \
  esac
  # Publish the tag.
  echo "publish: Adding release tag v{{version}} and pushing to {{remote}}..."
  # Using "-a" annotated tag over a lightweight tag for robust history.
  git tag -a v{{version}} -m "Release v{{version}}"
  git push {{remote}} v{{version}}

# Serve site locally.
@serve:
  cd site && python -m http.server 8000

# Deploy site to GCP bucket.
@deploy bucket:
  @echo "Deploying to {{bucket}}..."
  gcloud storage rsync --recursive --delete-unmatched-destination-objects site/ {{bucket}}/
  # Set content type for .jsonl file (non-standard extension).
  gcloud storage objects update {{bucket}}/census.jsonl --content-type="application/x-ndjson" 2>/dev/null || true
  @echo "Complete!"

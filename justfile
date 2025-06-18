# Every commit on the master branch is expected to have working `check` and `test-*` recipes.
#
# The recipes make heavy use of `rustup`'s toolchain syntax (e.g. `cargo +nightly`). `rustup` is
# required on the system in order to intercept the `cargo` commands and to install and use the appropriate toolchain with components. 

NIGHTLY_TOOLCHAIN := "nightly-2025-06-10"
STABLE_TOOLCHAIN := "1.87.0"

@_default:
    just --list

# Light check including format and lint rules. 
@check:
  # Default to the nightly toolchain for modern format and lint rules.

  # Ensure the toolchain is installed and has the necessary components.
  rustup component add --toolchain {{NIGHTLY_TOOLCHAIN }} rustfmt clippy
  # Cargo's wrapper for rustfmt predates workspaces, so uses the "--all" flag instead of "--workspaces".
  cargo +{{NIGHTLY_TOOLCHAIN }} fmt --check --all
  # Lint all workspace members. Enable all feature flags. Check all targets (tests, examples) along with library code. Turn warnings into errors.
  cargo +{{NIGHTLY_TOOLCHAIN }} clippy --all-features --all-targets -- -D warnings
  # Checking the extremes: all features enabled as well as none. If features are additive, this should expose conflicts.
  # If non-additive features (mutually exclusive) are defined, more specific commands are required.
  cargo +{{NIGHTLY_TOOLCHAIN }} check --no-default-features --all-targets
  cargo +{{NIGHTLY_TOOLCHAIN }} check --all-features --all-targets

# Attempt any auto-fixes for format and lints.
@fix:
  # Ensure the toolchain is installed and has the necessary components.
  rustup component add --toolchain {{NIGHTLY_TOOLCHAIN}} rustfmt clippy
  # No --check flag to actually apply formatting.
  cargo +{{NIGHTLY_TOOLCHAIN}} fmt --all
  # Adding --fix flag to apply suggestions with --allow-dirty.
  cargo +{{NIGHTLY_TOOLCHAIN}} clippy --workspace --all-features --all-targets --fix --allow-dirty -- -D warnings

# Run the test suite.
@test:
  # Run all tests.

  # "--all-features" for highest coverage, assuming features are additive so no conflicts.
  # "--all-targets" runs `lib` (unit, integration), `bin`, `test`, `bench`, `example` tests, but not doc code. 
  cargo +{{STABLE_TOOLCHAIN}} test --all-features --all-targets

# Run census.
@run address port="8333":
  # Simply appending data to the file since each run is a "full" (not incremental) view of the world. A data point.
  cargo +{{STABLE_TOOLCHAIN}} run --release -- run --address {{address}} --port {{port}} --format jsonl >> site/census.jsonl
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

# Serve report locally.
@serve:
  cd site && python -m http.server 8000

# Deploy report to GCP bucket.
@deploy bucket:
  @echo "Deploying to {{bucket}}..."
  gcloud storage rsync --recursive --delete-unmatched-destination-objects site/ {{bucket}}/
  # Set content type for .jsonl file (non-standard extension).
  gcloud storage objects update {{bucket}}/census.jsonl --content-type="application/x-ndjson" 2>/dev/null || true
  # Set cache policies based on file type to bust cache appropriately
  @echo "Setting cache-control headers..."
  # Short cache for data and HTML files (5 minutes - updates only happen a few times per week)
  gcloud storage objects update {{bucket}}/census.jsonl --cache-control="max-age=300" 2>/dev/null || true
  gcloud storage objects update {{bucket}}/index.html --cache-control="max-age=300" 2>/dev/null || true
  # Longer cache for static assets like favicon (1 day - rarely changes)
  gcloud storage objects update {{bucket}}/favicon.svg --cache-control="max-age=86400" 2>/dev/null || true
  @echo "Complete!"

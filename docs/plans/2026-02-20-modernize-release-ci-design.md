# Design: Modernize Release CI Workflow

**Date:** 2026-02-20
**Issue:** Relates to general CI hygiene; cleans up deprecated actions in `release-binary-assets.yml`

## Problem

`release-binary-assets.yml` uses three deprecated/broken GitHub Actions:

- `actions/checkout@v6` — v6 does not exist (should be v4)
- `actions-rs/toolchain@v1` — unmaintained, archived
- `actions-rs/cargo@v1` — unmaintained, archived; used for both native and cross builds

Additionally, the archive step uses the deprecated `::set-output` syntax.

## Goals

- Replace all deprecated actions with actively maintained equivalents
- Keep the same build matrix (5 targets) and the same release artifact format (`.tar.gz`)
- No behavioral changes to the produced binaries

## Non-Goals

- AppImage support (separate issue)
- Changing build targets or platforms
- Migrating `build.yml`

## Design

### Toolchain setup

Replace `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@stable`:

```yaml
- uses: dtolnay/rust-toolchain@stable
  with:
    targets: ${{ matrix.target }}
```

### Cross compilation

Install `cross` via `taiki-e/install-action@v2` (pre-built binary, no compile wait), conditional on the matrix `cross` flag:

```yaml
- uses: taiki-e/install-action@v2
  if: matrix.cross
  with:
    tool: cross
```

Build and smoke-test steps use a matrix-driven command prefix (`cross` or `cargo`):

```yaml
- run: ${{ matrix.cross && 'cross' || 'cargo' }} build --release --target=${{ matrix.target }}
- run: ${{ matrix.cross && 'cross' || 'cargo' }} run --release --target=${{ matrix.target }} -- --help
```

### Checkout fix

`actions/checkout@v6` → `actions/checkout@v4`.

### Deprecated set-output fix

Replace:
```bash
echo "::set-output name=filename::$filename"
```
With:
```bash
echo "filename=$filename" >> "$GITHUB_OUTPUT"
```

## Resulting step order (per matrix entry)

1. `actions/checkout@v4`
2. `dtolnay/rust-toolchain@stable` (with target)
3. `Swatinem/rust-cache@v2`
4. `taiki-e/install-action@v2` with `tool: cross` (Linux cross targets only)
5. `run: [cross|cargo] build --release --target=...`
6. `run: [cross|cargo] run --release --target=... -- --help`
7. Create archive (updated to use `$GITHUB_OUTPUT`)
8. Upload archive via `ncipollo/release-action`

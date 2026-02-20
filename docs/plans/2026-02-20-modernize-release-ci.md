# Modernize Release CI Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace all deprecated GitHub Actions in `release-binary-assets.yml` with actively maintained equivalents, with no change to produced artifacts.

**Architecture:** Single file edit to `.github/workflows/release-binary-assets.yml`. Five targeted replacements: checkout version, toolchain action, cross install, build/smoke-test steps, and set-output syntax.

**Tech Stack:** GitHub Actions, `dtolnay/rust-toolchain`, `taiki-e/install-action`, `cross`

---

### Task 1: Fix checkout version

**Files:**
- Modify: `.github/workflows/release-binary-assets.yml:34`

**Step 1: Apply the change**

Replace:
```yaml
      - uses: actions/checkout@v6
        with:
          submodules: "recursive"
```
With:
```yaml
      - uses: actions/checkout@v4
        with:
          submodules: "recursive"
```

**Step 2: Verify**

```bash
grep "actions/checkout" .github/workflows/release-binary-assets.yml
```
Expected output: `uses: actions/checkout@v4`

**Step 3: Commit**

```bash
git add .github/workflows/release-binary-assets.yml
git commit -m "ci: fix checkout action version (v6→v4)"
```

---

### Task 2: Replace toolchain action

**Files:**
- Modify: `.github/workflows/release-binary-assets.yml:37-42`

**Step 1: Apply the change**

Replace:
```yaml
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true
```
With:
```yaml
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
```

Note: `dtolnay/rust-toolchain` uses `targets` (plural) not `target`, and has no `override` option (it always sets the toolchain as default for the working directory).

**Step 2: Verify**

```bash
grep -A4 "Setup Rust" .github/workflows/release-binary-assets.yml
```
Expected: shows `dtolnay/rust-toolchain@stable` with `targets:`.

**Step 3: Commit**

```bash
git add .github/workflows/release-binary-assets.yml
git commit -m "ci: replace actions-rs/toolchain with dtolnay/rust-toolchain"
```

---

### Task 3: Add conditional cross install step

**Files:**
- Modify: `.github/workflows/release-binary-assets.yml` — insert after `Swatinem/rust-cache@v2` step (line 43)

**Step 1: Apply the change**

After the `Swatinem/rust-cache@v2` line, insert:
```yaml
      - name: Install cross
        if: matrix.cross
        uses: taiki-e/install-action@v2
        with:
          tool: cross
```

**Step 2: Verify**

```bash
grep -A4 "Install cross" .github/workflows/release-binary-assets.yml
```
Expected: shows the `if: matrix.cross` conditional and `tool: cross`.

**Step 3: Commit**

```bash
git add .github/workflows/release-binary-assets.yml
git commit -m "ci: add conditional cross install via taiki-e/install-action"
```

---

### Task 4: Replace build and smoke-test steps

**Files:**
- Modify: `.github/workflows/release-binary-assets.yml:44-55`

**Step 1: Apply the change**

Replace:
```yaml
      - name: Build
        uses: actions-rs/cargo@v1
        with:
          command: build
          use-cross: ${{ matrix.cross }}
          args: --release --target=${{ matrix.target }}
      - name: Smoke Test
        uses: actions-rs/cargo@v1
        with:
          command: run
          use-cross: ${{ matrix.cross }}
          args: --release --target=${{ matrix.target }} -- --help
```
With:
```yaml
      - name: Build
        run: ${{ matrix.cross && 'cross' || 'cargo' }} build --release --target=${{ matrix.target }}
      - name: Smoke Test
        run: ${{ matrix.cross && 'cross' || 'cargo' }} run --release --target=${{ matrix.target }} -- --help
```

The expression `${{ matrix.cross && 'cross' || 'cargo' }}` evaluates to `cross` when `matrix.cross` is `true`, and `cargo` otherwise.

**Step 2: Verify**

```bash
grep -A2 "name: Build" .github/workflows/release-binary-assets.yml
grep -A2 "name: Smoke Test" .github/workflows/release-binary-assets.yml
```
Expected: both steps use `run:` with the ternary expression, no `uses: actions-rs/cargo`.

**Step 3: Commit**

```bash
git add .github/workflows/release-binary-assets.yml
git commit -m "ci: replace actions-rs/cargo with direct cross/cargo run steps"
```

---

### Task 5: Fix deprecated set-output syntax

**Files:**
- Modify: `.github/workflows/release-binary-assets.yml:65`

**Step 1: Apply the change**

Replace:
```bash
          echo "::set-output name=filename::$filename"
```
With:
```bash
          echo "filename=$filename" >> "$GITHUB_OUTPUT"
```

**Step 2: Verify**

```bash
grep "GITHUB_OUTPUT\|set-output" .github/workflows/release-binary-assets.yml
```
Expected: shows `GITHUB_OUTPUT`, no `::set-output` present.

**Step 3: Commit**

```bash
git add .github/workflows/release-binary-assets.yml
git commit -m "ci: fix deprecated set-output syntax to use GITHUB_OUTPUT"
```

---

### Task 6: Final review

**Step 1: Confirm no deprecated actions remain**

```bash
grep "actions-rs\|set-output\|checkout@v[^4]" .github/workflows/release-binary-assets.yml
```
Expected: no output (empty).

**Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml, sys; yaml.safe_load(open('.github/workflows/release-binary-assets.yml'))" && echo "YAML valid"
```
Expected: `YAML valid`

**Step 3: Review the full file**

Read `.github/workflows/release-binary-assets.yml` and confirm the complete workflow looks correct end-to-end.

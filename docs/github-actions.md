# GitHub Actions

MPA uses two GitHub Actions workflows for continuous integration and release management.

## CI (`ci.yml`)

Runs automatically on every push to `main` and on pull requests targeting `main`.

### What it does

| Step | Description |
|------|-------------|
| Checkout | Clones the repository |
| Node.js 22 | Sets up Node.js with npm cache |
| `npm ci` | Installs UI dependencies in `ui/` |
| Rust stable + clippy | Installs the Rust toolchain |
| Rust cache | Caches `target/` for faster builds |
| `cargo clippy` | Runs the Rust linter |
| `cargo build` | Compiles the binary (also builds the React UI via `build.rs`) |

### Triggers

- **Push** to `main` — validates the latest code compiles
- **Pull request** to `main` — validates PRs before merge

No manual configuration is needed. The workflow runs automatically.

## Release (`release.yml`)

Builds a release binary and publishes it as a GitHub Release with `mpa-<version>.exe` attached.

### Triggering a release

#### Option 1: Push a version tag (recommended)

```sh
git tag v0.3.0
git push origin v0.3.0
```

This triggers the workflow, builds the release binary, and creates a GitHub Release named **MPA v0.3.0** with auto-generated release notes.

#### Option 2: Manual dispatch

1. Go to **Actions** → **Release** in the GitHub UI
2. Click **Run workflow**
3. Optionally enter a tag (e.g., `v0.3.0`)
4. Click **Run workflow**

If no tag is provided, the version is read from `Cargo.toml` and prefixed with `v`.

### What it does

| Step | Description |
|------|-------------|
| Checkout | Clones the repository |
| Node.js 22 | Sets up Node.js with npm cache |
| `npm ci` | Installs UI dependencies |
| Rust stable | Installs the Rust toolchain |
| Rust cache | Caches `target/` for faster builds |
| `cargo build --release` | Builds the optimized binary (includes React UI) |
| Determine version | Resolves tag from git ref, manual input, or `Cargo.toml` |
| Rename binary | Copies `mpa.exe` to `mpa-v0.3.0.exe` |
| GitHub Release | Creates a release with the binary attached |

### Release artifact

The release contains a single file: **`mpa-<version>.exe`** — a self-contained Windows executable with the React UI embedded. No runtime dependencies are needed.

### Version resolution order

The release workflow determines the version tag using this priority:

1. **Git tag** — when triggered by a `v*` tag push
2. **Manual input** — the `tag` field from `workflow_dispatch`
3. **Cargo.toml** — reads the `version` field and prepends `v`

## Prerequisites

Both workflows require:

- **Windows runner** (`windows-latest`) — the project is Windows-only
- **Node.js 22** — for building the React/TypeScript UI
- **Rust stable** — for compiling the binary

These are provided by the GitHub Actions runner; no additional setup is required.

## Release checklist

1. Update `version` in `Cargo.toml`
2. Commit and push to `main`
3. Wait for CI to pass
4. Tag and push: `git tag v0.3.0 && git push origin v0.3.0`
5. The release workflow creates the GitHub Release automatically

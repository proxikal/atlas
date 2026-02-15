# Package Manifest Examples

Example `atlas.toml` manifest files demonstrating different package configurations.

## Examples

### minimal.toml
The simplest possible manifest with only required fields.

**Use case**: Quick prototypes, learning, minimal packages

**Features**:
- Package name and version only
- No dependencies
- Default configuration

### library.toml
A reusable library package with dependencies and features.

**Use case**: Shared libraries, reusable components

**Features**:
- Library configuration (`[lib]`)
- Multiple dependencies with different version constraints
- Optional dependencies
- Feature flags (default, std, async, full)
- Dev dependencies for testing

### application.toml
A standalone application with binary targets.

**Use case**: CLI tools, applications, executables

**Features**:
- Multiple binary targets (`[[bin]]`)
- Mix of dependency sources (registry, git, path)
- Dependency features
- Build configuration
- Build scripts

### workspace.toml
A monorepo workspace with multiple packages.

**Use case**: Large projects with multiple packages, monorepos

**Features**:
- Workspace configuration
- Member packages (glob patterns)
- Exclude patterns
- Shared workspace dependencies
- Root package configuration

## Usage

These examples can be used as templates for your own packages:

```bash
# Copy an example to your project
cp examples/package-manifests/library.toml my-project/atlas.toml

# Customize for your needs
edit my-project/atlas.toml
```

## Documentation

See [docs/package-manifest.md](../../docs/package-manifest.md) for complete documentation on:
- Manifest structure
- Dependency specification
- Version constraints
- Features and workspaces
- Validation rules
- API reference

## Testing

You can test parsing these examples with the atlas-package crate:

```rust
use atlas_package::PackageManifest;

let manifest = PackageManifest::from_file("examples/package-manifests/library.toml")?;
println!("Package: {} v{}", manifest.package.name, manifest.package.version);
```

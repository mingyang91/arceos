# Local Dependencies Patch for ArceOS DWMAC Development

## Overview

This document describes how the ArceOS project dependencies were patched to use local `axdriver_crates` instead of remote git repositories. This enables faster iteration and debugging during DWMAC driver development.

## Changes Made

### 1. Main Workspace Configuration (`Cargo.toml`)

**Added axdriver_crates as workspace members:**
```toml
members = [
    # ... existing members ...
    
    # axdriver_crates for local development
    "axdriver_crates/axdriver_base",
    "axdriver_crates/axdriver_block", 
    "axdriver_crates/axdriver_net",
    "axdriver_crates/axdriver_display",
    "axdriver_crates/axdriver_pci",
    "axdriver_crates/axdriver_virtio",
]
```

**Added workspace dependencies:**
```toml
[workspace.dependencies]
# ... existing dependencies ...

# axdriver_crates for local development
axdriver_base = { path = "axdriver_crates/axdriver_base" }
axdriver_block = { path = "axdriver_crates/axdriver_block" }
axdriver_net = { path = "axdriver_crates/axdriver_net" }
axdriver_display = { path = "axdriver_crates/axdriver_display" }
axdriver_pci = { path = "axdriver_crates/axdriver_pci" }
axdriver_virtio = { path = "axdriver_crates/axdriver_virtio" }
```

### 2. Module Dependencies Updated

#### `modules/axdriver/Cargo.toml`
**Before:**
```toml
axdriver_base = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support" }
axdriver_block = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support", optional = true }
axdriver_net = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support", optional = true }
axdriver_display = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support", optional = true }
axdriver_pci = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support", optional = true }
axdriver_virtio = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support", optional = true }
```

**After:**
```toml
axdriver_base = { workspace = true }
axdriver_block = { workspace = true, optional = true }
axdriver_net = { workspace = true, optional = true }
axdriver_display = { workspace = true, optional = true }
axdriver_pci = { workspace = true, optional = true }
axdriver_virtio = { workspace = true, optional = true }
```

#### `modules/axnet/Cargo.toml`
**Before:**
```toml
axdriver_net = { git = "https://github.com/mingyang91/axdriver_crates.git", branch = "dwmac-support" }
```

**After:**
```toml
axdriver_net = { workspace = true }
```

#### `modules/axfs/Cargo.toml`
**Before:**
```toml
axdriver_block = { git = "https://github.com/arceos-org/axdriver_crates.git", tag = "v0.1.2" }
```

**After:**
```toml
axdriver_block = { workspace = true }
```

#### `modules/axdisplay/Cargo.toml`
**Before:**
```toml
axdriver_display = { git = "https://github.com/arceos-org/axdriver_crates.git", tag = "v0.1.2" }
```

**After:**
```toml
axdriver_display = { workspace = true }
```

## Benefits

1. **Faster Development**: No need to commit and push changes to test modifications
2. **Local Debugging**: Can modify driver code directly in `axdriver_crates/` folder
3. **Immediate Testing**: Changes are reflected immediately in builds
4. **Version Control**: Local changes can be tracked and committed when ready

## Usage

### Making Changes to DWMAC Driver

1. **Edit driver code directly:**
   ```bash
   # Edit the DWMAC driver implementation
   vim axdriver_crates/axdriver_net/src/dwmac.rs
   
   # Edit the HAL implementation  
   vim modules/axdriver/src/dwmac.rs
   ```

2. **Build and test immediately:**
   ```bash
   # Test specific package
   cargo check --package axdriver --features dwmac
   
   # Build full system
   make A=examples/httpserver PLATFORM=riscv64-starfive NET=y
   ```

3. **No git operations needed** for testing changes

### Reverting to Git Dependencies

If you need to revert to git dependencies (e.g., for production builds), simply:

1. Remove the axdriver_crates workspace members from `Cargo.toml`
2. Remove the axdriver_crates workspace dependencies
3. Restore the original git-based dependencies in each module

## File Structure

```
arceos/
├── Cargo.toml                    # Main workspace with local deps
├── axdriver_crates/              # Local driver crates
│   ├── axdriver_base/
│   ├── axdriver_net/            # DWMAC driver here
│   ├── axdriver_block/
│   ├── axdriver_display/
│   ├── axdriver_pci/
│   └── axdriver_virtio/
└── modules/
    ├── axdriver/                # HAL implementations
    ├── axnet/                   # Network stack
    ├── axfs/                    # File system
    └── axdisplay/               # Display
```

## Verification

The patch was verified by:
1. ✅ `cargo check --package axdriver --features dwmac` - Compiles successfully
2. ✅ `make A=examples/httpserver PLATFORM=riscv64-starfive NET=y` - Full build succeeds
3. ✅ All DWMAC driver features and debugging code accessible locally

## Notes

- The `axdriver_crates` folder contains a complete git repository with the DWMAC support branch
- Local changes should be committed to the local git repository when ready
- This setup is ideal for development and debugging phases
- Production builds may want to use tagged releases from git repositories 
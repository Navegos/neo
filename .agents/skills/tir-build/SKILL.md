---
name: build-tvm
description: Build TVM from the current worktree using CMake. Use when the user asks to compile, rebuild, or set up a local TVM build directory.
---

# Build TVM

Build TVM from the current worktree.

## Steps

1. Check that `build/` directory exists. If not, run initial setup:
   ```bash
   mkdir -p build
   cmake -S . -B build
   cmake --build build --parallel
   ```

2. If `build/` already exists, run incremental build:
   ```bash
   cmake --build build --parallel
   ```

3. Report success/failure and build time.

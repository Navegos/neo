# #0001 CVT1100 duplicate resource error when linking nbuild tests on Windows

- 2026-10-16T21:35:10Z `issue`: CVT1100 duplicate resource error when linking nbuild tests on Windows [crates/nbuild/tests/test_basic.rs]
- 2026-10-16T21:35:29Z `attempt`: Remove winres from crates/build.rs and use build metadata to eliminate CVT1100 duplicate PE resource conflict [crates/archinfo/build.rs] (worked)
- 2026-10-16T21:39:16Z `fix`: Eliminated duplicate winres PE resource conflict across crates, all 75 tests pass

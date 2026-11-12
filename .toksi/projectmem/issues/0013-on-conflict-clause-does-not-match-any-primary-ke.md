# #0013 ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash

- 2026-10-28T06:21:51Z `issue`: ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash [crates/tokless/src/icm/store.rs:77]
- 2026-10-28T06:23:16Z `fix`: Add UNIQUE constraint directly on summary_hash column in memories table

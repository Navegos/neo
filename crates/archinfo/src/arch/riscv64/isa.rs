// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/riscv64/isa.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Enum representing Instruction Set Architecture (ISA) extensions for the RISC-V 64-bit architecture.
///
/// These extensions define specific hardware capabilities or features that can be utilized during compilation.
/// Each variant corresponds to a specific feature or instruction set extension, enabling fine-grained control
/// over the compilation process for Riscv64 targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Riscv64ISA {
    /// No specific ISA extension.
    None,

//  Name                 Version      Description

    /// 'I' (Base Integer Instruction Set) (v2.1).
    I,                   // 2.1       'I' (Base Integer Instruction Set)

    /// 'E' (Embedded Instruction Set with 16 GPRs) (v2.0).
    E,                   // 2.0       'E' (Embedded Instruction Set with 16 GPRs)

    /// 'M' (Integer Multiplication and Division) (v2.0).
    M,                   // 2.0       'M' (Integer Multiplication and Division)

    /// 'A' (Atomic Instructions) (v2.1).
    A,                   // 2.1       'A' (Atomic Instructions)

    /// 'F' (Single-Precision Floating-Point) (v2.2).
    F,                   // 2.2       'F' (Single-Precision Floating-Point)

    /// 'D' (Double-Precision Floating-Point) (v2.2).
    D,                   // 2.2       'D' (Double-Precision Floating-Point)

    /// 'Q' (Quad-Precision Floating-Point) (v2.2).
    Q,                   // 2.2       'Q' (Quad-Precision Floating-Point)

    /// 'C' (Compressed Instructions) (v2.0).
    C,                   // 2.0       'C' (Compressed Instructions)

    /// 'B' (the collection of the Zba, Zbb, Zbs extensions) (v1.0).
    B,                   // 1.0       'B' (the collection of the Zba, Zbb, Zbs extensions)

    /// 'V' (Vector Extension for Application Processors) (v1.0).
    V,                   // 1.0       'V' (Vector Extension for Application Processors)

//  H,                   // 1.0       'H' (Hypervisor)

    /// 'Zic64b' (Cache Block Size Is 64 Bytes) (v1.0).
    Zic64b,              // 1.0       'Zic64b' (Cache Block Size Is 64 Bytes)

    /// 'Zicbom' (Cache-Block Management Instructions) (v1.0).
    Zicbom,              // 1.0       'Zicbom' (Cache-Block Management Instructions)

    /// 'Zicbop' (Cache-Block Prefetch Instructions) (v1.0).
    Zicbop,              // 1.0       'Zicbop' (Cache-Block Prefetch Instructions)

    /// 'Zicboz' (Cache-Block Zero Instructions) (v1.0).
    Zicboz,              // 1.0       'Zicboz' (Cache-Block Zero Instructions)

    /// 'Ziccamoa' (Main Memory Supports All Atomics in A) (v1.0).
    Ziccamoa,            // 1.0       'Ziccamoa' (Main Memory Supports All Atomics in A)

    /// 'Ziccamoc' (Main Memory Supports Atomics in Zacas) (v1.0).
    Ziccamoc,            // 1.0       'Ziccamoc' (Main Memory Supports Atomics in Zacas)

    /// 'Ziccid' (Instruction/Data Coherence and Consistency) (v1.0).
    Ziccid,              // 1.0       'Ziccid' (Instruction/Data Coherence and Consistency)

    /// 'Ziccif' (Main Memory Supports Instruction Fetch with Atomicity Requirement) (v1.0).
    Ziccif,              // 1.0       'Ziccif' (Main Memory Supports Instruction Fetch with Atomicity Requirement)

    /// 'Zicclsm' (Main Memory Supports Misaligned Loads/Stores) (v1.0).
    Zicclsm,             // 1.0       'Zicclsm' (Main Memory Supports Misaligned Loads/Stores)

    /// 'Ziccrse' (Main Memory Supports Forward Progress on LR/SC Sequences) (v1.0).
    Ziccrse,             // 1.0       'Ziccrse' (Main Memory Supports Forward Progress on LR/SC Sequences)

    /// 'Zicntr' (Base Counters and Timers) (v2.0).
    Zicntr,              // 2.0       'Zicntr' (Base Counters and Timers)

    /// 'Zicond' (Integer Conditional Operations) (v1.0).
    Zicond,              // 1.0       'Zicond' (Integer Conditional Operations)

    /// 'Zicsr' (CSRs) (v2.0).
    Zicsr,               // 2.0       'Zicsr' (CSRs)

    /// 'Zifencei' (fence.i) (v2.0).
    Zifencei,            // 2.0       'Zifencei' (fence.i)

    /// 'Zihintntl' (Non-Temporal Locality Hints) (v1.0).
    Zihintntl,           // 1.0       'Zihintntl' (Non-Temporal Locality Hints)

    /// 'Zihintpause' (Pause Hint) (v2.0).
    Zihintpause,         // 2.0       'Zihintpause' (Pause Hint)

    /// 'Zihpm' (Hardware Performance Counters) (v2.0).
    Zihpm,               // 2.0       'Zihpm' (Hardware Performance Counters)

    /// 'Zilsd' (Load/Store Pair Instructions) (v1.0).
    Zilsd,               // 1.0       'Zilsd' (Load/Store Pair Instructions)

    /// 'Zimop' (May-Be-Operations) (v1.0).
    Zimop,               // 1.0       'Zimop' (May-Be-Operations)

    /// 'Zmmul' (Integer Multiplication) (v1.0).
    Zmmul,               // 1.0       'Zmmul' (Integer Multiplication)

    /// 'Za128rs' (Reservation Set Size of at Most 128 Bytes) (v1.0).
    Za128rs,             // 1.0       'Za128rs' (Reservation Set Size of at Most 128 Bytes)

    /// 'Za64rs' (Reservation Set Size of at Most 64 Bytes) (v1.0).
    Za64rs,              // 1.0       'Za64rs' (Reservation Set Size of at Most 64 Bytes)

    /// 'Zaamo' (Atomic Memory Operations) (v1.0).
    Zaamo,               // 1.0       'Zaamo' (Atomic Memory Operations)

    /// 'Zabha' (Byte and Halfword Atomic Memory Operations) (v1.0).
    Zabha,               // 1.0       'Zabha' (Byte and Halfword Atomic Memory Operations)

    /// 'Zacas' (Atomic Compare-And-Swap Instructions) (v1.0).
    Zacas,               // 1.0       'Zacas' (Atomic Compare-And-Swap Instructions)

    /// 'Zalasr' (Load-Acquire and Store-Release Instructions) (v1.0).
    Zalasr,              // 1.0       'Zalasr' (Load-Acquire and Store-Release Instructions)

    /// 'Zalrsc' (Load-Reserved/Store-Conditional) (v1.0).
    Zalrsc,              // 1.0       'Zalrsc' (Load-Reserved/Store-Conditional)

    /// 'Zama16b' (Atomic 16-byte misaligned loads, stores and AMOs) (v1.0).
    Zama16b,             // 1.0       'Zama16b' (Atomic 16-byte misaligned loads, stores and AMOs)

    /// 'Zawrs' (Wait on Reservation Set) (v1.0).
    Zawrs,               // 1.0       'Zawrs' (Wait on Reservation Set)

    /// 'Zfa' (Additional Floating-Point) (v1.0).
    Zfa,                 // 1.0       'Zfa' (Additional Floating-Point)

    /// 'Zfbfmin' (Scalar BF16 Converts) (v1.0).
    Zfbfmin,             // 1.0       'Zfbfmin' (Scalar BF16 Converts)

    /// 'Zfh' (Half-Precision Floating-Point) (v1.0).
    Zfh,                 // 1.0       'Zfh' (Half-Precision Floating-Point)

    /// 'Zfhmin' (Half-Precision Floating-Point Minimal) (v1.0).
    Zfhmin,              // 1.0       'Zfhmin' (Half-Precision Floating-Point Minimal)

    /// 'Zfinx' (Float in Integer) (v1.0).
    Zfinx,               // 1.0       'Zfinx' (Float in Integer)

    /// 'Zdinx' (Double in Integer) (v1.0).
    Zdinx,               // 1.0       'Zdinx' (Double in Integer)

    /// 'Zca' (part of the C extension, excluding compressed floating point loads/stores) (v1.0).
    Zca,                 // 1.0       'Zca' (part of the C extension, excluding compressed floating point loads/stores)

    /// 'Zcb' (Compressed basic bit manipulation instructions) (v1.0).
    Zcb,                 // 1.0       'Zcb' (Compressed basic bit manipulation instructions)

    /// 'Zcd' (Compressed Double-Precision Floating-Point Instructions) (v1.0).
    Zcd,                 // 1.0       'Zcd' (Compressed Double-Precision Floating-Point Instructions)

    /// 'Zce' (Compressed extensions for microcontrollers) (v1.0).
    Zce,                 // 1.0       'Zce' (Compressed extensions for microcontrollers)

    /// 'Zcf' (Compressed Single-Precision Floating-Point Instructions) (v1.0).
    Zcf,                 // 1.0       'Zcf' (Compressed Single-Precision Floating-Point Instructions)

    /// 'Zclsd' (Compressed Load/Store Pair Instructions) (v1.0).
    Zclsd,               // 1.0       'Zclsd' (Compressed Load/Store Pair Instructions)

    /// 'Zcmop' (Compressed May-Be-Operations) (v1.0).
    Zcmop,               // 1.0       'Zcmop' (Compressed May-Be-Operations)

    /// 'Zcmp' (sequenced instructions for code-size reduction) (v1.0).
    Zcmp,                // 1.0       'Zcmp' (sequenced instructions for code-size reduction)

    /// 'Zcmt' (table jump instructions for code-size reduction) (v1.0).
    Zcmt,                // 1.0       'Zcmt' (table jump instructions for code-size reduction)

    /// 'Zba' (Address Generation Instructions) (v1.0).
    Zba,                 // 1.0       'Zba' (Address Generation Instructions)

    /// 'Zbb' (Basic Bit-Manipulation) (v1.0).
    Zbb,                 // 1.0       'Zbb' (Basic Bit-Manipulation)

    /// 'Zbc' (Carry-Less Multiplication) (v1.0).
    Zbc,                 // 1.0       'Zbc' (Carry-Less Multiplication)

    /// 'Zbkb' (Bitmanip instructions for Cryptography) (v1.0).
    Zbkb,                // 1.0       'Zbkb' (Bitmanip instructions for Cryptography)

    /// 'Zbkc' (Carry-less multiply instructions for Cryptography) (v1.0).
    Zbkc,                // 1.0       'Zbkc' (Carry-less multiply instructions for Cryptography)

    /// 'Zbkx' (Crossbar permutation instructions) (v1.0).
    Zbkx,                // 1.0       'Zbkx' (Crossbar permutation instructions)

    /// 'Zbs' (Single-Bit Instructions) (v1.0).
    Zbs,                 // 1.0       'Zbs' (Single-Bit Instructions)

    /// 'Zk' (Standard scalar cryptography extension) (v1.0).
    Zk,                  // 1.0       'Zk' (Standard scalar cryptography extension)

    /// 'Zkn' (NIST Algorithm Suite) (v1.0).
    Zkn,                 // 1.0       'Zkn' (NIST Algorithm Suite)

    /// 'Zknd' (NIST Suite: AES Decryption) (v1.0).
    Zknd,                // 1.0       'Zknd' (NIST Suite: AES Decryption)

    /// 'Zkne' (NIST Suite: AES Encryption) (v1.0).
    Zkne,                // 1.0       'Zkne' (NIST Suite: AES Encryption)

    /// 'Zknh' (NIST Suite: Hash Function Instructions) (v1.0).
    Zknh,                // 1.0       'Zknh' (NIST Suite: Hash Function Instructions)

    /// 'Zkr' (Entropy Source Extension) (v1.0).
    Zkr,                 // 1.0       'Zkr' (Entropy Source Extension)

    /// 'Zks' (ShangMi Algorithm Suite) (v1.0).
    Zks,                 // 1.0       'Zks' (ShangMi Algorithm Suite)

    /// 'Zksed' (ShangMi Suite: SM4 Block Cipher Instructions) (v1.0).
    Zksed,               // 1.0       'Zksed' (ShangMi Suite: SM4 Block Cipher Instructions)

    /// 'Zksh' (ShangMi Suite: SM3 Hash Function Instructions) (v1.0).
    Zksh,                // 1.0       'Zksh' (ShangMi Suite: SM3 Hash Function Instructions)

    /// 'Zkt' (Data Independent Execution Latency) (v1.0).
    Zkt,                 // 1.0       'Zkt' (Data Independent Execution Latency)

    /// 'Ztso' (Memory Model - Total Store Order) (v1.0).
    Ztso,                // 1.0       'Ztso' (Memory Model - Total Store Order)

    /// 'Zvbb' (Vector basic bit-manipulation instructions) (v1.0).
    Zvbb,                // 1.0       'Zvbb' (Vector basic bit-manipulation instructions)

    /// 'Zvbc' (Vector Carryless Multiplication) (v1.0).
    Zvbc,                // 1.0       'Zvbc' (Vector Carryless Multiplication)

    /// 'Zve32f' (Vector Extensions for Embedded Processors with maximal 32 EEW and F extension) (v1.0).
    Zve32f,              // 1.0       'Zve32f' (Vector Extensions for Embedded Processors with maximal 32 EEW and F extension)

    /// 'Zve32x' (Vector Extensions for Embedded Processors with maximal 32 EEW) (v1.0).
    Zve32x,              // 1.0       'Zve32x' (Vector Extensions for Embedded Processors with maximal 32 EEW)

    /// 'Zve64d' (Vector Extensions for Embedded Processors with maximal 64 EEW, F and D extension) (v1.0).
    Zve64d,              // 1.0       'Zve64d' (Vector Extensions for Embedded Processors with maximal 64 EEW, F and D extension)

    /// 'Zve64f' (Vector Extensions for Embedded Processors with maximal 64 EEW and F extension) (v1.0).
    Zve64f,              // 1.0       'Zve64f' (Vector Extensions for Embedded Processors with maximal 64 EEW and F extension)

    /// 'Zve64x' (Vector Extensions for Embedded Processors with maximal 64 EEW) (v1.0).
    Zve64x,              // 1.0       'Zve64x' (Vector Extensions for Embedded Processors with maximal 64 EEW)

    /// 'Zvfbfmin' (Vector BF16 Converts) (v1.0).
    Zvfbfmin,            // 1.0       'Zvfbfmin' (Vector BF16 Converts)

    /// 'Zvfbfwma' (Vector BF16 widening mul-add) (v1.0).
    Zvfbfwma,            // 1.0       'Zvfbfwma' (Vector BF16 widening mul-add)

    /// 'Zvfh' (Vector Half-Precision Floating-Point) (v1.0).
    Zvfh,                // 1.0       'Zvfh' (Vector Half-Precision Floating-Point)

    /// 'Zvfhmin' (Vector Half-Precision Floating-Point Minimal) (v1.0).
    Zvfhmin,             // 1.0       'Zvfhmin' (Vector Half-Precision Floating-Point Minimal)

    /// 'Zvkb' (Vector Bit-manipulation used in Cryptography) (v1.0).
    Zvkb,                // 1.0       'Zvkb' (Vector Bit-manipulation used in Cryptography)

    /// 'Zvkg' (Vector GCM instructions for Cryptography) (v1.0).
    Zvkg,                // 1.0       'Zvkg' (Vector GCM instructions for Cryptography)

    /// 'Zvkn' (shorthand for 'Zvkned', 'Zvknhb', 'Zvkb', and 'Zvkt') (v1.0).
    Zvkn,                // 1.0       'Zvkn' (shorthand for 'Zvkned', 'Zvknhb', 'Zvkb', and 'Zvkt')

    /// 'Zvknc' (shorthand for 'Zvknc' and 'Zvbc') (v1.0).
    Zvknc,               // 1.0       'Zvknc' (shorthand for 'Zvknc' and 'Zvbc')

    /// 'Zvkned' (Vector AES Encryption & Decryption (Single Round)) (v1.0).
    Zvkned,              // 1.0       'Zvkned' (Vector AES Encryption & Decryption (Single Round))

    /// 'Zvkng' (shorthand for 'Zvkn' and 'Zvkg') (v1.0).
    Zvkng,               // 1.0       'Zvkng' (shorthand for 'Zvkn' and 'Zvkg')

    /// 'Zvknha' (Vector SHA-2 (SHA-256 only)) (v1.0).
    Zvknha,              // 1.0       'Zvknha' (Vector SHA-2 (SHA-256 only))

    /// 'Zvknhb' (Vector SHA-2 (SHA-256 and SHA-512)) (v1.0).
    Zvknhb,              // 1.0       'Zvknhb' (Vector SHA-2 (SHA-256 and SHA-512))

    /// 'Zvks' (shorthand for 'Zvksed', 'Zvksh', 'Zvkb', and 'Zvkt') (v1.0).
    Zvks,                // 1.0       'Zvks' (shorthand for 'Zvksed', 'Zvksh', 'Zvkb', and 'Zvkt')

    /// 'Zvksc' (shorthand for 'Zvks' and 'Zvbc') (v1.0).
    Zvksc,               // 1.0       'Zvksc' (shorthand for 'Zvks' and 'Zvbc')

    /// 'Zvksed' (SM4 Block Cipher Instructions) (v1.0).
    Zvksed,              // 1.0       'Zvksed' (SM4 Block Cipher Instructions)

    /// 'Zvksg' (shorthand for 'Zvks' and 'Zvkg') (v1.0).
    Zvksg,               // 1.0       'Zvksg' (shorthand for 'Zvks' and 'Zvkg')

    /// 'Zvksh' (SM3 Hash Function Instructions) (v1.0).
    Zvksh,               // 1.0       'Zvksh' (SM3 Hash Function Instructions)

    /// 'Zvkt' (Vector Data-Independent Execution Latency) (v1.0).
    Zvkt,                // 1.0       'Zvkt' (Vector Data-Independent Execution Latency)

    /// 'Zvl1024b' (Minimum Vector Length 1024) (v1.0).
    Zvl1024b,            // 1.0       'Zvl1024b' (Minimum Vector Length 1024)

    /// 'Zvl128b' (Minimum Vector Length 128) (v1.0).
    Zvl128b,             // 1.0       'Zvl128b' (Minimum Vector Length 128)

    /// 'Zvl16384b' (Minimum Vector Length 16384) (v1.0).
    Zvl16384b,           // 1.0       'Zvl16384b' (Minimum Vector Length 16384)

    /// 'Zvl2048b' (Minimum Vector Length 2048) (v1.0).
    Zvl2048b,            // 1.0       'Zvl2048b' (Minimum Vector Length 2048)

    /// 'Zvl256b' (Minimum Vector Length 256) (v1.0).
    Zvl256b,             // 1.0       'Zvl256b' (Minimum Vector Length 256)

    /// 'Zvl32768b' (Minimum Vector Length 32768) (v1.0).
    Zvl32768b,           // 1.0       'Zvl32768b' (Minimum Vector Length 32768)

    /// 'Zvl32b' (Minimum Vector Length 32) (v1.0).
    Zvl32b,              // 1.0       'Zvl32b' (Minimum Vector Length 32)

    /// 'Zvl4096b' (Minimum Vector Length 4096) (v1.0).
    Zvl4096b,            // 1.0       'Zvl4096b' (Minimum Vector Length 4096)

    /// 'Zvl512b' (Minimum Vector Length 512) (v1.0).
    Zvl512b,             // 1.0       'Zvl512b' (Minimum Vector Length 512)

    /// 'Zvl64b' (Minimum Vector Length 64) (v1.0).
    Zvl64b,              // 1.0       'Zvl64b' (Minimum Vector Length 64)

    /// 'Zvl65536b' (Minimum Vector Length 65536) (v1.0).
    Zvl65536b,           // 1.0       'Zvl65536b' (Minimum Vector Length 65536)

    /// 'Zvl8192b' (Minimum Vector Length 8192) (v1.0).
    Zvl8192b,            // 1.0       'Zvl8192b' (Minimum Vector Length 8192)

    /// 'Zhinx' (Half Float in Integer) (v1.0).
    Zhinx,               // 1.0       'Zhinx' (Half Float in Integer)

    /// 'Zhinxmin' (Half Float in Integer Minimal) (v1.0).
    Zhinxmin,            // 1.0       'Zhinxmin' (Half Float in Integer Minimal)

//  Sdext,               // 1.0       'Sdext' (External debugger)
//  Sdtrig,              // 1.0       'Sdtrig' (Debugger triggers)
//  Sha,                 // 1.0       'Sha' (Augmented Hypervisor)
//  Shcounterenw,        // 1.0       'Shcounterenw' (Support writeable hcounteren enable bit for any hpmcounter that is not read-only zero)
//  Shgatpa,             // 1.0       'Shgatpa' (SvNNx4 mode supported for all modes supported by satp, as well as Bare)
//  Shlcofideleg,        // 1.0       'Shlcofideleg' (Delegating LCOFI Interrupts to VS-mode)
//  Shtvala,             // 1.0       'Shtvala' (htval provides all needed values)
//  Shvsatpa,            // 1.0       'Shvsatpa' (vsatp supports all modes supported by satp)
//  Shvstvala,           // 1.0       'Shvstvala' (vstval provides all needed values)
//  Shvstvecd,           // 1.0       'Shvstvecd' (vstvec supports Direct mode)
//  Smaia,               // 1.0       'Smaia' (Advanced Interrupt Architecture Machine Level)
//  Smcdeleg,            // 1.0       'Smcdeleg' (Counter Delegation Machine Level)
//  Smcntrpmf,           // 1.0       'Smcntrpmf' (Cycle and Instret Privilege Mode Filtering)
//  Smcsrind,            // 1.0       'Smcsrind' (Indirect CSR Access Machine Level)
//  Smctr,               // 1.0       'Smctr' (Control Transfer Records Machine Level)
//  Smdbltrp,            // 1.0       'Smdbltrp' (Double Trap Machine Level)
//  Smepmp,              // 1.0       'Smepmp' (Enhanced Physical Memory Protection)
//  Smmpm,               // 1.0       'Smmpm' (Machine-level Pointer Masking for M-mode)
//  Smnpm,               // 1.0       'Smnpm' (Machine-level Pointer Masking for next lower privilege mode)
//  Smrnmi,              // 1.0       'Smrnmi' (Resumable Non-Maskable Interrupts)
//  Smstateen,           // 1.0       'Smstateen' (Machine-mode view of the state-enable extension)
//  Ssaia,               // 1.0       'Ssaia' (Advanced Interrupt Architecture Supervisor Level)
//  Ssccfg,              // 1.0       'Ssccfg' (Counter Configuration Supervisor Level)
//  Ssccptr,             // 1.0       'Ssccptr' (Main memory supports page table reads)
//  Sscofpmf,            // 1.0       'Sscofpmf' (Count Overflow and Mode-Based Filtering)
//  Sscounterenw,        // 1.0       'Sscounterenw' (Support writeable scounteren enable bit for any hpmcounter that is not read-only zero)
//  Sscsrind,            // 1.0       'Sscsrind' (Indirect CSR Access Supervisor Level)
//  Ssctr,               // 1.0       'Ssctr' (Control Transfer Records Supervisor Level)
//  Ssdbltrp,            // 1.0       'Ssdbltrp' (Double Trap Supervisor Level)
//  Ssnpm,               // 1.0       'Ssnpm' (Supervisor-level Pointer Masking for next lower privilege mode)
//  Sspm,                // 1.0       'Sspm' (Indicates Supervisor-mode Pointer Masking)
//  Ssqosid,             // 1.0       'Ssqosid' (Quality-of-Service (QoS) Identifiers)
//  Ssstateen,           // 1.0       'Ssstateen' (Supervisor-mode view of the state-enable extension)
//  Ssstrict,            // 1.0       'Ssstrict' (No non-conforming extensions are present)
//  Sstc,                // 1.0       'Sstc' (Supervisor-mode timer interrupts)
//  Sstvala,             // 1.0       'Sstvala' (stval provides all needed values)
//  Sstvecd,             // 1.0       'Sstvecd' (stvec supports Direct mode)
//  Ssu64xl,             // 1.0       'Ssu64xl' (UXLEN=64 supported)

    /// 'Supm' (Indicates User-mode Pointer Masking) (v1.0).
    Supm,                // 1.0       'Supm' (Indicates User-mode Pointer Masking)

//  Svade,               // 1.0       'Svade' (Raise exceptions on improper A/D bits)
//  Svadu,               // 1.0       'Svadu' (Hardware A/D updates)
//  Svbare,              // 1.0       'Svbare' (satp mode Bare supported)
//  Svinval,             // 1.0       'Svinval' (Fine-Grained Address-Translation Cache Invalidation)
//  Svnapot,             // 1.0       'Svnapot' (NAPOT Translation Contiguity)
//  Svpbmt,              // 1.0       'Svpbmt' (Page-Based Memory Types)
//  Svrsw60t59b,         // 1.0       'Svrsw60t59b' (PTE Reserved-for-Software Bits 60-59)
//  Svvptc,              // 1.0       'Svvptc' (Obviating Memory-Management Instructions after Marking PTEs Valid)

    /// 'XAIFET' (AI Foundry ET Extension) (v1.0).
    Xaifet,              // 1.0       'XAIFET' (AI Foundry ET Extension)

    /// 'XAndesBFHCvt' (Andes Scalar BFLOAT16 Conversion Extension) (v5.0).
    Xandesbfhcvt,        // 5.0       'XAndesBFHCvt' (Andes Scalar BFLOAT16 Conversion Extension)

    /// 'XAndesPerf' (Andes Performance Extension) (v5.0).
    Xandesperf,          // 5.0       'XAndesPerf' (Andes Performance Extension)

    /// 'XAndesVBFHCvt' (Andes Vector BFLOAT16 Conversion Extension) (v5.0).
    Xandesvbfhcvt,       // 5.0       'XAndesVBFHCvt' (Andes Vector BFLOAT16 Conversion Extension)

    /// 'XAndesVDot' (Andes Vector Dot Product Extension) (v5.0).
    Xandesvdot,          // 5.0       'XAndesVDot' (Andes Vector Dot Product Extension)

    /// 'XAndesVPackFPH' (Andes Vector Packed FP16 Extension) (v5.0).
    Xandesvpackfph,      // 5.0       'XAndesVPackFPH' (Andes Vector Packed FP16 Extension)

    /// 'XAndesVSIntH' (Andes Vector Small INT Handling Extension) (v5.0).
    Xandesvsinth,        // 5.0       'XAndesVSIntH' (Andes Vector Small INT Handling Extension)

    /// 'XAndesVSIntLoad' (Andes Vector INT4 Load Extension) (v5.0).
    Xandesvsintload,     // 5.0       'XAndesVSIntLoad' (Andes Vector INT4 Load Extension)

    /// 'XCheriot' (CHERIoT extension) (v1.0).
    Xcheriot,            // 1.0       'XCheriot' (CHERIoT extension)

    /// 'XCValu' (CORE-V ALU Operations) (v1.0).
    Xcvalu,              // 1.0       'XCValu' (CORE-V ALU Operations)

    /// 'XCVbi' (CORE-V Immediate Branching) (v1.0).
    Xcvbi,               // 1.0       'XCVbi' (CORE-V Immediate Branching)

    /// 'XCVbitmanip' (CORE-V Bit Manipulation) (v1.0).
    Xcvbitmanip,         // 1.0       'XCVbitmanip' (CORE-V Bit Manipulation)

    /// 'XCVelw' (CORE-V Event Load Word) (v1.0).
    Xcvelw,              // 1.0       'XCVelw' (CORE-V Event Load Word)

    /// 'XCVmac' (CORE-V Multiply-Accumulate) (v1.0).
    Xcvmac,              // 1.0       'XCVmac' (CORE-V Multiply-Accumulate)

    /// 'XCVmem' (CORE-V Post-incrementing Load & Store) (v1.0).
    Xcvmem,              // 1.0       'XCVmem' (CORE-V Post-incrementing Load & Store)

    /// 'XCVsimd' (CORE-V SIMD ALU) (v1.0).
    Xcvsimd,             // 1.0       'XCVsimd' (CORE-V SIMD ALU)

    /// 'XMIPSCBOP' (MIPS Software Prefetch) (v1.0).
    Xmipscbop,           // 1.0       'XMIPSCBOP' (MIPS Software Prefetch)

    /// 'XMIPSCMov' (MIPS conditional move instruction (mips.ccmov)) (v1.0).
    Xmipscmov,           // 1.0       'XMIPSCMov' (MIPS conditional move instruction (mips.ccmov))

    /// 'XMIPSEXECTL' (MIPS execution control) (v1.0).
    Xmipsexectl,         // 1.0       'XMIPSEXECTL' (MIPS execution control)

    /// 'XMIPSLSP' (MIPS optimization for hardware load-store bonding) (v1.0).
    Xmipslsp,            // 1.0       'XMIPSLSP' (MIPS optimization for hardware load-store bonding)

    /// 'Xqccmp' (Qualcomm 16-bit Push/Pop and Double Moves) (v0.3).
    Xqccmp,              // 0.3       'Xqccmp' (Qualcomm 16-bit Push/Pop and Double Moves)

    /// 'Xqci' (Qualcomm uC Extension) (v0.13).
    Xqci,                // 0.13      'Xqci' (Qualcomm uC Extension)

    /// 'Xqcia' (Qualcomm uC Arithmetic Extension) (v0.7).
    Xqcia,               // 0.7       'Xqcia' (Qualcomm uC Arithmetic Extension)

    /// 'Xqciac' (Qualcomm uC Load-Store Address Calculation Extension) (v0.3).
    Xqciac,              // 0.3       'Xqciac' (Qualcomm uC Load-Store Address Calculation Extension)

    /// 'Xqcibi' (Qualcomm uC Branch Immediate Extension) (v0.2).
    Xqcibi,              // 0.2       'Xqcibi' (Qualcomm uC Branch Immediate Extension)

    /// 'Xqcibm' (Qualcomm uC Bit Manipulation Extension) (v0.8).
    Xqcibm,              // 0.8       'Xqcibm' (Qualcomm uC Bit Manipulation Extension)

    /// 'Xqcicli' (Qualcomm uC Conditional Load Immediate Extension) (v0.3).
    Xqcicli,             // 0.3       'Xqcicli' (Qualcomm uC Conditional Load Immediate Extension)

    /// 'Xqcicm' (Qualcomm uC Conditional Move Extension) (v0.2).
    Xqcicm,              // 0.2       'Xqcicm' (Qualcomm uC Conditional Move Extension)

    /// 'Xqcics' (Qualcomm uC Conditional Select Extension) (v0.2).
    Xqcics,              // 0.2       'Xqcics' (Qualcomm uC Conditional Select Extension)

    /// 'Xqcicsr' (Qualcomm uC CSR Extension) (v0.4).
    Xqcicsr,             // 0.4       'Xqcicsr' (Qualcomm uC CSR Extension)

    /// 'Xqciint' (Qualcomm uC Interrupts Extension) (v0.10).
    Xqciint,             // 0.10      'Xqciint' (Qualcomm uC Interrupts Extension)

    /// 'Xqciio' (Qualcomm uC External Input Output Extension) (v0.1).
    Xqciio,              // 0.1       'Xqciio' (Qualcomm uC External Input Output Extension)

    /// 'Xqcilb' (Qualcomm uC Long Branch Extension) (v0.2).
    Xqcilb,              // 0.2       'Xqcilb' (Qualcomm uC Long Branch Extension)

    /// 'Xqcili' (Qualcomm uC Load Large Immediate Extension) (v0.2).
    Xqcili,              // 0.2       'Xqcili' (Qualcomm uC Load Large Immediate Extension)

    /// 'Xqcilia' (Qualcomm uC Large Immediate Arithmetic Extension) (v0.2).
    Xqcilia,             // 0.2       'Xqcilia' (Qualcomm uC Large Immediate Arithmetic Extension)

    /// 'Xqcilo' (Qualcomm uC Large Offset Load Store Extension) (v0.3).
    Xqcilo,              // 0.3       'Xqcilo' (Qualcomm uC Large Offset Load Store Extension)

    /// 'Xqcilsm' (Qualcomm uC Load Store Multiple Extension) (v0.6).
    Xqcilsm,             // 0.6       'Xqcilsm' (Qualcomm uC Load Store Multiple Extension)

    /// 'Xqcisim' (Qualcomm uC Simulation Hint Extension) (v0.2).
    Xqcisim,             // 0.2       'Xqcisim' (Qualcomm uC Simulation Hint Extension)

    /// 'Xqcisls' (Qualcomm uC Scaled Load Store Extension) (v0.2).
    Xqcisls,             // 0.2       'Xqcisls' (Qualcomm uC Scaled Load Store Extension)

    /// 'Xqcisync' (Qualcomm uC Sync Delay Extension) (v0.3).
    Xqcisync,            // 0.3       'Xqcisync' (Qualcomm uC Sync Delay Extension)

    /// 'XSfcease' (SiFive sf.cease Instruction) (v1.0).
    Xsfcease,            // 1.0       'XSfcease' (SiFive sf.cease Instruction)

    /// 'XSfmm128t' (TE=128 configuration) (v0.6).
    Xsfmm128t,           // 0.6       'XSfmm128t' (TE=128 configuration)

    /// 'XSfmm16t' (TE=16 configuration) (v0.6).
    Xsfmm16t,            // 0.6       'XSfmm16t' (TE=16 configuration)

    /// 'XSfmm32a' (TEW=32-bit accumulation operands - int: 8b; float: fp16, bf16, fp32) (v0.6).
    Xsfmm32a,            // 0.6       'XSfmm32a' (TEW=32-bit accumulation operands - int: 8b; float: fp16, bf16, fp32)

    /// 'XSfmm32a16f' (TEW=32-bit accumulation, operands - float: 16b, widen=2 (IEEE, BF)) (v0.6).
    Xsfmm32a16f,         // 0.6       'XSfmm32a16f' (TEW=32-bit accumulation, operands - float: 16b, widen=2 (IEEE, BF))

    /// 'XSfmm32a32f' (TEW=32-bit accumulation, operands - float: 32b) (v0.6).
    Xsfmm32a32f,         // 0.6       'XSfmm32a32f' (TEW=32-bit accumulation, operands - float: 32b)

    /// 'XSfmm32a8f' (TEW=32-bit accumulation, operands - float: fp8) (v0.6).
    Xsfmm32a8f,          // 0.6       'XSfmm32a8f' (TEW=32-bit accumulation, operands - float: fp8)

    /// 'XSfmm32a8i' (TEW=32-bit accumulation, operands - int: 8b) (v0.6).
    Xsfmm32a8i,          // 0.6       'XSfmm32a8i' (TEW=32-bit accumulation, operands - int: 8b)

    /// 'XSfmm32t' (TE=32 configuration) (v0.6).
    Xsfmm32t,            // 0.6       'XSfmm32t' (TE=32 configuration)

    /// 'XSfmm64a64f' (TEW=64-bit accumulation, operands - float: fp64) (v0.6).
    Xsfmm64a64f,         // 0.6       'XSfmm64a64f' (TEW=64-bit accumulation, operands - float: fp64)

    /// 'XSfmm64t' (TE=64 configuration) (v0.6).
    Xsfmm64t,            // 0.6       'XSfmm64t' (TE=64 configuration)

    /// 'XSfmmbase' (All non arithmetic instructions for all TEWs and sf.vtzero) (v0.6).
    Xsfmmbase,           // 0.6       'XSfmmbase' (All non arithmetic instructions for all TEWs and sf.vtzero)

    /// 'XSfvcp' (SiFive Custom Vector Coprocessor Interface Instructions) (v1.0).
    Xsfvcp,              // 1.0       'XSfvcp' (SiFive Custom Vector Coprocessor Interface Instructions)

    /// 'XSfvfbfexp16e' (SiFive Vector Floating-Point Exponential Function Instruction, BFloat16) (v0.5).
    Xsfvfbfexp16e,       // 0.5       'XSfvfbfexp16e' (SiFive Vector Floating-Point Exponential Function Instruction, BFloat16)

    /// 'XSfvfexp16e' (SiFive Vector Floating-Point Exponential Function Instruction, Half Precision) (v0.5).
    Xsfvfexp16e,         // 0.5       'XSfvfexp16e' (SiFive Vector Floating-Point Exponential Function Instruction, Half Precision)

    /// 'XSfvfexp32e' (SiFive Vector Floating-Point Exponential Function Instruction, Single Precision) (v0.5).
    Xsfvfexp32e,         // 0.5       'XSfvfexp32e' (SiFive Vector Floating-Point Exponential Function Instruction, Single Precision)

    /// 'XSfvfexpa' (SiFive Vector Floating-Point Exponential Approximation Instruction) (v0.2).
    Xsfvfexpa,           // 0.2       'XSfvfexpa' (SiFive Vector Floating-Point Exponential Approximation Instruction)

    /// 'XSfvfexpa64e' (SiFive Vector Floating-Point Exponential Approximation Instruction with Double-Precision) (v0.2).
    Xsfvfexpa64e,        // 0.2       'XSfvfexpa64e' (SiFive Vector Floating-Point Exponential Approximation Instruction with Double-Precision)

    /// 'XSfvfnrclipxfqf' (SiFive FP32-to-int8 Ranged Clip Instructions) (v1.0).
    Xsfvfnrclipxfqf,     // 1.0       'XSfvfnrclipxfqf' (SiFive FP32-to-int8 Ranged Clip Instructions)

    /// 'XSfvfwmaccqqq' (SiFive Matrix Multiply Accumulate Instruction (4-by-4)) (v1.0).
    Xsfvfwmaccqqq,       // 1.0       'XSfvfwmaccqqq' (SiFive Matrix Multiply Accumulate Instruction (4-by-4))

    /// 'XSfvqmaccdod' (SiFive Int8 Matrix Multiplication Instructions (2-by-8 and 8-by-2)) (v1.0).
    Xsfvqmaccdod,        // 1.0       'XSfvqmaccdod' (SiFive Int8 Matrix Multiplication Instructions (2-by-8 and 8-by-2))

    /// 'XSfvqmaccqoq' (SiFive Int8 Matrix Multiplication Instructions (4-by-8 and 8-by-4)) (v1.0).
    Xsfvqmaccqoq,        // 1.0       'XSfvqmaccqoq' (SiFive Int8 Matrix Multiplication Instructions (4-by-8 and 8-by-4))

    /// 'XSiFivecdiscarddlone' (SiFive sf.cdiscard.d.l1 Instruction) (v1.0).
    Xsifivecdiscarddlone,// 1.0       'XSiFivecdiscarddlone' (SiFive sf.cdiscard.d.l1 Instruction)

    /// 'XSiFivecflushdlone' (SiFive sf.cflush.d.l1 Instruction) (v1.0).
    Xsifivecflushdlone,  // 1.0       'XSiFivecflushdlone' (SiFive sf.cflush.d.l1 Instruction)

    /// 'XSMTVDot' (SpacemiT Vector Dot Product Extension) (v1.0).
    Xsmtvdot,            // 1.0       'XSMTVDot' (SpacemiT Vector Dot Product Extension)

    /// 'XSMTVDotII' (SpacemiT Vector Extension for Matrix 2.0) (v1.0).
    Xsmtvdotii,          // 1.0       'XSMTVDotII' (SpacemiT Vector Extension for Matrix 2.0)

    /// 'XTHeadBa' (T-Head address calculation instructions) (v1.0).
    Xtheadba,            // 1.0       'XTHeadBa' (T-Head address calculation instructions)

    /// 'XTHeadBb' (T-Head basic bit-manipulation instructions) (v1.0).
    Xtheadbb,            // 1.0       'XTHeadBb' (T-Head basic bit-manipulation instructions)

    /// 'XTHeadBs' (T-Head single-bit instructions) (v1.0).
    Xtheadbs,            // 1.0       'XTHeadBs' (T-Head single-bit instructions)

    /// 'XTHeadCmo' (T-Head cache management instructions) (v1.0).
    Xtheadcmo,           // 1.0       'XTHeadCmo' (T-Head cache management instructions)

    /// 'XTHeadCondMov' (T-Head conditional move instructions) (v1.0).
    Xtheadcondmov,       // 1.0       'XTHeadCondMov' (T-Head conditional move instructions)

    /// 'XTHeadFMemIdx' (T-Head FP Indexed Memory Operations) (v1.0).
    Xtheadfmemidx,       // 1.0       'XTHeadFMemIdx' (T-Head FP Indexed Memory Operations)

    /// 'XTHeadMac' (T-Head Multiply-Accumulate Instructions) (v1.0).
    Xtheadmac,           // 1.0       'XTHeadMac' (T-Head Multiply-Accumulate Instructions)

    /// 'XTHeadMemIdx' (T-Head Indexed Memory Operations) (v1.0).
    Xtheadmemidx,        // 1.0       'XTHeadMemIdx' (T-Head Indexed Memory Operations)

    /// 'XTHeadMemPair' (T-Head two-GPR Memory Operations) (v1.0).
    Xtheadmempair,       // 1.0       'XTHeadMemPair' (T-Head two-GPR Memory Operations)

    /// 'XTHeadSync' (T-Head multicore synchronization instructions) (v1.0).
    Xtheadsync,          // 1.0       'XTHeadSync' (T-Head multicore synchronization instructions)

    /// 'XTHeadVdot' (T-Head Vector Extensions for Dot) (v1.0).
    Xtheadvdot,          // 1.0       'XTHeadVdot' (T-Head Vector Extensions for Dot)

    /// 'XVentanaCondOps' (Ventana Conditional Ops) (v1.0).
    Xventanacondops,     // 1.0       'XVentanaCondOps' (Ventana Conditional Ops)

    /// 'Xwchc' (WCH/QingKe additional compressed opcodes) (v2.2).
    Xwchc,               // 2.2       'Xwchc' (WCH/QingKe additional compressed opcodes)

//  Experimental extensions & every extensions bellow before profiles requires in clang -menable-experimental-extensions

    /// 'P' ('Base P' (Packed SIMD)) (v0.21). Requires `-menable-experimental-extensions`.
    P,                   // 0.21      'P' ('Base P' (Packed SIMD))

    /// 'Y' ('Base Y' (CHERI)) (v0.98). Requires `-menable-experimental-extensions`.
    Y,                   // 0.98      'Y' ('Base Y' (CHERI))

    /// 'Zibi' (Branch with Immediate) (v0.1). Requires `-menable-experimental-extensions`.
    Zibi,                // 0.1       'Zibi' (Branch with Immediate)

    /// 'Zicfilp' (Landing pad) (v1.0). Requires `-menable-experimental-extensions`.
    Zicfilp,             // 1.0       'Zicfilp' (Landing pad)

    /// 'Zicfiss' (Shadow stack) (v1.0). Requires `-menable-experimental-extensions`.
    Zicfiss,             // 1.0       'Zicfiss' (Shadow stack)

    /// 'Zvabd' (Vector Absolute Difference) (v0.7). Requires `-menable-experimental-extensions`.
    Zvabd,               // 0.7       'Zvabd' (Vector Absolute Difference)

    /// 'Zvbc32e' (Vector Carryless Multiplication with 32-bits elements) (v0.7). Requires `-menable-experimental-extensions`.
    Zvbc32e,             // 0.7       'Zvbc32e' (Vector Carryless Multiplication with 32-bits elements)

    /// 'Zvdot4a8i' (Vector 4-element Dot Product of packed 8-bit Integers) (v0.1). Requires `-menable-experimental-extensions`.
    Zvdot4a8i,           // 0.1       'Zvdot4a8i' (Vector 4-element Dot Product of packed 8-bit Integers)

    /// 'Zvfbdota32f' (FP32 batched dot-product extension) (v0.2). Requires `-menable-experimental-extensions`.
    Zvfbdota32f,         // 0.2       'Zvfbdota32f' (FP32 batched dot-product extension)

    /// 'Zvfbfa' (Additional BF16 vector compute support) (v0.1). Requires `-menable-experimental-extensions`.
    Zvfbfa,              // 0.1       'Zvfbfa' (Additional BF16 vector compute support)

    /// 'Zvfofp8min' (Vector OFP8 Converts) (v0.2). Requires `-menable-experimental-extensions`.
    Zvfofp8min,          // 0.2       'Zvfofp8min' (Vector OFP8 Converts)

    /// 'Zvfqwbdota8f' (OCP FP8 batched dot-product extension) (v0.2). Requires `-menable-experimental-extensions`.
    Zvfqwbdota8f,        // 0.2       'Zvfqwbdota8f' (OCP FP8 batched dot-product extension)

    /// 'Zvfqwdota8f' (OCP FP8 Dot-Product) (v0.2). Requires `-menable-experimental-extensions`.
    Zvfqwdota8f,         // 0.2       'Zvfqwdota8f' (OCP FP8 Dot-Product)

    /// 'Zvfwbdota16bf' (BF16 batched dot-product extension) (v0.2). Requires `-menable-experimental-extensions`.
    Zvfwbdota16bf,       // 0.2       'Zvfwbdota16bf' (BF16 batched dot-product extension)

    /// 'Zvfwdota16bf' (BF16 Dot-Product) (v0.2). Requires `-menable-experimental-extensions`.
    Zvfwdota16bf,        // 0.2       'Zvfwdota16bf' (BF16 Dot-Product)

    /// 'Zvkgs' (Vector-Scalar GCM instructions for Cryptography) (v0.7). Requires `-menable-experimental-extensions`.
    Zvkgs,               // 0.7       'Zvkgs' (Vector-Scalar GCM instructions for Cryptography)

    /// 'Zvqwbdota16i' (16-bit integer batched dot-product extension) (v0.2). Requires `-menable-experimental-extensions`.
    Zvqwbdota16i,        // 0.2       'Zvqwbdota16i' (16-bit integer batched dot-product extension)

    /// 'Zvqwbdota8i' (8-bit integer batched dot-product extension) (v0.2). Requires `-menable-experimental-extensions`.
    Zvqwbdota8i,         // 0.2       'Zvqwbdota8i' (8-bit integer batched dot-product extension)

    /// 'Zvqwdota16i' (16-bit Integer Dot-Product) (v0.2). Requires `-menable-experimental-extensions`.
    Zvqwdota16i,         // 0.2       'Zvqwdota16i' (16-bit Integer Dot-Product)

    /// 'Zvqwdota8i' (8-bit Integer Dot-Product) (v0.2). Requires `-menable-experimental-extensions`.
    Zvqwdota8i,          // 0.2       'Zvqwdota8i' (8-bit Integer Dot-Product)

    /// 'Zvvfmm' (Floating-Point Matrix Multiply-Accumulate) (v0.1). Requires `-menable-experimental-extensions`.
    Zvvfmm,              // 0.1       'Zvvfmm' (Floating-Point Matrix Multiply-Accumulate)

    /// 'Zvvmm' (Integer Matrix Multiply-Accumulate) (v0.1). Requires `-menable-experimental-extensions`.
    Zvvmm,               // 0.1       'Zvvmm' (Integer Matrix Multiply-Accumulate)

    /// 'Zvvmtls' (Matrix Tile Load/Store) (v0.1). Requires `-menable-experimental-extensions`.
    Zvvmtls,             // 0.1       'Zvvmtls' (Matrix Tile Load/Store)

    /// 'Zvvmttls' (Transposing Matrix Tile Load/Store) (v0.1). Requires `-menable-experimental-extensions`.
    Zvvmttls,            // 0.1       'Zvvmttls' (Transposing Matrix Tile Load/Store)

    /// 'Zvzip' (Vector Reordering Structured Data) (v0.1). Requires `-menable-experimental-extensions`.
    Zvzip,               // 0.1       'Zvzip' (Vector Reordering Structured Data)

//  Smpmpmt,             // 0.6       'Smpmpmt' (PMP-based Memory Types Extension)
//  Svukte,              // 0.3       'Svukte' (Address-Independent Latency of User-Mode Faults to Supervisor Addresses)

    /// 'Xqccmt' (Qualcomm 16-bit Table Jump) (v0.1). Requires `-menable-experimental-extensions`.
    Xqccmt,              // 0.1       'Xqccmt' (Qualcomm 16-bit Table Jump)

//  Xsfmclic,            // 0.1       'XSfmclic' (SiFive CLIC Machine-mode CSRs)
//  Xsfsclic,            // 0.1       'XSfsclic' (SiFive CLIC Supervisor-mode CSRs)

//  Supported Profiles
//  Rva20s64,       // rv64i_m_a_f_d_c_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicsr_zifencei_zmmul_za128rs_zaamo_zalrsc_zca_zcd_ssccptr_sstvala_sstvecd_svade_svbare

    /// RVA20U64 User-mode Profile.
    Rva20u64,       // rv64i_m_a_f_d_c_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicsr_zmmul_za128rs_zaamo_zalrsc_zca_zcd

//  Rva22s64,       // rv64i_m_a_f_d_c_b_zic64b_zicbom_zicbop_zicboz_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicsr_zifencei_zihintpause_zihpm_zmmul_za64rs_zaamo_zalrsc_zfhmin_zca_zcd_zba_zbb_zbs_zkt_ssccptr_sscounterenw_sstvala_sstvecd_svade_svbare_svinval_svpbmt

    /// RVA22U64 User-mode Profile.
    Rva22u64,       // rv64i_m_a_f_d_c_b_zic64b_zicbom_zicbop_zicboz_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicsr_zihintpause_zihpm_zmmul_za64rs_zaamo_zalrsc_zfhmin_zca_zcd_zba_zbb_zbs_zkt

//  Rva23s64,       // rv64i_m_a_f_d_c_b_v_h_zic64b_zicbom_zicbop_zicboz_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicond_zicsr_zifencei_zihintntl_zihintpause_zihpm_zimop_zmmul_za64rs_zaamo_zalrsc_zawrs_zfa_zfhmin_zca_zcb_zcd_zcmop_zba_zbb_zbs_zkt_zvbb_zve32f_zve32x_zve64d_zve64f_zve64x_zvfhmin_zvkb_zvkt_zvl128b_zvl32b_zvl64b_sha_shcounterenw_shgatpa_shtvala_shvsatpa_shvstvala_shvstvecd_ssccptr_sscofpmf_sscounterenw_ssnpm_ssstateen_sstc_sstvala_sstvecd_ssu64xl_supm_svade_svbare_svinval_svnapot_svpbmt

    /// RVA23U64 User-mode Profile.
    Rva23u64,       // rv64i_m_a_f_d_c_b_v_zic64b_zicbom_zicbop_zicboz_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicond_zicsr_zihintntl_zihintpause_zihpm_zimop_zmmul_za64rs_zaamo_zalrsc_zawrs_zfa_zfhmin_zca_zcb_zcd_zcmop_zba_zbb_zbs_zkt_zvbb_zve32f_zve32x_zve64d_zve64f_zve64x_zvfhmin_zvkb_zvkt_zvl128b_zvl32b_zvl64b_supm

//  Rvb23s64,       // rv64i_m_a_f_d_c_b_zic64b_zicbom_zicbop_zicboz_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicond_zicsr_zifencei_zihintntl_zihintpause_zihpm_zimop_zmmul_za64rs_zaamo_zalrsc_zawrs_zfa_zca_zcb_zcd_zcmop_zba_zbb_zbs_zkt_ssccptr_sscofpmf_sscounterenw_sstc_sstvala_sstvecd_ssu64xl_svade_svbare_svinval_svnapot_svpbmt

    /// RVB23U64 User-mode Profile.
    Rvb23u64,       // rv64i_m_a_f_d_c_b_zic64b_zicbom_zicbop_zicboz_ziccamoa_ziccif_zicclsm_ziccrse_zicntr_zicond_zicsr_zihintntl_zihintpause_zihpm_zimop_zmmul_za64rs_zaamo_zalrsc_zawrs_zfa_zca_zcb_zcd_zcmop_zba_zbb_zbs_zkt

//  Rvi20u32, rv32

    /// RVI20U64 User-mode Profile (rv64i).
    Rvi20u64,       // rv64i

//  Experimental Profiles
//    Rvm23u32, rv32
}

impl fmt::Display for Riscv64ISA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Riscv64ISA {
    pub fn as_str(&self) -> &'static str {
        match self {
            Riscv64ISA::None => "none",
            Riscv64ISA::I => "i",
            Riscv64ISA::E => "e",
            Riscv64ISA::M => "m",
            Riscv64ISA::A => "a",
            Riscv64ISA::F => "f",
            Riscv64ISA::D => "d",
            Riscv64ISA::Q => "q",
            Riscv64ISA::C => "c",
            Riscv64ISA::B => "b",
            Riscv64ISA::V => "v",
//          Riscv64ISA::H => "h",
            Riscv64ISA::Zic64b => "zic64b",
            Riscv64ISA::Zicbom => "zicbom",
            Riscv64ISA::Zicbop => "zicbop",
            Riscv64ISA::Zicboz => "zicboz",
            Riscv64ISA::Ziccamoa => "ziccamoa",
            Riscv64ISA::Ziccamoc => "ziccamoc",
            Riscv64ISA::Ziccid => "ziccid",
            Riscv64ISA::Ziccif => "ziccif",
            Riscv64ISA::Zicclsm => "zicclsm",
            Riscv64ISA::Ziccrse => "ziccrse",
            Riscv64ISA::Zicntr => "zicntr",
            Riscv64ISA::Zicond => "zicond",
            Riscv64ISA::Zicsr => "zicsr",
            Riscv64ISA::Zifencei => "zifencei",
            Riscv64ISA::Zihintntl => "zihintntl",
            Riscv64ISA::Zihintpause => "zihintpause",
            Riscv64ISA::Zihpm => "zihpm",
            Riscv64ISA::Zilsd => "zilsd",
            Riscv64ISA::Zimop => "zimop",
            Riscv64ISA::Zmmul => "zmmul",
            Riscv64ISA::Za128rs => "za128rs",
            Riscv64ISA::Za64rs => "za64rs",
            Riscv64ISA::Zaamo => "zaamo",
            Riscv64ISA::Zabha => "zabha",
            Riscv64ISA::Zacas => "zacas",
            Riscv64ISA::Zalasr => "zalasr",
            Riscv64ISA::Zalrsc => "zalrsc",
            Riscv64ISA::Zama16b => "zama16b",
            Riscv64ISA::Zawrs => "zawrs",
            Riscv64ISA::Zfa => "zfa",
            Riscv64ISA::Zfbfmin => "zfbfmin",
            Riscv64ISA::Zfh => "zfh",
            Riscv64ISA::Zfhmin => "zfhmin",
            Riscv64ISA::Zfinx => "zfinx",
            Riscv64ISA::Zdinx => "zdinx",
            Riscv64ISA::Zca => "zca",
            Riscv64ISA::Zcb => "zcb",
            Riscv64ISA::Zcd => "zcd",
            Riscv64ISA::Zce => "zce",
            Riscv64ISA::Zcf => "zcf",
            Riscv64ISA::Zclsd => "zclsd",
            Riscv64ISA::Zcmop => "zcmop",
            Riscv64ISA::Zcmp => "zcmp",
            Riscv64ISA::Zcmt => "zcmt",
            Riscv64ISA::Zba => "zba",
            Riscv64ISA::Zbb => "zbb",
            Riscv64ISA::Zbc => "zbc",
            Riscv64ISA::Zbkb => "zbkb",
            Riscv64ISA::Zbkc => "zbkc",
            Riscv64ISA::Zbkx => "zbkx",
            Riscv64ISA::Zbs => "zbs",
            Riscv64ISA::Zk => "zk",
            Riscv64ISA::Zkn => "zkn",
            Riscv64ISA::Zknd => "zknd",
            Riscv64ISA::Zkne => "zkne",
            Riscv64ISA::Zknh => "zknh",
            Riscv64ISA::Zkr => "zkr",
            Riscv64ISA::Zks => "zks",
            Riscv64ISA::Zksed => "zksed",
            Riscv64ISA::Zksh => "zksh",
            Riscv64ISA::Zkt => "zkt",
            Riscv64ISA::Ztso => "ztso",
            Riscv64ISA::Zvbb => "zvbb",
            Riscv64ISA::Zvbc => "zvbc",
            Riscv64ISA::Zve32f => "zve32f",
            Riscv64ISA::Zve32x => "zve32x",
            Riscv64ISA::Zve64d => "zve64d",
            Riscv64ISA::Zve64f => "zve64f",
            Riscv64ISA::Zve64x => "zve64x",
            Riscv64ISA::Zvfbfmin => "zvfbfmin",
            Riscv64ISA::Zvfbfwma => "zvfbfwma",
            Riscv64ISA::Zvfh => "zvfh",
            Riscv64ISA::Zvfhmin => "zvfhmin",
            Riscv64ISA::Zvkb => "zvkb",
            Riscv64ISA::Zvkg => "zvkg",
            Riscv64ISA::Zvkn => "zvkn",
            Riscv64ISA::Zvknc => "zvknc",
            Riscv64ISA::Zvkned => "zvkned",
            Riscv64ISA::Zvkng => "zvkng",
            Riscv64ISA::Zvknha => "zvknha",
            Riscv64ISA::Zvknhb => "zvknhb",
            Riscv64ISA::Zvks => "zvks",
            Riscv64ISA::Zvksc => "zvksc",
            Riscv64ISA::Zvksed => "zvksed",
            Riscv64ISA::Zvksg => "zvksg",
            Riscv64ISA::Zvksh => "zvksh",
            Riscv64ISA::Zvkt => "zvkt",
            Riscv64ISA::Zvl1024b => "zvl1024b",
            Riscv64ISA::Zvl128b => "zvl128b",
            Riscv64ISA::Zvl16384b => "zvl16384b",
            Riscv64ISA::Zvl2048b => "zvl2048b",
            Riscv64ISA::Zvl256b => "zvl256b",
            Riscv64ISA::Zvl32768b => "zvl32768b",
            Riscv64ISA::Zvl32b => "zvl32b",
            Riscv64ISA::Zvl4096b => "zvl4096b",
            Riscv64ISA::Zvl512b => "zvl512b",
            Riscv64ISA::Zvl64b => "zvl64b",
            Riscv64ISA::Zvl65536b => "zvl65536b",
            Riscv64ISA::Zvl8192b => "zvl8192b",
            Riscv64ISA::Zhinx => "zhinx",
            Riscv64ISA::Zhinxmin => "zhinxmin",
//          Riscv64ISA::Sdext => "sdext",
//          Riscv64ISA::Sdtrig => "sdtrig",
//          Riscv64ISA::Sha => "sha",
//          Riscv64ISA::Shcounterenw => "shcounterenw",
//          Riscv64ISA::Shgatpa => "shgatpa",
//          Riscv64ISA::Shlcofideleg => "shlcofideleg",
//          Riscv64ISA::Shtvala => "shtvala",
//          Riscv64ISA::Shvsatpa => "shvsatpa",
//          Riscv64ISA::Shvstvala => "shvstvala",
//          Riscv64ISA::Shvstvecd => "shvstvecd",
//          Riscv64ISA::Smaia => "smaia",
//          Riscv64ISA::Smcdeleg => "smcdeleg",
//          Riscv64ISA::Smcntrpmf => "smcntrpmf",
//          Riscv64ISA::Smcsrind => "smcsrind",
//          Riscv64ISA::Smctr => "smctr",
//          Riscv64ISA::Smdbltrp => "smdbltrp",
//          Riscv64ISA::Smepmp => "smepmp",
//          Riscv64ISA::Smmpm => "smmpm",
//          Riscv64ISA::Smnpm => "smnpm",
//          Riscv64ISA::Smrnmi => "smrnmi",
//          Riscv64ISA::Smstateen => "smstateen",
//          Riscv64ISA::Ssaia => "ssaia",
//          Riscv64ISA::Ssccfg => "ssccfg",
//          Riscv64ISA::Ssccptr => "ssccptr",
//          Riscv64ISA::Sscofpmf => "sscofpmf",
//          Riscv64ISA::Sscounterenw => "sscounterenw",
//          Riscv64ISA::Sscsrind => "sscsrind",
//          Riscv64ISA::Ssctr => "ssctr",
//          Riscv64ISA::Ssdbltrp => "ssdbltrp",
//          Riscv64ISA::Ssnpm => "ssnpm",
//          Riscv64ISA::Sspm => "sspm",
//          Riscv64ISA::Ssqosid => "ssqosid",
//          Riscv64ISA::Ssstateen => "ssstateen",
//          Riscv64ISA::Ssstrict => "ssstrict",
//          Riscv64ISA::Sstc => "sstc",
//          Riscv64ISA::Sstvala => "sstvala",
//          Riscv64ISA::Sstvecd => "sstvecd",
//          Riscv64ISA::Ssu64xl => "ssu64xl",
            Riscv64ISA::Supm => "supm",
//          Riscv64ISA::Svade => "svade",
//          Riscv64ISA::Svadu => "svadu",
//          Riscv64ISA::Svbare => "svbare",
//          Riscv64ISA::Svinval => "svinval",
//          Riscv64ISA::Svnapot => "svnapot",
//          Riscv64ISA::Svpbmt => "svpbmt",
//          Riscv64ISA::Svrsw60t59b => "svrsw60t59b",
//          Riscv64ISA::Svvptc => "svvptc",
            Riscv64ISA::Xaifet => "xaifet",
            Riscv64ISA::Xandesbfhcvt => "xandesbfhcvt",
            Riscv64ISA::Xandesperf => "xandesperf",
            Riscv64ISA::Xandesvbfhcvt => "xandesvbfhcvt",
            Riscv64ISA::Xandesvdot => "xandesvdot",
            Riscv64ISA::Xandesvpackfph => "xandesvpackfph",
            Riscv64ISA::Xandesvsinth => "xandesvsinth",
            Riscv64ISA::Xandesvsintload => "xandesvsintload",
            Riscv64ISA::Xcheriot => "xcheriot",
            Riscv64ISA::Xcvalu => "xcvalu",
            Riscv64ISA::Xcvbi => "xcvbi",
            Riscv64ISA::Xcvbitmanip => "xcvbitmanip",
            Riscv64ISA::Xcvelw => "xcvelw",
            Riscv64ISA::Xcvmac => "xcvmac",
            Riscv64ISA::Xcvmem => "xcvmem",
            Riscv64ISA::Xcvsimd => "xcvsimd",
            Riscv64ISA::Xmipscbop => "xmipscbop",
            Riscv64ISA::Xmipscmov => "xmipscmov",
            Riscv64ISA::Xmipsexectl => "xmipsexectl",
            Riscv64ISA::Xmipslsp => "xmipslsp",
            Riscv64ISA::Xqccmp => "xqccmp",
            Riscv64ISA::Xqci => "xqci",
            Riscv64ISA::Xqcia => "xqcia",
            Riscv64ISA::Xqciac => "xqciac",
            Riscv64ISA::Xqcibi => "xqcibi",
            Riscv64ISA::Xqcibm => "xqcibm",
            Riscv64ISA::Xqcicli => "xqcicli",
            Riscv64ISA::Xqcicm => "xqcicm",
            Riscv64ISA::Xqcics => "xqcics",
            Riscv64ISA::Xqcicsr => "xqcicsr",
            Riscv64ISA::Xqciint => "xqciint",
            Riscv64ISA::Xqciio => "xqciio",
            Riscv64ISA::Xqcilb => "xqcilb",
            Riscv64ISA::Xqcili => "xqcili",
            Riscv64ISA::Xqcilia => "xqcilia",
            Riscv64ISA::Xqcilo => "xqcilo",
            Riscv64ISA::Xqcilsm => "xqcilsm",
            Riscv64ISA::Xqcisim => "xqcisim",
            Riscv64ISA::Xqcisls => "xqcisls",
            Riscv64ISA::Xqcisync => "xqcisync",
            Riscv64ISA::Xsfcease => "xsfcease",
            Riscv64ISA::Xsfmm128t => "xsfmm128t",
            Riscv64ISA::Xsfmm16t => "xsfmm16t",
            Riscv64ISA::Xsfmm32a => "xsfmm32a",
            Riscv64ISA::Xsfmm32a16f => "xsfmm32a16f",
            Riscv64ISA::Xsfmm32a32f => "xsfmm32a32f",
            Riscv64ISA::Xsfmm32a8f => "xsfmm32a8f",
            Riscv64ISA::Xsfmm32a8i => "xsfmm32a8i",
            Riscv64ISA::Xsfmm32t => "xsfmm32t",
            Riscv64ISA::Xsfmm64a64f => "xsfmm64a64f",
            Riscv64ISA::Xsfmm64t => "xsfmm64t",
            Riscv64ISA::Xsfmmbase => "xsfmmbase",
            Riscv64ISA::Xsfvcp => "xsfvcp",
            Riscv64ISA::Xsfvfbfexp16e => "xsfvfbfexp16e",
            Riscv64ISA::Xsfvfexp16e => "xsfvfexp16e",
            Riscv64ISA::Xsfvfexp32e => "xsfvfexp32e",
            Riscv64ISA::Xsfvfexpa => "xsfvfexpa",
            Riscv64ISA::Xsfvfexpa64e => "xsfvfexpa64e",
            Riscv64ISA::Xsfvfnrclipxfqf => "xsfvfnrclipxfqf",
            Riscv64ISA::Xsfvfwmaccqqq => "xsfvfwmaccqqq",
            Riscv64ISA::Xsfvqmaccdod => "xsfvqmaccdod",
            Riscv64ISA::Xsfvqmaccqoq => "xsfvqmaccqoq",
            Riscv64ISA::Xsifivecdiscarddlone => "xsifivecdiscarddlone",
            Riscv64ISA::Xsifivecflushdlone => "xsifivecflushdlone",
            Riscv64ISA::Xsmtvdot => "xsmtvdot",
            Riscv64ISA::Xsmtvdotii => "xsmtvdotii",
            Riscv64ISA::Xtheadba => "xtheadba",
            Riscv64ISA::Xtheadbb => "xtheadbb",
            Riscv64ISA::Xtheadbs => "xtheadbs",
            Riscv64ISA::Xtheadcmo => "xtheadcmo",
            Riscv64ISA::Xtheadcondmov => "xtheadcondmov",
            Riscv64ISA::Xtheadfmemidx => "xtheadfmemidx",
            Riscv64ISA::Xtheadmac => "xtheadmac",
            Riscv64ISA::Xtheadmemidx => "xtheadmemidx",
            Riscv64ISA::Xtheadmempair => "xtheadmempair",
            Riscv64ISA::Xtheadsync => "xtheadsync",
            Riscv64ISA::Xtheadvdot => "xtheadvdot",
            Riscv64ISA::Xventanacondops => "xventanacondops",
            Riscv64ISA::Xwchc => "xwchc",
            Riscv64ISA::P => "p",
            Riscv64ISA::Y => "y",
            Riscv64ISA::Zibi => "zibi",
            Riscv64ISA::Zicfilp => "zicfilp",
            Riscv64ISA::Zicfiss => "zicfiss",
            Riscv64ISA::Zvabd => "zvabd",
            Riscv64ISA::Zvbc32e => "zvbc32e",
            Riscv64ISA::Zvdot4a8i => "zvdot4a8i",
            Riscv64ISA::Zvfbdota32f => "zvfbdota32f",
            Riscv64ISA::Zvfbfa => "zvfbfa",
            Riscv64ISA::Zvfofp8min => "zvfofp8min",
            Riscv64ISA::Zvfqwbdota8f => "zvfqwbdota8f",
            Riscv64ISA::Zvfqwdota8f => "zvfqwdota8f",
            Riscv64ISA::Zvfwbdota16bf => "zvfwbdota16bf",
            Riscv64ISA::Zvfwdota16bf => "zvfwdota16bf",
            Riscv64ISA::Zvkgs => "zvkgs",
            Riscv64ISA::Zvqwbdota16i => "zvqwbdota16i",
            Riscv64ISA::Zvqwbdota8i => "zvqwbdota8i",
            Riscv64ISA::Zvqwdota16i => "zvqwdota16i",
            Riscv64ISA::Zvqwdota8i => "zvqwdota8i",
            Riscv64ISA::Zvvfmm => "zvvfmm",
            Riscv64ISA::Zvvmm => "zvvmm",
            Riscv64ISA::Zvvmtls => "zvvmtls",
            Riscv64ISA::Zvvmttls => "zvvmttls",
            Riscv64ISA::Zvzip => "zvzip",
//          Riscv64ISA::Smpmpmt => "smpmpmt",
//          Riscv64ISA::Svukte => "svukte",
            Riscv64ISA::Xqccmt => "xqccmt",
//          Riscv64ISA::Xsfmclic => "xsfmclic",
//          Riscv64ISA::Xsfsclic => "xsfsclic",

//          Riscv64ISA::Rva20s64 => "rva20s64",
            Riscv64ISA::Rva20u64 => "rva20u64",
//          Riscv64ISA::Rva22s64 => "rva22s64",
            Riscv64ISA::Rva22u64 => "rva22u64",
//          Riscv64ISA::Rva23s64 => "rva23s64",
            Riscv64ISA::Rva23u64 => "rva23u64",
//          Riscv64ISA::Rvb23s64 => "rvb23s64",
            Riscv64ISA::Rvb23u64 => "rvb23u64",
//            Riscv64ISA::Rvi20u32 => "rvi20u32",
            Riscv64ISA::Rvi20u64 => "rvi20u64",
//            Riscv64ISA::Rvm23u32 => "rvm23u32",
        }
    }

    pub fn is_experimental(&self) -> bool {
        matches!(
            self,
            Riscv64ISA::P
                | Riscv64ISA::Y
                | Riscv64ISA::Zibi
                | Riscv64ISA::Zicfilp
                | Riscv64ISA::Zicfiss
                | Riscv64ISA::Zvabd
                | Riscv64ISA::Zvbc32e
                | Riscv64ISA::Zvdot4a8i
                | Riscv64ISA::Zvfbdota32f
                | Riscv64ISA::Zvfbfa
                | Riscv64ISA::Zvfofp8min
                | Riscv64ISA::Zvfqwbdota8f
                | Riscv64ISA::Zvfqwdota8f
                | Riscv64ISA::Zvfwbdota16bf
                | Riscv64ISA::Zvfwdota16bf
                | Riscv64ISA::Zvkgs
                | Riscv64ISA::Zvqwbdota16i
                | Riscv64ISA::Zvqwbdota8i
                | Riscv64ISA::Zvqwdota16i
                | Riscv64ISA::Zvqwdota8i
                | Riscv64ISA::Zvvfmm
                | Riscv64ISA::Zvvmm
                | Riscv64ISA::Zvvmtls
                | Riscv64ISA::Zvvmttls
                | Riscv64ISA::Zvzip
                | Riscv64ISA::Xqccmt
        )
    }

    pub fn all() -> &'static [Riscv64ISA] {
        &[
            Riscv64ISA::I,
            Riscv64ISA::E,
            Riscv64ISA::M,
            Riscv64ISA::A,
            Riscv64ISA::F,
            Riscv64ISA::D,
            Riscv64ISA::Q,
            Riscv64ISA::C,
            Riscv64ISA::B,
            Riscv64ISA::V,
            Riscv64ISA::Zic64b,
            Riscv64ISA::Zicbom,
            Riscv64ISA::Zicbop,
            Riscv64ISA::Zicboz,
            Riscv64ISA::Ziccamoa,
            Riscv64ISA::Ziccamoc,
            Riscv64ISA::Ziccid,
            Riscv64ISA::Ziccif,
            Riscv64ISA::Zicclsm,
            Riscv64ISA::Ziccrse,
            Riscv64ISA::Zicntr,
            Riscv64ISA::Zicond,
            Riscv64ISA::Zicsr,
            Riscv64ISA::Zifencei,
            Riscv64ISA::Zihintntl,
            Riscv64ISA::Zihintpause,
            Riscv64ISA::Zihpm,
            Riscv64ISA::Zilsd,
            Riscv64ISA::Zimop,
            Riscv64ISA::Zmmul,
            Riscv64ISA::Za128rs,
            Riscv64ISA::Za64rs,
            Riscv64ISA::Zaamo,
            Riscv64ISA::Zabha,
            Riscv64ISA::Zacas,
            Riscv64ISA::Zalasr,
            Riscv64ISA::Zalrsc,
            Riscv64ISA::Zama16b,
            Riscv64ISA::Zawrs,
            Riscv64ISA::Zfa,
            Riscv64ISA::Zfbfmin,
            Riscv64ISA::Zfh,
            Riscv64ISA::Zfhmin,
            Riscv64ISA::Zfinx,
            Riscv64ISA::Zdinx,
            Riscv64ISA::Zca,
            Riscv64ISA::Zcb,
            Riscv64ISA::Zcd,
            Riscv64ISA::Zce,
            Riscv64ISA::Zcf,
            Riscv64ISA::Zclsd,
            Riscv64ISA::Zcmop,
            Riscv64ISA::Zcmp,
            Riscv64ISA::Zcmt,
            Riscv64ISA::Zba,
            Riscv64ISA::Zbb,
            Riscv64ISA::Zbc,
            Riscv64ISA::Zbkb,
            Riscv64ISA::Zbkc,
            Riscv64ISA::Zbkx,
            Riscv64ISA::Zbs,
            Riscv64ISA::Zk,
            Riscv64ISA::Zkn,
            Riscv64ISA::Zknd,
            Riscv64ISA::Zkne,
            Riscv64ISA::Zknh,
            Riscv64ISA::Zkr,
            Riscv64ISA::Zks,
            Riscv64ISA::Zksed,
            Riscv64ISA::Zksh,
            Riscv64ISA::Zkt,
            Riscv64ISA::Ztso,
            Riscv64ISA::Zvbb,
            Riscv64ISA::Zvbc,
            Riscv64ISA::Zve32f,
            Riscv64ISA::Zve32x,
            Riscv64ISA::Zve64d,
            Riscv64ISA::Zve64f,
            Riscv64ISA::Zve64x,
            Riscv64ISA::Zvfbfmin,
            Riscv64ISA::Zvfbfwma,
            Riscv64ISA::Zvfh,
            Riscv64ISA::Zvfhmin,
            Riscv64ISA::Zvkb,
            Riscv64ISA::Zvkg,
            Riscv64ISA::Zvkn,
            Riscv64ISA::Zvknc,
            Riscv64ISA::Zvkned,
            Riscv64ISA::Zvkng,
            Riscv64ISA::Zvknha,
            Riscv64ISA::Zvknhb,
            Riscv64ISA::Zvks,
            Riscv64ISA::Zvksc,
            Riscv64ISA::Zvksed,
            Riscv64ISA::Zvksg,
            Riscv64ISA::Zvksh,
            Riscv64ISA::Zvkt,
            Riscv64ISA::Zvl1024b,
            Riscv64ISA::Zvl128b,
            Riscv64ISA::Zvl16384b,
            Riscv64ISA::Zvl2048b,
            Riscv64ISA::Zvl256b,
            Riscv64ISA::Zvl32768b,
            Riscv64ISA::Zvl32b,
            Riscv64ISA::Zvl4096b,
            Riscv64ISA::Zvl512b,
            Riscv64ISA::Zvl64b,
            Riscv64ISA::Zvl65536b,
            Riscv64ISA::Zvl8192b,
            Riscv64ISA::Zhinx,
            Riscv64ISA::Zhinxmin,
            Riscv64ISA::Supm,
            Riscv64ISA::Xaifet,
            Riscv64ISA::Xandesbfhcvt,
            Riscv64ISA::Xandesperf,
            Riscv64ISA::Xandesvbfhcvt,
            Riscv64ISA::Xandesvdot,
            Riscv64ISA::Xandesvpackfph,
            Riscv64ISA::Xandesvsinth,
            Riscv64ISA::Xandesvsintload,
            Riscv64ISA::Xcheriot,
            Riscv64ISA::Xcvalu,
            Riscv64ISA::Xcvbi,
            Riscv64ISA::Xcvbitmanip,
            Riscv64ISA::Xcvelw,
            Riscv64ISA::Xcvmac,
            Riscv64ISA::Xcvmem,
            Riscv64ISA::Xcvsimd,
            Riscv64ISA::Xmipscbop,
            Riscv64ISA::Xmipscmov,
            Riscv64ISA::Xmipsexectl,
            Riscv64ISA::Xmipslsp,
            Riscv64ISA::Xqccmp,
            Riscv64ISA::Xqci,
            Riscv64ISA::Xqcia,
            Riscv64ISA::Xqciac,
            Riscv64ISA::Xqcibi,
            Riscv64ISA::Xqcibm,
            Riscv64ISA::Xqcicli,
            Riscv64ISA::Xqcicm,
            Riscv64ISA::Xqcics,
            Riscv64ISA::Xqcicsr,
            Riscv64ISA::Xqciint,
            Riscv64ISA::Xqciio,
            Riscv64ISA::Xqcilb,
            Riscv64ISA::Xqcili,
            Riscv64ISA::Xqcilia,
            Riscv64ISA::Xqcilo,
            Riscv64ISA::Xqcilsm,
            Riscv64ISA::Xqcisim,
            Riscv64ISA::Xqcisls,
            Riscv64ISA::Xqcisync,
            Riscv64ISA::Xsfcease,
            Riscv64ISA::Xsfmm128t,
            Riscv64ISA::Xsfmm16t,
            Riscv64ISA::Xsfmm32a,
            Riscv64ISA::Xsfmm32a16f,
            Riscv64ISA::Xsfmm32a32f,
            Riscv64ISA::Xsfmm32a8f,
            Riscv64ISA::Xsfmm32a8i,
            Riscv64ISA::Xsfmm32t,
            Riscv64ISA::Xsfmm64a64f,
            Riscv64ISA::Xsfmm64t,
            Riscv64ISA::Xsfmmbase,
            Riscv64ISA::Xsfvcp,
            Riscv64ISA::Xsfvfbfexp16e,
            Riscv64ISA::Xsfvfexp16e,
            Riscv64ISA::Xsfvfexp32e,
            Riscv64ISA::Xsfvfexpa,
            Riscv64ISA::Xsfvfexpa64e,
            Riscv64ISA::Xsfvfnrclipxfqf,
            Riscv64ISA::Xsfvfwmaccqqq,
            Riscv64ISA::Xsfvqmaccdod,
            Riscv64ISA::Xsfvqmaccqoq,
            Riscv64ISA::Xsifivecdiscarddlone,
            Riscv64ISA::Xsifivecflushdlone,
            Riscv64ISA::Xsmtvdot,
            Riscv64ISA::Xsmtvdotii,
            Riscv64ISA::Xtheadba,
            Riscv64ISA::Xtheadbb,
            Riscv64ISA::Xtheadbs,
            Riscv64ISA::Xtheadcmo,
            Riscv64ISA::Xtheadcondmov,
            Riscv64ISA::Xtheadfmemidx,
            Riscv64ISA::Xtheadmac,
            Riscv64ISA::Xtheadmemidx,
            Riscv64ISA::Xtheadmempair,
            Riscv64ISA::Xtheadsync,
            Riscv64ISA::Xtheadvdot,
            Riscv64ISA::Xventanacondops,
            Riscv64ISA::Xwchc,
            Riscv64ISA::P,
            Riscv64ISA::Y,
            Riscv64ISA::Zibi,
            Riscv64ISA::Zicfilp,
            Riscv64ISA::Zicfiss,
            Riscv64ISA::Zvabd,
            Riscv64ISA::Zvbc32e,
            Riscv64ISA::Zvdot4a8i,
            Riscv64ISA::Zvfbdota32f,
            Riscv64ISA::Zvfbfa,
            Riscv64ISA::Zvfofp8min,
            Riscv64ISA::Zvfqwbdota8f,
            Riscv64ISA::Zvfqwdota8f,
            Riscv64ISA::Zvfwbdota16bf,
            Riscv64ISA::Zvfwdota16bf,
            Riscv64ISA::Zvkgs,
            Riscv64ISA::Zvqwbdota16i,
            Riscv64ISA::Zvqwbdota8i,
            Riscv64ISA::Zvqwdota16i,
            Riscv64ISA::Zvqwdota8i,
            Riscv64ISA::Zvvfmm,
            Riscv64ISA::Zvvmm,
            Riscv64ISA::Zvvmtls,
            Riscv64ISA::Zvvmttls,
            Riscv64ISA::Zvzip,
            Riscv64ISA::Xqccmt,
        ]
    }
}

impl FromStr for Riscv64ISA {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let norm = s.to_ascii_lowercase().replace(['_', '.'], "-");
        match norm.as_str() {
            "none" => Ok(Riscv64ISA::None),
            "i" => Ok(Riscv64ISA::I),
            "e" => Ok(Riscv64ISA::E),
            "m" => Ok(Riscv64ISA::M),
            "a" => Ok(Riscv64ISA::A),
            "f" => Ok(Riscv64ISA::F),
            "d" => Ok(Riscv64ISA::D),
            "q" => Ok(Riscv64ISA::Q),
            "c" => Ok(Riscv64ISA::C),
            "b" => Ok(Riscv64ISA::B),
            "v" => Ok(Riscv64ISA::V),
            "zic64b" => Ok(Riscv64ISA::Zic64b),
            "zicbom" => Ok(Riscv64ISA::Zicbom),
            "zicbop" => Ok(Riscv64ISA::Zicbop),
            "zicboz" => Ok(Riscv64ISA::Zicboz),
            "ziccamoa" => Ok(Riscv64ISA::Ziccamoa),
            "ziccamoc" => Ok(Riscv64ISA::Ziccamoc),
            "ziccid" => Ok(Riscv64ISA::Ziccid),
            "ziccif" => Ok(Riscv64ISA::Ziccif),
            "zicclsm" => Ok(Riscv64ISA::Zicclsm),
            "ziccrse" => Ok(Riscv64ISA::Ziccrse),
            "zicntr" => Ok(Riscv64ISA::Zicntr),
            "zicond" => Ok(Riscv64ISA::Zicond),
            "zicsr" => Ok(Riscv64ISA::Zicsr),
            "zifencei" => Ok(Riscv64ISA::Zifencei),
            "zihintntl" => Ok(Riscv64ISA::Zihintntl),
            "zihintpause" => Ok(Riscv64ISA::Zihintpause),
            "zihpm" => Ok(Riscv64ISA::Zihpm),
            "zilsd" => Ok(Riscv64ISA::Zilsd),
            "zimop" => Ok(Riscv64ISA::Zimop),
            "zmmul" => Ok(Riscv64ISA::Zmmul),
            "za128rs" => Ok(Riscv64ISA::Za128rs),
            "za64rs" => Ok(Riscv64ISA::Za64rs),
            "zaamo" => Ok(Riscv64ISA::Zaamo),
            "zabha" => Ok(Riscv64ISA::Zabha),
            "zacas" => Ok(Riscv64ISA::Zacas),
            "zalasr" => Ok(Riscv64ISA::Zalasr),
            "zalrsc" => Ok(Riscv64ISA::Zalrsc),
            "zama16b" => Ok(Riscv64ISA::Zama16b),
            "zawrs" => Ok(Riscv64ISA::Zawrs),
            "zfa" => Ok(Riscv64ISA::Zfa),
            "zfbfmin" => Ok(Riscv64ISA::Zfbfmin),
            "zfh" => Ok(Riscv64ISA::Zfh),
            "zfhmin" => Ok(Riscv64ISA::Zfhmin),
            "zfinx" => Ok(Riscv64ISA::Zfinx),
            "zdinx" => Ok(Riscv64ISA::Zdinx),
            "zca" => Ok(Riscv64ISA::Zca),
            "zcb" => Ok(Riscv64ISA::Zcb),
            "zcd" => Ok(Riscv64ISA::Zcd),
            "zce" => Ok(Riscv64ISA::Zce),
            "zcf" => Ok(Riscv64ISA::Zcf),
            "zclsd" => Ok(Riscv64ISA::Zclsd),
            "zcmop" => Ok(Riscv64ISA::Zcmop),
            "zcmp" => Ok(Riscv64ISA::Zcmp),
            "zcmt" => Ok(Riscv64ISA::Zcmt),
            "zba" => Ok(Riscv64ISA::Zba),
            "zbb" => Ok(Riscv64ISA::Zbb),
            "zbc" => Ok(Riscv64ISA::Zbc),
            "zbkb" => Ok(Riscv64ISA::Zbkb),
            "zbkc" => Ok(Riscv64ISA::Zbkc),
            "zbkx" => Ok(Riscv64ISA::Zbkx),
            "zbs" => Ok(Riscv64ISA::Zbs),
            "zk" => Ok(Riscv64ISA::Zk),
            "zkn" => Ok(Riscv64ISA::Zkn),
            "zknd" => Ok(Riscv64ISA::Zknd),
            "zkne" => Ok(Riscv64ISA::Zkne),
            "zknh" => Ok(Riscv64ISA::Zknh),
            "zkr" => Ok(Riscv64ISA::Zkr),
            "zks" => Ok(Riscv64ISA::Zks),
            "zksed" => Ok(Riscv64ISA::Zksed),
            "zksh" => Ok(Riscv64ISA::Zksh),
            "zkt" => Ok(Riscv64ISA::Zkt),
            "ztso" => Ok(Riscv64ISA::Ztso),
            "zvbb" => Ok(Riscv64ISA::Zvbb),
            "zvbc" => Ok(Riscv64ISA::Zvbc),
            "zve32f" => Ok(Riscv64ISA::Zve32f),
            "zve32x" => Ok(Riscv64ISA::Zve32x),
            "zve64d" => Ok(Riscv64ISA::Zve64d),
            "zve64f" => Ok(Riscv64ISA::Zve64f),
            "zve64x" => Ok(Riscv64ISA::Zve64x),
            "zvfbfmin" => Ok(Riscv64ISA::Zvfbfmin),
            "zvfbfwma" => Ok(Riscv64ISA::Zvfbfwma),
            "zvfh" => Ok(Riscv64ISA::Zvfh),
            "zvfhmin" => Ok(Riscv64ISA::Zvfhmin),
            "zvkb" => Ok(Riscv64ISA::Zvkb),
            "zvkg" => Ok(Riscv64ISA::Zvkg),
            "zvkn" => Ok(Riscv64ISA::Zvkn),
            "zvknc" => Ok(Riscv64ISA::Zvknc),
            "zvkned" => Ok(Riscv64ISA::Zvkned),
            "zvkng" => Ok(Riscv64ISA::Zvkng),
            "zvknha" => Ok(Riscv64ISA::Zvknha),
            "zvknhb" => Ok(Riscv64ISA::Zvknhb),
            "zvks" => Ok(Riscv64ISA::Zvks),
            "zvksc" => Ok(Riscv64ISA::Zvksc),
            "zvksed" => Ok(Riscv64ISA::Zvksed),
            "zvksg" => Ok(Riscv64ISA::Zvksg),
            "zvksh" => Ok(Riscv64ISA::Zvksh),
            "zvkt" => Ok(Riscv64ISA::Zvkt),
            "zvl1024b" => Ok(Riscv64ISA::Zvl1024b),
            "zvl128b" => Ok(Riscv64ISA::Zvl128b),
            "zvl16384b" => Ok(Riscv64ISA::Zvl16384b),
            "zvl2048b" => Ok(Riscv64ISA::Zvl2048b),
            "zvl256b" => Ok(Riscv64ISA::Zvl256b),
            "zvl32768b" => Ok(Riscv64ISA::Zvl32768b),
            "zvl32b" => Ok(Riscv64ISA::Zvl32b),
            "zvl4096b" => Ok(Riscv64ISA::Zvl4096b),
            "zvl512b" => Ok(Riscv64ISA::Zvl512b),
            "zvl64b" => Ok(Riscv64ISA::Zvl64b),
            "zvl65536b" => Ok(Riscv64ISA::Zvl65536b),
            "zvl8192b" => Ok(Riscv64ISA::Zvl8192b),
            "zhinx" => Ok(Riscv64ISA::Zhinx),
            "zhinxmin" => Ok(Riscv64ISA::Zhinxmin),
            "supm" => Ok(Riscv64ISA::Supm),
            "xaifet" => Ok(Riscv64ISA::Xaifet),
            "xandesbfhcvt" => Ok(Riscv64ISA::Xandesbfhcvt),
            "xandesperf" => Ok(Riscv64ISA::Xandesperf),
            "xandesvbfhcvt" => Ok(Riscv64ISA::Xandesvbfhcvt),
            "xandesvdot" => Ok(Riscv64ISA::Xandesvdot),
            "xandesvpackfph" => Ok(Riscv64ISA::Xandesvpackfph),
            "xandesvsinth" => Ok(Riscv64ISA::Xandesvsinth),
            "xandesvsintload" => Ok(Riscv64ISA::Xandesvsintload),
            "xcheriot" => Ok(Riscv64ISA::Xcheriot),
            "xcvalu" => Ok(Riscv64ISA::Xcvalu),
            "xcvbi" => Ok(Riscv64ISA::Xcvbi),
            "xcvbitmanip" => Ok(Riscv64ISA::Xcvbitmanip),
            "xcvelw" => Ok(Riscv64ISA::Xcvelw),
            "xcvmac" => Ok(Riscv64ISA::Xcvmac),
            "xcvmem" => Ok(Riscv64ISA::Xcvmem),
            "xcvsimd" => Ok(Riscv64ISA::Xcvsimd),
            "xmipscbop" => Ok(Riscv64ISA::Xmipscbop),
            "xmipscmov" => Ok(Riscv64ISA::Xmipscmov),
            "xmipsexectl" => Ok(Riscv64ISA::Xmipsexectl),
            "xmipslsp" => Ok(Riscv64ISA::Xmipslsp),
            "xqccmp" => Ok(Riscv64ISA::Xqccmp),
            "xqci" => Ok(Riscv64ISA::Xqci),
            "xqcia" => Ok(Riscv64ISA::Xqcia),
            "xqciac" => Ok(Riscv64ISA::Xqciac),
            "xqcibi" => Ok(Riscv64ISA::Xqcibi),
            "xqcibm" => Ok(Riscv64ISA::Xqcibm),
            "xqcicli" => Ok(Riscv64ISA::Xqcicli),
            "xqcicm" => Ok(Riscv64ISA::Xqcicm),
            "xqcics" => Ok(Riscv64ISA::Xqcics),
            "xqcicsr" => Ok(Riscv64ISA::Xqcicsr),
            "xqciint" => Ok(Riscv64ISA::Xqciint),
            "xqciio" => Ok(Riscv64ISA::Xqciio),
            "xqcilb" => Ok(Riscv64ISA::Xqcilb),
            "xqcili" => Ok(Riscv64ISA::Xqcili),
            "xqcilia" => Ok(Riscv64ISA::Xqcilia),
            "xqcilo" => Ok(Riscv64ISA::Xqcilo),
            "xqcilsm" => Ok(Riscv64ISA::Xqcilsm),
            "xqcisim" => Ok(Riscv64ISA::Xqcisim),
            "xqcisls" => Ok(Riscv64ISA::Xqcisls),
            "xqcisync" => Ok(Riscv64ISA::Xqcisync),
            "xsfcease" => Ok(Riscv64ISA::Xsfcease),
            "xsfmm128t" => Ok(Riscv64ISA::Xsfmm128t),
            "xsfmm16t" => Ok(Riscv64ISA::Xsfmm16t),
            "xsfmm32a" => Ok(Riscv64ISA::Xsfmm32a),
            "xsfmm32a16f" => Ok(Riscv64ISA::Xsfmm32a16f),
            "xsfmm32a32f" => Ok(Riscv64ISA::Xsfmm32a32f),
            "xsfmm32a8f" => Ok(Riscv64ISA::Xsfmm32a8f),
            "xsfmm32a8i" => Ok(Riscv64ISA::Xsfmm32a8i),
            "xsfmm32t" => Ok(Riscv64ISA::Xsfmm32t),
            "xsfmm64a64f" => Ok(Riscv64ISA::Xsfmm64a64f),
            "xsfmm64t" => Ok(Riscv64ISA::Xsfmm64t),
            "xsfmmbase" => Ok(Riscv64ISA::Xsfmmbase),
            "xsfvcp" => Ok(Riscv64ISA::Xsfvcp),
            "xsfvfbfexp16e" => Ok(Riscv64ISA::Xsfvfbfexp16e),
            "xsfvfexp16e" => Ok(Riscv64ISA::Xsfvfexp16e),
            "xsfvfexp32e" => Ok(Riscv64ISA::Xsfvfexp32e),
            "xsfvfexpa" => Ok(Riscv64ISA::Xsfvfexpa),
            "xsfvfexpa64e" => Ok(Riscv64ISA::Xsfvfexpa64e),
            "xsfvfnrclipxfqf" => Ok(Riscv64ISA::Xsfvfnrclipxfqf),
            "xsfvfwmaccqqq" => Ok(Riscv64ISA::Xsfvfwmaccqqq),
            "xsfvqmaccdod" => Ok(Riscv64ISA::Xsfvqmaccdod),
            "xsfvqmaccqoq" => Ok(Riscv64ISA::Xsfvqmaccqoq),
            "xsifivecdiscarddlone" => Ok(Riscv64ISA::Xsifivecdiscarddlone),
            "xsifivecflushdlone" => Ok(Riscv64ISA::Xsifivecflushdlone),
            "xsmtvdot" => Ok(Riscv64ISA::Xsmtvdot),
            "xsmtvdotii" => Ok(Riscv64ISA::Xsmtvdotii),
            "xtheadba" => Ok(Riscv64ISA::Xtheadba),
            "xtheadbb" => Ok(Riscv64ISA::Xtheadbb),
            "xtheadbs" => Ok(Riscv64ISA::Xtheadbs),
            "xtheadcmo" => Ok(Riscv64ISA::Xtheadcmo),
            "xtheadcondmov" => Ok(Riscv64ISA::Xtheadcondmov),
            "xtheadfmemidx" => Ok(Riscv64ISA::Xtheadfmemidx),
            "xtheadmac" => Ok(Riscv64ISA::Xtheadmac),
            "xtheadmemidx" => Ok(Riscv64ISA::Xtheadmemidx),
            "xtheadmempair" => Ok(Riscv64ISA::Xtheadmempair),
            "xtheadsync" => Ok(Riscv64ISA::Xtheadsync),
            "xtheadvdot" => Ok(Riscv64ISA::Xtheadvdot),
            "xventanacondops" => Ok(Riscv64ISA::Xventanacondops),
            "xwchc" => Ok(Riscv64ISA::Xwchc),
            "p" => Ok(Riscv64ISA::P),
            "y" => Ok(Riscv64ISA::Y),
            "zibi" => Ok(Riscv64ISA::Zibi),
            "zicfilp" => Ok(Riscv64ISA::Zicfilp),
            "zicfiss" => Ok(Riscv64ISA::Zicfiss),
            "zvabd" => Ok(Riscv64ISA::Zvabd),
            "zvbc32e" => Ok(Riscv64ISA::Zvbc32e),
            "zvdot4a8i" => Ok(Riscv64ISA::Zvdot4a8i),
            "zvfbdota32f" => Ok(Riscv64ISA::Zvfbdota32f),
            "zvfbfa" => Ok(Riscv64ISA::Zvfbfa),
            "zvfofp8min" => Ok(Riscv64ISA::Zvfofp8min),
            "zvfqwbdota8f" => Ok(Riscv64ISA::Zvfqwbdota8f),
            "zvfqwdota8f" => Ok(Riscv64ISA::Zvfqwdota8f),
            "zvfwbdota16bf" => Ok(Riscv64ISA::Zvfwbdota16bf),
            "zvfwdota16bf" => Ok(Riscv64ISA::Zvfwdota16bf),
            "zvkgs" => Ok(Riscv64ISA::Zvkgs),
            "zvqwbdota16i" => Ok(Riscv64ISA::Zvqwbdota16i),
            "zvqwbdota8i" => Ok(Riscv64ISA::Zvqwbdota8i),
            "zvqwdota16i" => Ok(Riscv64ISA::Zvqwdota16i),
            "zvqwdota8i" => Ok(Riscv64ISA::Zvqwdota8i),
            "zvvfmm" => Ok(Riscv64ISA::Zvvfmm),
            "zvvmm" => Ok(Riscv64ISA::Zvvmm),
            "zvvmtls" => Ok(Riscv64ISA::Zvvmtls),
            "zvvmttls" => Ok(Riscv64ISA::Zvvmttls),
            "zvzip" => Ok(Riscv64ISA::Zvzip),
            "xqccmt" => Ok(Riscv64ISA::Xqccmt),
            "rva20u64" => Ok(Riscv64ISA::Rva20u64),
            "rva22u64" => Ok(Riscv64ISA::Rva22u64),
            "rva23u64" => Ok(Riscv64ISA::Rva23u64),
            "rvb23u64" => Ok(Riscv64ISA::Rvb23u64),
            "rvi20u64" => Ok(Riscv64ISA::Rvi20u64),
            other => Err(format!("Unknown Riscv64 ISA extension: {}", other)),
        }
    }
}

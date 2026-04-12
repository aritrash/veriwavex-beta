# VeriWaveX v2.0.1 - Professional Verilog Simulation Suite

VeriWaveX is a high-performance, cross-platform Electronic Design Automation (EDA) suite designed for Verilog HDL development, simulation, and synthesis. Built with Rust and powered by industrial-grade open-source backends, it provides a seamless bridge between code and hardware visualization.

Developed by Aritrash Sarkar

[![Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Proprietary-gold?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux-brightgreen?style=for-the-badge)](https://github.com/aritrash/veriwavex/releases)
[![Version](https://img.shields.io/badge/version-1.1.0--stable-green?style=for-the-badge)](https://github.com/aritrash/veriwavex/releases)

## Built With

* [Rust](https://www.rust-lang.org/) - Performance-critical systems orchestration.
* [eframe/egui](https://github.com/emilk/egui) - GPU-accelerated Immediate Mode GUI.
* [Icarus Verilog](http://iverilog.icarus.com/) - High-fidelity simulation engine.
* [GTKWave](http://gtkwave.sourceforge.net/) - Industry-standard waveform visualization.

---

## Key Features

* **Advanced Editor:** Syntax highlighting and project management for Verilog (.v) and Testbenches.
* **Integrated Synthesis:** Real-time RTL Schematic generation using the Yosys Open SYnthesis Suite.
* **Logic Visualization:** High-fidelity gate-level diagrams via Graphviz integration.
* **Simulation Engine:** Full support for Icarus Verilog (iverilog/vvp) for functional verification.
* **Waveform Analysis:** Deep-linking with GTKWave for signal debugging and VCD analysis.

---

## Installation (Windows)

1.  Download the `VeriWaveX_v2.0.1_Setup.exe`.
2.  Run the installer and accept the EULA.
3.  The installer will automatically configure the bundled `vendor` toolchain.
4.  Launch VeriWaveX from the Start Menu or Desktop shortcut.

---

## v2.0.1 Release Notes (The "Stabilization" Update)

This is a critical maintenance release following the v2.0.0 major update. It addresses pathing issues and toolchain communication bugs discovered during initial lab testing.

### Bug Fixes
* **Fixed GTKWave Launch Issue:** Corrected the pathing logic in the binary resolver that prevented the Waveform Viewer from opening after successful simulation.
* **Pathing Robustness:** Improved the "Cargo-aware" path detection logic to prevent "OS Error 2" when running the app from different directory levels.
* **VCD Generation:** Fixed a race condition where the IDE would attempt to open GTKWave before the simulator had finished flushing the .vcd file to disk.

### Improvements
* **Refined Console Logs:** Added "Path Tracing" debug info (internal) to help diagnose toolchain issues in various environments.
* **Binary Execution:** Updated execution flags to prevent ghost console windows from popping up during background synthesis.

---

## License & Attribution

VeriWaveX is proprietary software. All rights reserved. 
Unauthorized redistribution or reverse engineering of the bundled toolchain is prohibited.

Copyright (c) 2026 Aritrash Sarkar.
Meghnad Saha Institute of Technology.
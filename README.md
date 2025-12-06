# Stegano RS WebApp

A secure, local-first web application for steganography, built with Rust, WebAssembly, and React. 

**Live Demo: [https://mage-enderman.github.io/stegano-rs-webapp/](https://mage-enderman.github.io/stegano-rs-webapp/)**

## Overview

This project is a modern reimagining of the `stegano-rs` CLI tool as a Progressive Web App (PWA). It allows you to:

*   **Hide Data**: Embed secret files and text into PNG images.
*   **Unveil Data**: Extract hidden data from carrier images.
*   **Secure**: All processing happens locally in your browser via WebAssembly (Wasm). No data is ever uploaded to a server.
*   **Encrypt**: Optional password protection for your hidden data using robust cryptography (Argon2 + ChaCha20Poly1305).

## Features

- **In-Browser Processing**: Powered by `stegano-core` compiled to Wasm.
- **Offline Capable**: Installable as a PWA on desktop and mobile.
- **Modern UI**: Dark-themed, responsive interface built with React and Vite.
- **Flexible Output**: Choose suffix, prefix, or custom filenames for your steganographic images.

## Development

### Prerequisites

- **Rust**: `stable` toolchain with `wasm32-unknown-unknown` target.
- **Node.js**: Version 20+.
- **wasm-pack**: For building the Wasm module (`npm install -g wasm-pack` or `cargo install wasm-pack`).

### Build & Run

1.  **Build Wasm Module**:
    ```bash
    wasm-pack build crates/stegano-wasm --target web --out-dir ../../webapp/src/pkg --dev
    ```

2.  **Run WebApp**:
    ```bash
    cd webapp
    npm install
    npm run dev
    ```

## Architecture

- `crates/stegano-core`: The core Rust library handling image manipulation and encryption.
- `crates/stegano-wasm`: Wasm bindings exposing core functionality to JavaScript.
- `crates/stegano-seasmoke`: Cryptography helper crate.
- `webapp/`: The React + TypeScript frontend.

## License

GPL-3.0

<div id="readme-top"></div>

<h1 align="center">
  <br />
    <a href="https://xodium.org/">
        <img src="logo.svg" alt="Cadiotheka Logo" width="200">
    </a>
  <br /><br />
  Cadiotheka
  <br />
  <br />
</h1>

<h4 align="center">The open hub for CAD creators</h4><br />

<div align="center">

[![Contributors][contributors_shield_url]][contributors_url]
[![Issues][issues_shield_url]][issues_url]
[![License][license_shield_url]][license_url]
</div>

## Table of Contents

- [About](#about)
- [Usage](#usage)
- [Built With](#built-with)
- [Code of Conduct][code_of_conduct_url]
- [Contributing][contributing_url]
- [License][license_url]
- [Requirements](#requirements)
- [Security][security_url]

## About

Cadiotheka is an open hub for CAD creators. It collects, organizes, and provides resources, tooling, and references to support people working with computer-aided design. The hub runs as a browser application built with [egui](https://github.com/emilk/egui) and compiled to WebAssembly.

## Requirements

- [Rust](https://www.rust-lang.org/) — latest stable toolchain
- `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/)

Install the target and Trunk with:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

<p align="right"><a href="#readme-top">▲</a></p>

## Running Locally

Cadiotheka runs in the browser as a WebAssembly application. Start a local
server with Trunk:

```bash
git clone https://github.com/XodiumSoftware/cadiotheka.git
cd cadiotheka
trunk serve --port 8080
```

Then open <http://localhost:8080/index.html#dev> in a browser.

For a release build:

```bash
trunk build --release
```

The static site is placed in `dist/`.

<p align="right"><a href="#readme-top">▲</a></p>

## Built With

<div align="center">

[![Built With][built_with_shield_url]][built_with_url]
</div>

<p align="right"><a href="#readme-top">▲</a></p>

[code_of_conduct_url]: https://github.com/XodiumSoftware/cadiotheka?tab=coc-ov-file

[contributing_url]: https://github.com/XodiumSoftware/cadiotheka/blob/main/CONTRIBUTING.md

[contributors_shield_url]: https://img.shields.io/github/contributors/XodiumSoftware/cadiotheka?style=for-the-badge&color=blue

[contributors_url]: https://github.com/XodiumSoftware/cadiotheka/graphs/contributors

[issues_shield_url]: https://img.shields.io/github/issues/XodiumSoftware/cadiotheka?style=for-the-badge&color=yellow

[issues_url]: https://github.com/XodiumSoftware/cadiotheka/issues

[license_shield_url]: https://img.shields.io/github/license/XodiumSoftware/cadiotheka?style=for-the-badge&color=blue

[license_url]: https://github.com/XodiumSoftware/cadiotheka?tab=AGPL-3.0-1-ov-file

[built_with_shield_url]: https://skillicons.dev/icons?i=rust,github,githubactions

[built_with_url]: https://skillicons.dev

[security_url]: https://github.com/XodiumSoftware/cadiotheka?tab=security-ov-file

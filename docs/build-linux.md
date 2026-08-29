Instructions use Ubuntu package names. Package names may differ on other distros.

If it's inconvenient to install the latest version of [just](https://github.com/casey/just), use the without just instructions. The catch is that the without just instructions are more likely to change in the future, so if you're packaging openscq30 and the latest version of just is easily available, prefer the with just instructions.

## Building openscq30-cli on Linux

1. Install the latest version of rust

### Without just

2. Run `cargo build --package openscq30-cli --profile release-fast` (or `cargo build --package openscq30-cli --release`, but it's very slow to build)
3. The compiled binary can be found at `target/release-fast/openscq30`

### With just

2. Run `just build-cli-fast` (or `just build-cli` but it's very slow to build)
3. The compiled binary can be found at `build-output/openscq30`

## Building openscq30-gui on Linux

Choose exactly one frontend target: `cosmic` or `kde`. Their binaries share
the installed name, `openscq30-gui`, and are not co-installable.

### COSMIC

1. Install the latest version of rust
2. Install pkg-config libdbus-1-dev libxkbcommon-dev

#### Without just

3. Run `cargo build --package openscq30-gui --profile release-fast` (or `cargo build --package openscq30-gui --release`, but it's very slow to build)
4. The compiled binary can be found at `target/release-fast/openscq30-gui`

#### With just

3. Run `just build-gui-fast cosmic` (or `just build-gui cosmic` but it's very slow to build)
4. The compiled binary can be found at `build-output/cosmic/openscq30-gui`

### KDE

1. Install the latest version of rust, CMake, a C++ compiler, pkg-config, libdbus-1-dev, and libxkbcommon-dev.
2. On KDE Neon, install qt6-base-dev, qt6-declarative-dev, qt6-declarative-dev-tools, and kf6-kirigami-dev.
3. Run `just build-gui-fast kde` (or `just build-gui kde` but it's very slow to build).
4. The compiled binary can be found at `build-output/kde/openscq30-gui`.

The tested KDE Neon toolchain is Rust 1.97.1, CMake 3.30.5, Qt 6.11.1, KF6 Kirigami 6.29.0, and CXX-Qt 0.10.0.

To install a selected target with the CLI, use `just install cosmic /usr/local` or `just install kde /usr/local`.

## Runtime Dependencies

- [cosmic-icons](https://github.com/pop-os/cosmic-icons/): required by the COSMIC target. If a package isn't available, clone the git repo and run `just install`.
- KDE target: Qt 6 Quick, QML, and Kirigami runtime modules from the distribution, plus BlueZ and system D-Bus.

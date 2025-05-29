# DS Tuner

DS Tuner is a DualSense controller raw input modifier. It modifies the data received from the controller even before it reaches the driver via [HID-BPF](https://docs.kernel.org/hid/hid-bpf.html), so it works in every context **(even hidraw)**.

Currently you can adjust:

 * Analog stick deadzone
 * Trigger deadzone

_Full list of options can be found in [ds-tuner.toml](ds-tuner.toml)._

### Supported Controllers

 * DualSense

Currently only the base DualSense controller is supported since it's the only one I own, but other Sony controllers could be supported supported as well with minimal changes. (PRs welcome)

## Usage

Can be used either manually or as a systemd service. The configuration file will be hotreloaded upon change.

### Manually

Compile and then run the execuatable as root.

```sh
cargo build --release && sudo ./target/release/ds-tuner
```

_The config will be read from the current working directory._

### Syetemd Service

Example instructions to install it can be found in [PKGBUILD](pkg/PKGBUILD).

## License
[GPL-3.0-only](LICENSE) - Copyright (C) 2025 Csányi István
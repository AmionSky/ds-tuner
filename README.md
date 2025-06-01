# DS Tuner

DS Tuner is a DualSense controller raw input modifier. It modifies the data received from the controller even before it reaches the driver via [HID-BPF](https://docs.kernel.org/hid/hid-bpf.html), so it works in every context **(even hidraw)**.

Currently you can adjust:

 * Analog stick deadzone
 * Trigger deadzone

_Full list of options can be found in [ds-tuner.toml](ds-tuner.toml)._

### Supported Controllers

Currently only the base DualSense controller is supported since it's the only one I own, but other Sony controllers could be supported supported as well with minimal changes. (PRs welcome)

## Usage

Can be used either manually or as a systemd service. The configuration file will be hotreloaded upon change.

### Manually (with cargo install)

Install it with cargo.

```sh
cargo install ds-tuner
```

Then run execuatable as root. Optionally specify the config file to use.

```sh
sudo ds-tuner start --config <path to your config file>
```

_The config path defaults to `ds-tuner.toml` in the current working directory._

### Syetemd Service

Example instructions to install it can be found in [PKGBUILD](pkg/PKGBUILD).

## License

[GPL-3.0-only](LICENSE) - Copyright (C) 2025 Csányi István
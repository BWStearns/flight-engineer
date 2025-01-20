# Flight Engineer

This is a library and command line tool for talking directly to MAVLink vehicles.

The goal is to allow for high level commands, as well as finer grained MAVLink comms, as well as proxying to/from remote systems (GCS or cloud systems).

While _definitely_ not yet present, a long term goal is to make this "pluggable" maybe somewhat like Bevy.

# Usage

Example: `cargo run serial:/dev/tty.usbmodem01:57600 telem`

Right now only `telem`, `reboot`, and `params` are implemented, and telem is really only proof of life since it only prints one message, but they're viable examples to build off of.

# Embedded

Personal embedded systems playground. Starting with Arduino, likely moving
to ESP32 and beyond.

## Use

### Flash

```bash
arduino-cli compile --fqbn arduino:avr:uno <sketch>/
arduino-cli upload -p /dev/<device> --fqbn arduino:avr:uno <sketch>/
```

### Serial

```bash
# picocom -b 9600 /dev/ttyACM0
picocom -b 9600 /dev/<device>
```

The -b 9600 is the baud rate — it must match what's in your sketch.
Exit using `Ctrl+A` then `Ctrl+X`.

## Setup

### Packages

```sh
sudo apt install clangd picocom

# Via package manager (or download binary from GitHub)
curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh
# Move to PATH if needed
sudo mv bin/arduino-cli /usr/local/bin/

arduino-cli config init
arduino-cli core update-index
arduino-cli core install arduino:avr      # for Uno/Nano/Mega
# arduino-cli core install arduino:megaavr  # for Nano Every
# arduino-cli core install esp32:esp32      # for ESP32
```

### System

Write to device permissions.

**Permanent** Add your user to the dialout group. On Linux, serial ports like
/dev/ttyACM0 are owned by the dialout group, so only members can access them.
This is the permanent fix but requires a logout/login to take effect because
group memberships are loaded at session start.

```sh
usermod -aG dialout $USER
```

**Temporary** Directly gives read/write permission to everyone on that specific
device file right now, bypassing the group check. It works immediately but is
temporary — resets when you unplug and replug the board.

```sh
chmod a+rw /dev/<device>
```

### Neovim

```sh
:MasonInstall arduino-language-server
```

```lua
require('lspconfig').arduino_language_server.setup {
  cmd = {
    "arduino-language-server",
    "-clangd", "/usr/bin/clangd",   -- system clangd
    "-cli", "/usr/local/bin/arduino-cli",
    "-cli-config", vim.fn.expand("~/.arduino15/arduino-cli.yaml"),
    "-fqbn", "arduino:avr:uno",
  },
}
```

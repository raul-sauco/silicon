# Embedded

Personal embedded systems playground. Starting with Arduino, likely moving
to ESP32 and beyond.

## Use

### Tools

- `arduino-cli` — compile & upload
- Neovim + `arduino-language-server` — editor setup

### Flash

```bash
arduino-cli compile --fqbn arduino:avr:uno <sketch>/
arduino-cli upload -p /dev/ttyUSB0 --fqbn arduino:avr:uno <sketch>/
```

### Serial

```bash
picocom -b 9600 /dev/ttyUSB0
```

## Setup

### Packages

```sh
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

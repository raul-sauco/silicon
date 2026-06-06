# Battery shunt

DIY battery shunt inspired in the Smart Shunt

> This has its own file, instead of being a subproject in the motorhome file
> because it seems like something that can be used elsewhere.

## Architecture

- External _manganin_ resistor. 100A/75mV?
- Dedicated sensing pads in PCB to avoid track resistance.
- Prevent ground loops with an I2C Digital Isolator (like an ISO1540).
- Buck converter: run from the battery 12v

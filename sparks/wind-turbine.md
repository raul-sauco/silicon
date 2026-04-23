# wind-turbine

DIY wind turbine using a salvaged washing machine BLDC motor as the alternator,
ESP32-S3 for monitoring and dump load control.

## Motor

The ideal donor is a **Fisher & Paykel SmartDrive** or equivalent LG direct-drive
front/top-loader — these use a permanent magnet synchronous motor that outputs
3-phase AC when spun, making them natural alternators.

- Factory wired for ~300V — needs rewiring to low-voltage high-amperage (target: 24V)
- LG front and top loaders use the same architecture and are easier to source in Spain
- Avoid induction motors (older machines, no permanent magnets) — won't self-generate
- Avoid universal/brushed motors — inefficient as generators, wear quickly

> ⚠️ Open-circuit at high RPM can produce 1500V+ — always connect dump load before
> disconnecting battery. Treat output as lethal until proven otherwise.

## System Architecture

> Blades (3x)
> ↓
> Hub (car clutch plate fits F&P spline directly)
> ↓
> Washing machine BLDC motor (rewired for 24V low-voltage output)
> ↓
> 3-phase bridge rectifier (6x diodes)
> ↓
> DC bus (12V or 24V)
> ↓
> MPPT charge controller ──→ Battery bank
> ↓
> Dump load resistor (water heater element)
> ↓
> ESP32-S3 monitoring + control
> ↓
> WiFi dashboard (voltage, current, RPM, wind speed, kWh)

## Blades

Three options, in order of effort:

| Option      | Material                   | Notes                                                             |
| ----------- | -------------------------- | ----------------------------------------------------------------- |
| PVC pipe    | Split + shaped PVC offcuts | Easiest, free if you have pipe, good enough for low wind          |
| Carved wood | Hardwood                   | Better aero, heavier, more rewarding                              |
| 3D printed  | PETG or ASA                | Many profiles on Printables designed for F&P motors, UV resistant |

Axis choice:

- **Vertical axis** — works with wind from any direction, electronics at ground level, simpler mechanically
- **Horizontal axis** — more efficient, needs yaw mechanism to face wind

## Electronics (AliExpress parts)

| Part                                  | Role                               | Approx price |
| ------------------------------------- | ---------------------------------- | ------------ |
| ESP32-S3 DevKitC-1                    | Main controller, WiFi, Rust target | ~€5–8        |
| INA226 current + voltage sensor (I2C) | DC bus monitoring                  | ~€1–2        |
| Hall effect sensor (e.g. AH3144)      | Shaft RPM measurement              | ~€0.50–1     |
| Cup anemometer module                 | Actual wind speed input            | ~€5–10       |
| IRF540N MOSFET + driver               | Dump load switching                | ~€1–2        |
| 3-phase bridge rectifier (50A)        | AC → DC conversion                 | ~€2–4        |
| DS3231 RTC module                     | Timekeeping for energy logging     | ~€1–2        |
| 2.4" TFT ILI9341 display              | Local dashboard                    | ~€4–8        |
| Waterproof box IP65                   | Outdoor electronics enclosure      | ~€4–8        |
| Water heater element 12V/24V          | Dump load resistor                 | ~€3–8        |
| LM2596 buck converter                 | 24V → 5V for ESP32                 | ~€0.80–1.50  |

## Firmware (embedded Rust)

- Target: `esp32s3` via `esp-hal` (no_std bare metal)
- INA226 over I2C — continuous voltage/current sampling
- Hall sensor interrupt → RPM calculation
- MOSFET PWM dump load control when bus voltage exceeds threshold
- WiFi → MQTT or simple HTTP dashboard
- Deep sleep between samples to extend any battery-backed operation

## Rust crates

- `esp-hal` — bare metal ESP32-S3 HAL
- `embedded-hal` — hardware abstraction traits
- `ina226` — INA226 driver
- `fugit` — time/frequency types for embedded

## Safety checklist

- [ ] Dump load wired and tested before any battery connection
- [ ] Rectifier and DC bus fused appropriately
- [ ] Motor rewired and verified at low RPM before full deployment
- [ ] Enclosure rated IP65 or better
- [ ] Mechanical blade balance verified before spin-up
- [ ] Overspeed protection plan (furling tail or electronic brake)

## References

- [Instructables — Vertical Wind Generator from Washing Machine Motor](https://www.instructables.com/A-Vertical-Wind-Generator-from-Washing-Machine-Mot/)
- [Permies — SmartDrive wind generator thread](https://permies.com/t/18586/Smart-Drive-wind-generator)
- [Hackaday — ESP32 solar tracker (same monitoring approach)](https://hackaday.io/project/185105-low-cost-solar-panel-solution-mppt-sun-tracker)

## Open questions

- [ ] Confirm washing machine motor type (brand/model?)
- [ ] Vertical vs horizontal axis decision
- [ ] Blade material and profile
- [ ] 12V vs 24V bus voltage
- [ ] Grid-tie vs battery-only vs direct DC loads

# Open Loop Solar tracker

Using GPS + astronomical calculations, The math tells you exactly where the sun
is at any moment. Called an open-loop solar tracker and it's more reliable than
than sensor-based trackers that get confused by clouds.

A good example here on Hackaday, includes components, datasheets and videos.

[Hackaday — ESP32 solar tracker (same monitoring approach)][1]

High-torque digital servos (25kg/cm+) — expensive, ~€20-40 each

## Design decisions to make before ordering

Single vs dual axis — for La Línea's latitude (~36°N) a single east-west
axis captures most of the gain. Elevation changes slowly through the seasons
and a fixed tilt at ~36° already handles it reasonably. Start single-axis.

GPS vs NTP — your location is fixed, so NTP over WiFi + hardcoded coordinates
is genuinely the right call. The NEO-6M is only worth it if you want the
tracker to be portable or if you want GPS as a precision time source independent
of internet.

The AS5600 encoder is the sleeper part on that list — it's a contactless
magnetic angle sensor that tells you the actual panel angle, not just the
commanded angle. Closing that loop in Rust (commanded position → verify
actual position → correct if needed) turns it from an open-loop timer into a
real closed-loop controller. Highly recommended even for the prototype.

## Actuators

### Linear

More robust, slower, purpose-built for solar trackers, widely available on
AliExpress ~€15-25 each worm gear DC motors with encoder — best mechanical
solution, can't be back-driven by wind, but requires motor driver circuitry

### Servos

For a practice/smaller panel the SG90 or MG996R servos are fine. For a real
12V panel, linear actuators are the realistic choice.

### Windshield wiper motors

The _Hackaday_ page reduces cost using old car windshield wiper motors.

## Code

Crate `solarpositioning` on crates.io — a Rust library implementing both the
NREL SPA and Grena3 algorithms, giving you azimuth and elevation for any
lat/lon/time, with no_std support so it runs directly on the ESP32 bare metal.

`Grandado` This is production-quality — over 1000 test points validated
against the reference implementation.

Reference Arduino project to study: G6EJD/ESP32_2D_Sun_Tracker — an ESP32 that
calculates sun position from lat/lon/datetime using NOAA equations and drives
azimuth + elevation servos in a gimbal. Grandado It's Arduino/C++ not Rust,
but the logic maps directly and it's well documented.

On GPS vs NTP: A simpler approach used by some ESP32 tracker projects is to get
UTC time over NTP via WiFi, fall back to an RTC module if WiFi is unavailable,
sleep 10 minutes between updates during the day, and park at the morning
position overnight. eBay Since you're in La Línea and likely have reliable
WiFi near your panels, NTP + hardcoded coordinates is honestly simpler than GPS
for a fixed installation — GPS adds cost and complexity for coordinates that
never change. Save GPS for if you want to make it portable.

## Parts List

### 🧠 Brains

| Name / Search Term           | Qty | Short explanation                                                             | Approx price | Link |
| ---------------------------- | --- | ----------------------------------------------------------------------------- | ------------ | ---- |
| ESP32-S3 DevKitC-1           | 1   | Main controller; runs solar position math + motor control + WiFi NTP          | ~$5–8        |      |
| DS3231 RTC module (I2C)      | 1   | Real-time clock backup if WiFi/NTP unavailable; battery-backed, very accurate | ~$1–2        |      |
| NEO-6M GPS module (optional) | 1   | Only needed if tracker will move locations; skip for fixed installation       | ~$3–6        |      |

### ⚙️ Motion — Small panel (practice, <1kg)

| Name / Search Term                        | Qty | Short explanation                                                              | Approx price | Link |
| ----------------------------------------- | --- | ------------------------------------------------------------------------------ | ------------ | ---- |
| MG996R metal gear servo                   | 2   | 13kg/cm torque; suitable for small panels and prototyping; azimuth + elevation | ~$3–5 each   |      |
| PCA9685 16-channel PWM servo driver (I2C) | 1   | Controls multiple servos from ESP32 over I2C; frees up GPIO pins               | ~$1–3        |      |

### ⚙️ Motion — Full 12V panel (production build)

| Name / Search Term                      | Qty | Short explanation                                                                           | Approx price  | Link |
| --------------------------------------- | --- | ------------------------------------------------------------------------------------------- | ------------- | ---- |
| Linear actuator 12V 100–150mm stroke    | 1–2 | Purpose-built for solar trackers; worm gear can't be back-driven by wind; 1 for single-axis | ~$15–25 each  |      |
| IBT-2 / BTS7960 43A motor driver        | 1–2 | H-bridge driver for linear actuator; handles 12V/high current easily                        | ~$3–5 each    |      |
| Hall effect limit switch or reed switch | 2–4 | End-stop detection; prevents actuator over-travel and mechanical damage                     | ~$0.50–1 each |      |

### 📐 Feedback & Sensing (optional but recommended)

| Name / Search Term                   | Qty | Short explanation                                                             | Approx price | Link |
| ------------------------------------ | --- | ----------------------------------------------------------------------------- | ------------ | ---- |
| AS5600 magnetic encoder module (I2C) | 1–2 | Contactless angle sensor; verifies actual panel position vs calculated target | ~$1–3        |      |
| INA226 current+voltage sensor (I2C)  | 1   | Measures panel output power — lets you log efficiency gain from tracking      | ~$1–2        |      |
| BMP280 / BME280 temperature+pressure | 1   | Log ambient conditions alongside power output                                 | ~$1–3        |      |

### 🖥️ Display & Interface

| Name / Search Term             | Qty | Short explanation                                                            | Approx price | Link |
| ------------------------------ | --- | ---------------------------------------------------------------------------- | ------------ | ---- |
| 2.4" TFT SPI ILI9341 (320x240) | 1   | Show azimuth, elevation, panel power, time, next move — satisfying dashboard | ~$4–8        |      |
| Rotary encoder KY-040          | 1   | Manual override / menu navigation                                            | ~$0.50–1     |      |

### ⚡ Power

| Name / Search Term                     | Qty | Short explanation                                                    | Approx price | Link |
| -------------------------------------- | --- | -------------------------------------------------------------------- | ------------ | ---- |
| DC-DC buck converter LM2596 adjustable | 2   | Steps 12V panel down to 5V for ESP32 and servos                      | ~$0.80–1.50  |      |
| Waterproof project box (IP65)          | 1   | Protects electronics outdoors; search "caja estanca IP65 AliExpress" | ~$4–8        |      |
| M12 waterproof cable glands (pack)     | 1   | Sealed cable entry for the project box                               | ~$1–2        |      |

### 🔌 Misc

| Name / Search Term               | Qty   | Short explanation                              | Approx price | Link |
| -------------------------------- | ----- | ---------------------------------------------- | ------------ | ---- |
| Dupont wire kit + JST connectors | 1 set | For internal wiring; JST for motor connections | ~$2–4        |      |
| CR2032 coin cell (for DS3231)    | 2     | RTC battery backup                             | ~$0.50–1     |      |

[1]: https://hackaday.io/project/185105-low-cost-solar-panel-solution-mppt-sun-tracker

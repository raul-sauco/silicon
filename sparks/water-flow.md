# Water flow measuring system

Measuring water flow to monitor compsumption and leaks.

Leaks can be detected monitoring at two points and comparing and/or tracking
patterns and detecting unusual spikes.

## Sensors

### Hall Effect / Turbine Flow Sensors (most popular for DIY)

A small turbine spins with water flow; a Hall effect sensor counts pulses

- Each pulse = a known volume (e.g. 1 pulse per 2.25 mL)
- Output: digital pulse train — very easy for microcontrollers to read
- Examples: YF-S201, FS400A — cheap (~$3–15), widely available
- Works inline on 1/2" or 3/4" pipes

#### YF-S201

Claimed accuracy: ±3–5% (at optimal flow rates) In practice, more like
±5–10% — it's a budget sensor. Best accuracy in the 1–10 L/min range;
degrades at very low or very high flow. Repeatability is decent, consistent
even if not perfectly accurate, so good for relative measurements

#### FS400A

Slightly better build quality, similar accuracy: ±2–3% claimed. More
consistent across a wider flow range, OK at typical domestic 20L/s consumption.

- DN25
- JST XH 2.54mm 3-pin connector: Red: 5V, Black: GND, Yellow/White: Signal

Needs a logic level shifter or a couple of resistors to bring the 5V to 3.3V

[Amazon ~10eur][1]
[AliExpress ~8eur][2]

### Ultrasonic Flow Sensors (non-invasive, clamp-on)

Clamp onto the outside of the pipe — no plumbing required
More expensive (~$30–300+) but non-intrusive
Output: usually 4–20 mA, RS485/Modbus, or pulse — needs conversion for 3.3V MCUs

### Electromagnetic Flow Meters

Very accurate, no moving parts
Overkill for most projects, expensive, requires conductive fluid

## Code

Pulses can be read with something like the sketch below

```cpp
volatile int pulseCount = 0;

void IRAM_ATTR pulseCounter() {
  pulseCount++;
}

void setup() {
  pinMode(2, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(2), pulseCounter, FALLING);
  Serial.begin(115200);
}

void loop() {
  delay(1000);
  noInterrupts();
  int count = pulseCount;
  pulseCount = 0;
  interrupts();

  // YF-S201: ~7.5 pulses per second = 1 L/min
  float flowRate = count / 7.5;  // L/min
  Serial.print("Flow rate: ");
  Serial.println(flowRate);
}
```

[1]: https://www.amazon.es/-/en/Koanhinn-Fs400A-Water-Sensor-1-2Mpa/dp/B0BYD9FSVQ
[2]: https://es.aliexpress.com/item/1005008958019846.html

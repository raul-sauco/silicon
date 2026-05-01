/**
 * DS18B20 1-Wire Temperature Sensor
 *
 * Libraries required (Arduino IDE → Library Manager):
 *   - "DallasTemperature" by Miles Burton
 *   - "OneWire" by Jim Studt et al.
 *
 * Wiring:
 *   DS18B20 VDD  → 3.3V or 5V
 *   DS18B20 GND  → GND
 *   DS18B20 DATA → Pin 2  (+ 4.7kΩ pull-up resistor to VDD)
 */

#include <DallasTemperature.h>
#include <OneWire.h>

// ── Configuration ────────────────────────────────────────────────────────────
constexpr uint8_t ONE_WIRE_PIN = 2;
constexpr uint16_t SAMPLE_INTERVAL = 1000; // ms between readings
constexpr uint8_t SENSOR_RESOLUTION =
    12; // bits: 9=0.5°C, 10=0.25°C, 11=0.125°C, 12=0.0625°C

// ── 1-Wire & sensor setup ────────────────────────────────────────────────────
OneWire oneWire(ONE_WIRE_PIN);
DallasTemperature sensors(&oneWire);

// ── Helpers ──────────────────────────────────────────────────────────────────

/** Print one sensor's reading in both Celsius and Fahrenheit. */
void printTemperature(uint8_t index) {
    const float tempC = sensors.getTempCByIndex(index);

    if (tempC == DEVICE_DISCONNECTED_C) {
        Serial.print(F("  Sensor "));
        Serial.print(index);
        Serial.println(F(": disconnected or not found"));
        return;
    }

    const float tempF = DallasTemperature::toFahrenheit(tempC);

    Serial.print(F("  Sensor "));
    Serial.print(index);
    Serial.print(F(": "));
    Serial.print(tempC, 2);
    Serial.print(F(" °C  /  "));
    Serial.print(tempF, 2);
    Serial.println(F(" °F"));
}

// ── Arduino lifecycle ────────────────────────────────────────────────────────
void setup() {
    Serial.begin(9600);
    Serial.println(F("DS18B20 Temperature Sensor"));
    Serial.println(F("──────────────────────────"));

    sensors.begin();

    const uint8_t deviceCount = sensors.getDeviceCount();
    Serial.print(F("Devices found on bus: "));
    Serial.println(deviceCount);

    if (deviceCount == 0) {
        Serial.println(F("WARNING: No sensors detected. Check wiring and "
                         "pull-up resistor."));
        return;
    }

    for (uint8_t i = 0; i < deviceCount; i++) {
        sensors.setResolution(i, SENSOR_RESOLUTION);
    }

    Serial.print(F("Resolution set to: "));
    Serial.print(SENSOR_RESOLUTION);
    Serial.println(F(" bit"));
    Serial.println();
}

void loop() {
    sensors.requestTemperatures();

    const uint8_t deviceCount = sensors.getDeviceCount();
    Serial.print(F("Reading "));
    Serial.print(deviceCount);
    Serial.println(F(" sensor(s):"));

    for (uint8_t i = 0; i < deviceCount; i++) {
        printTemperature(i);
    }

    Serial.println();
    delay(SAMPLE_INTERVAL);
}

// I2C Bus Scanner
// Upload to your Arduino, then open Serial Monitor at 9600 baud.
// Scans all 127 possible I2C addresses and prints any devices found.
// AHT20 should appear at 0x38, BMP280 at 0x76 or 0x77.

#include <Wire.h>

void setup() {
    Serial.begin(9600);
    Wire.begin();
    Serial.println("Scanning I2C bus...");
}

void loop() {
    uint8_t count = 0;
    for (uint8_t addr = 1; addr < 127; addr++) {
        Wire.beginTransmission(addr);
        uint8_t error = Wire.endTransmission();
        if (error == 0) {
            char buffer[32];
            sprintf(buffer, "Device found at 0x%02X", addr);
            Serial.println(buffer);
            count++;
        }
    }
    if (count == 0) {
        Serial.println("No I2C devices found");
    }
    delay(3000);
}

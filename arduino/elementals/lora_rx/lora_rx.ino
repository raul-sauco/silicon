// Arduino Pin     RA-02 Pin    Notes
// ────────────────────────────────────────────
// Power RA-02 from shifter's 3.3V rail if it has one, Otherwise from a
// separate 3.3V regulator — NOT directly from Arduino 3.3V pin (often too weak)
//
// 5V → shifter → 3.3V    Red
// GND             GND    Black       Direct, no shifter needed
// D13 (SCK)       SCK    Yellow      Through level shifter
// D11 (MOSI)      MOSI   Green       Through level shifter
// D10 (NSS/CS)    NSS    Blue        Through level shifter
// D9              RST    Purple      Through level shifter
// D2              DIO0   Orange      Through level shifter
//
// (RA-02 → Arduino direction)
// D12 (MISO)      MISO   White       Through level shifter
//
// D4             → LED (Long leg/Anode) → Resistor (220-330Ω) → GND

#include <LoRa.h>
#include <SPI.h>

#define LORA_CS 10
#define LORA_RST 9
#define LORA_DIO0 2
#define LED_PIN 4

void setup() {
    pinMode(LED_PIN, OUTPUT);
    Serial.begin(115200);

    // --- THE BUFFER COUNTDOWN ---
    // Flash the LED 3 times slowly. This gives your computer's
    // Serial Monitor 2 seconds to reconnect before ANY logs or init happen.
    for (int i = 0; i < 3; i++) {
        digitalWrite(LED_PIN, HIGH);
        delay(300);
        digitalWrite(LED_PIN, LOW);
        delay(300);
    }

    Serial.println("\n--- BOOTING SYSTEM ---");
    Serial.println("Initializing LoRa Hardware...");

    LoRa.setSPIFrequency(1E6);
    LoRa.setPins(LORA_CS, LORA_RST, LORA_DIO0);

    if (!LoRa.begin(433E6)) {
        Serial.println("LoRa init failed");
        // Rapid flash the LED if initialization fails to alert you without
        // looking at Serial Monitor
        while (true) {
            digitalWrite(LED_PIN, HIGH);
            delay(100);
            digitalWrite(LED_PIN, LOW);
            delay(100);
        }
    }

    LoRa.setSpreadingFactor(7);
    LoRa.setSignalBandwidth(125E3);
    LoRa.setCodingRate4(5);

    Serial.println("LoRa RX ready");

    // Quick double flash to confirm setup completed successfully
    digitalWrite(LED_PIN, HIGH);
    delay(150);
    digitalWrite(LED_PIN, LOW);
    delay(150);
    digitalWrite(LED_PIN, HIGH);
    delay(150);
    digitalWrite(LED_PIN, LOW);
}

void loop() {
    int packetSize = LoRa.parsePacket();

    if (packetSize) {
        // Turn on the LED immediately upon receiving a transmission
        digitalWrite(LED_PIN, HIGH);

        Serial.print("Received: ");
        while (LoRa.available()) {
            Serial.print((char)LoRa.read());
        }

        Serial.print("  RSSI: ");
        Serial.print(LoRa.packetRssi());
        Serial.print(" dBm  SNR: ");
        Serial.println(LoRa.packetSnr());

        // Keep the LED on long enough to be highly visible, then turn it off
        delay(200);
        digitalWrite(LED_PIN, LOW);
    }
}

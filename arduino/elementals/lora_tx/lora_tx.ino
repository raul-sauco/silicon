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

#include <LoRa.h>
#include <SPI.h>

#define LORA_CS 10
#define LORA_RST 9
#define LORA_DIO0 2

void setup() {
    Serial.begin(115200);
    while (!Serial)
        ;

    LoRa.setPins(LORA_CS, LORA_RST, LORA_DIO0);

    if (!LoRa.begin(433E6)) {
        Serial.println("LoRa init failed");
        while (true) {
            delay(100);
        }
    }

    LoRa.setSpreadingFactor(7);
    LoRa.setSignalBandwidth(125E3);
    LoRa.setCodingRate4(5);
    LoRa.setTxPower(5); // dBm, matches your Rust TX setting

    Serial.println("LoRa TX ready");
}

void loop() {
    Serial.println("Sending: HelloIno!");

    LoRa.beginPacket();
    LoRa.print("Hello Board!");
    LoRa.endPacket();

    delay(2000);
}

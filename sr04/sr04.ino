#include "SR04.h"

const uint8_t ECHO_PIN = 11;
const uint8_t TRIG_PIN = 12;
const uint16_t MEASURE_INTERVAL = 1000;

SR04 sr04(ECHO_PIN, TRIG_PIN);

void setup() {
    Serial.begin(9600);
}

void loop() {
    Serial.print(sr04.Distance());
    Serial.println("cm");
    delay(MEASURE_INTERVAL);
}

#include <Arduino.h>

constexpr uint8_t PIN_INPUT = 3;

void setup() {
    pinMode(LED_BUILTIN, OUTPUT);
    pinMode(PIN_INPUT, INPUT);
}

void loop() {
    digitalWrite(LED_BUILTIN, digitalRead(PIN_INPUT));
}

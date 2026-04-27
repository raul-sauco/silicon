#include <Arduino.h>

/**
 * Digital input → External LED
 *
 * Wiring:
 *   Button/sensor DO → Pin 3
 *   LED anode        → Pin 13 → 220Ω resistor → GND
 */

// ── Configuration ─────────────────────────────────────────────────────────
constexpr uint8_t PIN_LED = 13;
constexpr uint8_t PIN_INPUT = 3;

// ── Forward declarations ───────────────────────────────────────────────────
void setLed(bool state);

// ── Arduino lifecycle ──────────────────────────────────────────────────────
void setup() {
    Serial.begin(9600);
    pinMode(PIN_LED, OUTPUT);
    pinMode(PIN_INPUT, INPUT);
    setLed(false);
}

void loop() {
    const bool triggered = digitalRead(PIN_INPUT) == HIGH;
    setLed(triggered);
}

void setLed(bool state) {
    static bool lastState = false;
    if (state == lastState)
        return; // only act and print on change

    digitalWrite(PIN_LED, state);
    Serial.println(state ? F("HIGH — LED on") : F("LOW  — LED off"));
    lastState = state;
}

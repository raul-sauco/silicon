#include <Arduino.h>

/**
 * PWM crossfade between two LEDs
 *
 * Wiring:
 *   Pin 10 (~) → 220Ω → Yellow LED → GND
 *   Pin 11 (~) → 220Ω → Red LED   → GND
 */

// ── Configuration ─────────────────────────────────────────────────────────
constexpr uint8_t PIN_RED = 11;
constexpr uint8_t PIN_YELLOW = 10;
constexpr uint8_t STEP_DELAY = 15;

// ── Helpers ───────────────────────────────────────────────────────────────
void crossfade(uint8_t fromPin, uint8_t toPin,
               const __FlashStringHelper *label) {
    Serial.println(label);
    for (uint8_t val = 255; val > 0; val--) {
        analogWrite(fromPin, val);
        analogWrite(toPin, 255 - val);
        delay(STEP_DELAY);
    }
}

// ── Arduino lifecycle ──────────────────────────────────────────────────────
void setup() {
    Serial.begin(9600);
    pinMode(PIN_RED, OUTPUT);
    pinMode(PIN_YELLOW, OUTPUT);
    analogWrite(PIN_RED, 255);
    analogWrite(PIN_YELLOW, 0);
}

void loop() {
    crossfade(PIN_RED, PIN_YELLOW, F("Red → Yellow"));
    crossfade(PIN_YELLOW, PIN_RED, F("Yellow → Red"));
}

#include <Arduino.h>

/**
 * PWM Fade — ramps brightness up and down on a PWM pin
 *
 * Wiring:
 *   LED/laser anode → Pin 9 (~) → current limiting resistor → GND
 *   (or any other PWM-capable pin marked with ~ on the board)
 */

// ── Configuration ─────────────────────────────────────────────────────────
constexpr uint8_t PIN_PWM = 9;
// increase to fade faster, decrease for smoother
constexpr uint8_t FADE_STEP = 1;
constexpr uint8_t STEP_DELAY = 25; // ms between each step

// ── Forward declarations ───────────────────────────────────────────────────
void fadeTo(uint8_t target);

// ── Arduino lifecycle ──────────────────────────────────────────────────────
void setup() {
    pinMode(PIN_PWM, OUTPUT);
    analogWrite(PIN_PWM, 0); // start off
}

void loop() {
    fadeTo(255); // fade in
    fadeTo(0);   // fade out
}

void fadeTo(uint8_t target) {
    static uint8_t current = 0;
    while (current != target) {
        current += (current < target) ? FADE_STEP : -FADE_STEP;
        analogWrite(PIN_PWM, current);
        delay(STEP_DELAY);
    }
}

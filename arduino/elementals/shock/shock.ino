#include <Arduino.h>

/**
 * Shock / Knock Sensor → LED toggle
 *
 * Works with:
 *   - Vibration/shock switch modules (spring or ball type)
 *   - Knock sensor modules (piezo type)
 *
 * Wiring:
 *   Sensor DO (digital out) → Pin 3
 *   Built-in LED            → LED_BUILTIN (pin 13 on Uno)
 *
 * Note: Most sensor modules have a built-in LED that lights on trigger,
 *   and a potentiometer to adjust sensitivity (knock modules only).
 */

// ── Configuration ─────────────────────────────────────────────────────────
constexpr uint8_t PIN_INPUT = 3;
constexpr uint16_t DEBOUNCE_MS =
    50; // raise if LED toggles multiple times per knock
constexpr uint16_t LOCKOUT_MS = 200; // ignore re-triggers within this window

// ── State ──────────────────────────────────────────────────────────────────
bool ledState = false;
bool lastInputState = HIGH;
uint32_t lastTriggerTime = 0;

// ── Forward declarations ───────────────────────────────────────────────────
void handleTrigger();

// ── Arduino lifecycle ──────────────────────────────────────────────────────
void setup() {
    Serial.begin(9600);
    pinMode(LED_BUILTIN, OUTPUT);
    pinMode(PIN_INPUT, INPUT);
    digitalWrite(LED_BUILTIN, ledState);
    Serial.println(F("Shock/knock sensor ready"));
}

void loop() {
    const bool currentInput = digitalRead(PIN_INPUT);
    const uint32_t now = millis();

    // Detect falling edge (HIGH → LOW) outside the lockout window
    if (currentInput == LOW && lastInputState == HIGH) {
        if (now - lastTriggerTime >= LOCKOUT_MS) {
            delay(DEBOUNCE_MS);
            if (digitalRead(PIN_INPUT) == LOW) {
                lastTriggerTime = now;
                handleTrigger();
            }
        }
    }

    lastInputState = currentInput;
}

void handleTrigger() {
    ledState = !ledState;
    digitalWrite(LED_BUILTIN, ledState);
    Serial.print(F("Triggered — LED: "));
    Serial.println(ledState ? F("ON") : F("OFF"));
}

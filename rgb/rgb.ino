#include <Arduino.h>

/**
 * RGB LED — smooth colour crossfade
 *
 * Fades seamlessly between a sequence of colours,
 * blending all three channels simultaneously.
 *
 * Wiring:
 *   Pin 11 (~) → 220Ω → Red anode
 *   Pin 10 (~) → 220Ω → Green anode
 *   Pin  9 (~) → 220Ω → Blue anode
 *   Common cathode → GND
 *
 * Note: for common anode RGB LED, flip all analogWrite values:
 *   analogWrite(pin, 255 - value)
 */

// ── Configuration ─────────────────────────────────────────────────────────
constexpr uint8_t PIN_RED = 11;
constexpr uint8_t PIN_GREEN = 10;
constexpr uint8_t PIN_BLUE = 9;
constexpr uint8_t FADE_STEPS = 255; // higher = smoother
constexpr uint8_t STEP_DELAY = 10;  // ms per step — lower = faster

// ── Colour table — add, remove or reorder freely ──────────────────────────
struct Colour {
    uint8_t r, g, b;
    const char *name;
};

const Colour COLOURS[] = {
    {255, 0, 0, "Red"},       {0, 255, 0, "Green"},  {0, 0, 255, "Blue"},
    {255, 255, 0, "Yellow"},  {0, 255, 255, "Cyan"}, {255, 0, 255, "Magenta"},
    {255, 255, 255, "White"},
};

constexpr uint8_t COLOUR_COUNT = sizeof(COLOURS) / sizeof(COLOURS[0]);

// ── Forward declarations ───────────────────────────────────────────────────
void setRGB(uint8_t r, uint8_t g, uint8_t b);
void crossfade(Colour from, Colour to);

// ── Arduino lifecycle ──────────────────────────────────────────────────────
void setup() {
    Serial.begin(9600);
    pinMode(PIN_RED, OUTPUT);
    pinMode(PIN_GREEN, OUTPUT);
    pinMode(PIN_BLUE, OUTPUT);
    setRGB(0, 0, 0);
}

void loop() {
    for (uint8_t i = 0; i < COLOUR_COUNT; i++) {
        const Colour from = COLOURS[i];
        const Colour to =
            COLOURS[(i + 1) % COLOUR_COUNT]; // wraps back to first
        crossfade(from, to);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

void crossfade(Colour from, Colour to) {
    Serial.print(F("Fading to: "));
    Serial.println(to.name);

    for (uint8_t step = 0; step < FADE_STEPS; step++) {
        const float t = (float)step / FADE_STEPS; // 0.0 → 1.0
        setRGB(from.r + t * (to.r - from.r), from.g + t * (to.g - from.g),
               from.b + t * (to.b - from.b));
        delay(STEP_DELAY);
    }
}

void setRGB(uint8_t r, uint8_t g, uint8_t b) {
    analogWrite(PIN_RED, r);
    analogWrite(PIN_GREEN, g);
    analogWrite(PIN_BLUE, b);
}

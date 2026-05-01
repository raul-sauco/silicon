/**
 * 74HC595 Shift Register — 7-Segment Display Driver
 *
 * Wiring:
 *   Pin 8  → DS   (74HC595 pin 14) — Serial data
 *   Pin 9  → STCP (74HC595 pin 12) — Latch / Storage clock
 *   Pin 10 → SHCP (74HC595 pin 11) — Shift clock
 */

// ── Pin definitions ──────────────────────────────────────────────────────────
constexpr uint8_t PIN_LATCH = 9;  // STCP — latches shifted data to output
constexpr uint8_t PIN_CLOCK = 10; // SHCP — shift clock
constexpr uint8_t PIN_DATA = 8;   // DS   — serial data

// ── Display timing ───────────────────────────────────────────────────────────
constexpr uint16_t STEP_DELAY_MS = 500;

// ── 7-segment encoding table (common cathode, active HIGH) ───────────────────
//   Segments: .GFEDCBA
//   Index:     0     1     2     3     4     5     6     7
//              8     9     A     b     C     d     E     F    blank
constexpr uint8_t SEGMENT_TABLE[] = {
    0x3F, // 0 — 0b00111111
    0x06, // 1 — 0b00000110
    0x5B, // 2 — 0b01011011
    0x4F, // 3 — 0b01001111
    0x66, // 4 — 0b01100110
    0x6D, // 5 — 0b01101101
    0x7D, // 6 — 0b01111101
    0x07, // 7 — 0b00000111
    0x7F, // 8 — 0b01111111
    0x6F, // 9 — 0b01101111
    0x77, // A — 0b01110111
    0x7C, // b — 0b01111100
    0x39, // C — 0b00111001
    0x5E, // d — 0b01011110
    0x79, // E — 0b01111001
    0x71, // F — 0b01110001
    0x00, // blank
};

constexpr uint8_t TABLE_SIZE = sizeof(SEGMENT_TABLE);
constexpr uint8_t BLANK_INDEX = TABLE_SIZE - 1;

// ── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Send one byte to the 74HC595 and latch it to the outputs.
 * @param value  Raw segment byte (use SEGMENT_TABLE entries or a raw bitmask).
 */
void displayRaw(uint8_t value) {
    digitalWrite(PIN_LATCH, LOW);
    shiftOut(PIN_DATA, PIN_CLOCK, MSBFIRST, value);
    digitalWrite(PIN_LATCH, HIGH);
}

/**
 * Display a glyph by table index (0–15 = 0…F, 16 = blank).
 * Out-of-range indices silently show blank.
 */
void displayGlyph(uint8_t index) {
    if (index >= TABLE_SIZE)
        index = BLANK_INDEX;
    displayRaw(SEGMENT_TABLE[index]);
}

// ── Arduino lifecycle ────────────────────────────────────────────────────────
void setup() {
    pinMode(PIN_LATCH, OUTPUT);
    pinMode(PIN_CLOCK, OUTPUT);
    pinMode(PIN_DATA, OUTPUT);

    displayGlyph(BLANK_INDEX); // start with display off
}

void loop() {
    // Cycle through all 16 hex glyphs (0–F)
    for (uint8_t i = 0; i < BLANK_INDEX; i++) {
        displayGlyph(i);
        delay(STEP_DELAY_MS);
    }
}

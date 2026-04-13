const uint8_t PIN_LATCH = 11;
const uint8_t PIN_CLOCK = 9;
const uint8_t PIN_DATA = 12;
const uint16_t STEP_DELAY = 100;

uint8_t leds = 0;

void updateShiftRegister() {
    digitalWrite(PIN_LATCH, LOW);
    shiftOut(PIN_DATA, PIN_CLOCK, LSBFIRST, leds);
    digitalWrite(PIN_LATCH, HIGH);
}

void setup() {
    pinMode(PIN_LATCH, OUTPUT);
    pinMode(PIN_DATA, OUTPUT);
    pinMode(PIN_CLOCK, OUTPUT);
}

void loop() {
    updateShiftRegister();
    delay(STEP_DELAY);

    for (uint8_t i = 0; i < 8; i++) {
        bitSet(leds, i);
        updateShiftRegister();
        delay(STEP_DELAY);
    }

    for (uint8_t i = 0; i < 8; i++) {
        bitClear(leds, i);
        updateShiftRegister();
        delay(STEP_DELAY);
    }
}

const uint8_t PIN_LIGHT = A0;
const uint8_t PIN_LATCH = 11;
const uint8_t PIN_CLOCK = 9;
const uint8_t PIN_DATA = 12;

const uint8_t NUM_LEDS = 8;
const uint16_t ANALOG_MAX = 1023;
const uint8_t LEDS_PER_STEP = ANALOG_MAX / NUM_LEDS;

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
    uint16_t reading = analogRead(PIN_LIGHT);
    uint8_t leds_lit = constrain(reading / LEDS_PER_STEP, 0, NUM_LEDS);

    leds = 0;
    for (uint8_t i = 0; i < leds_lit; i++) {
        bitSet(leds, i);
    }
    updateShiftRegister();
}

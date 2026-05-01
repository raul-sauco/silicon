constexpr uint8_t SEGMENT_MAP[] = {
    0b11111100, // 0
    0b01100000, // 1
    0b11011010, // 2
    0b11110010, // 3
    0b01100110, // 4
    0b10110110, // 5
    0b10111110, // 6
    0b11100000, // 7
    0b11111110, // 8
    0b11100110  // 9
};

constexpr uint8_t LATCH_PIN = 3;
constexpr uint8_t CLOCK_PIN = 4;
constexpr uint8_t DATA_PIN = 2;

constexpr size_t DIGIT_COUNT = sizeof(SEGMENT_MAP) / sizeof(SEGMENT_MAP[0]);

void writeDigit(uint8_t digit) {
    if (digit >= DIGIT_COUNT)
        return;

    digitalWrite(LATCH_PIN, LOW);
    shiftOut(DATA_PIN, CLOCK_PIN, LSBFIRST, SEGMENT_MAP[digit]);
    digitalWrite(LATCH_PIN, HIGH);
}

void setup() {
    pinMode(LATCH_PIN, OUTPUT);
    pinMode(CLOCK_PIN, OUTPUT);
    pinMode(DATA_PIN, OUTPUT);
}

void loop() {
    for (int digit = 9; digit >= 0; --digit) {
        writeDigit(digit);
        delay(1000);
    }

    delay(3000);
}

const uint8_t PIN_LATCH = 11;
const uint8_t PIN_CLOCK = 9;
const uint8_t PIN_DATA = 12;

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
    updateShiftRegister();

    Serial.begin(9600);
    Serial.println("Enter LED number 0-7 or 'x' to clear");
}

void loop() {
    if (!Serial.available())
        return;

    char ch = Serial.read();

    if (ch >= '0' && ch <= '7') {
        uint8_t led = ch - '0';
        bitSet(leds, led);
        updateShiftRegister();
        char buffer[24];
        sprintf(buffer, "Turned on LED %d", led);
        Serial.println(buffer);
    } else if (ch == 'x') {
        leds = 0;
        updateShiftRegister();
        Serial.println("Cleared");
    }
}

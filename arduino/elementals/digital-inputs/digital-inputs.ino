const uint8_t PIN_LED = 5;
const uint8_t PIN_BUTTON_A = 9;
const uint8_t PIN_BUTTON_B = 8;

bool led_on = false;

void setup() {
    pinMode(PIN_LED, OUTPUT);
    pinMode(PIN_BUTTON_A, INPUT_PULLUP);
    pinMode(PIN_BUTTON_B, INPUT_PULLUP);
}

void loop() {
    if (digitalRead(PIN_BUTTON_A) == LOW || digitalRead(PIN_BUTTON_B) == LOW) {
        digitalWrite(PIN_LED, led_on ? LOW : HIGH);
        led_on = !led_on;
        delay(200); // Debounce
    }
}

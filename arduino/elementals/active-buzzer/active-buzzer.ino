const uint8_t PIN_BUZZER = 12;

void setup() {
    pinMode(PIN_BUZZER, OUTPUT);
}

void beep(uint16_t duration) {
    digitalWrite(PIN_BUZZER, HIGH);
    delay(duration);
    digitalWrite(PIN_BUZZER, LOW);
    delay(duration);
}

void loop() {
    for (int i = 0; i < 20; i++) {
        uint16_t duration;
        if (i < 5) {
            duration = 500;
        } else if (i < 10) {
            duration = 300;
        } else {
            duration = 100;
        }
        beep(duration);
    }
    digitalWrite(PIN_BUZZER, HIGH);
    delay(5000);
}

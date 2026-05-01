const uint8_t PIN_TILT = 2;

void setup() {
    pinMode(LED_BUILTIN, OUTPUT);
    pinMode(PIN_TILT, INPUT_PULLUP);
}

void loop() {
    if (digitalRead(PIN_TILT) == HIGH) {
        digitalWrite(LED_BUILTIN, LOW);
    } else {
        digitalWrite(LED_BUILTIN, HIGH);
    }
}

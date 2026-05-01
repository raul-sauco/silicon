// LD2420 Presence Detection → LED
// https://www.espboards.dev/sensors/ld2420/
// OT1 (GPIO output) mode — no UART parsing needed

constexpr uint8_t PRESENCE_PIN = 2;

void setup() {
    pinMode(PRESENCE_PIN, INPUT);
    pinMode(LED_BUILTIN, OUTPUT);
    // Serial.begin(115200); sensor UART rate
    Serial.begin(9600);
}

void loop() {
    bool presence = digitalRead(PRESENCE_PIN) == HIGH;
    digitalWrite(LED_BUILTIN, presence ? HIGH : LOW);
    if (presence) {
        Serial.println("Presence detected");
    }
    delay(100);
}

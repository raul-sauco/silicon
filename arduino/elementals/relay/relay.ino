// DC motor direction control via relay — pins 3/4 direction, pin 5 enable.

constexpr uint8_t ENABLE = 5;
constexpr uint8_t DIRA = 3;
constexpr uint8_t DIRB = 4;

void setup() {
    pinMode(ENABLE, OUTPUT);
    pinMode(DIRA, OUTPUT);
    pinMode(DIRB, OUTPUT);
    Serial.begin(9600);

    digitalWrite(ENABLE, HIGH);
}

void loop() {
    for (int i = 0; i < 5; i++) {
        Serial.println("spin");
        digitalWrite(DIRA, HIGH);
        digitalWrite(DIRB, LOW);
        delay(2000);

        Serial.println("stop");
        digitalWrite(DIRA, LOW);
        digitalWrite(DIRB, HIGH);
        delay(2000);
    }
}

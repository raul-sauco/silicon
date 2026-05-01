const uint8_t PIN_SW = 2;
const uint8_t PIN_X = A0;
const uint8_t PIN_Y = A1;

const uint16_t READ_INTERVAL = 500;

void setup() {
    pinMode(PIN_SW, INPUT_PULLUP);
    Serial.begin(9600);
    Serial.print("\033[2J\033[H");
}

void loop() {
    char buffer[64];
    sprintf(buffer, "\033[1HSwitch: %d   X: %4d   Y: %4d   ",
            digitalRead(PIN_SW), analogRead(PIN_X), analogRead(PIN_Y));
    Serial.print(buffer);
    delay(READ_INTERVAL);
}

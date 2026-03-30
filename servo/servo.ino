#include <Servo.h>

const uint8_t PIN_SERVO = 9;
const uint8_t SERVO_MIN = 0;
const uint8_t SERVO_MAX = 180;
const uint8_t SERVO_STEP = 1;
const uint8_t SERVO_DELAY = 15;

Servo servo;

void sweep(uint8_t from, uint8_t to) {
    if (from < to) {
        for (uint8_t pos = from; pos <= to; pos += SERVO_STEP) {
            servo.write(pos);
            delay(SERVO_DELAY);
        }
    } else {
        for (uint8_t pos = from; pos >= to; pos -= SERVO_STEP) {
            servo.write(pos);
            delay(SERVO_DELAY);
        }
    }
}

void setup() {
    servo.attach(PIN_SERVO);
}

void loop() {
    sweep(SERVO_MIN, SERVO_MAX);
    sweep(SERVO_MAX, SERVO_MIN);
}

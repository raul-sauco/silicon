/**
 * 28BYJ-48 Stepper Motor — One revolution CW, then CCW
 *
 * Uses AccelStepper instead of the built-in Stepper library:
 *   - Non-blocking: step() doesn't freeze the CPU for the full move
 *   - Acceleration / deceleration profiles (prevents missed steps)
 *   - Consistent API for both unipolar and bipolar motors
 *
 * Install: AccelStepper by Mike McCauley
 * https://www.airspayce.com/mikem/arduino/AccelStepper/
 *
 * Wiring (ULN2003 driver board → Arduino):
 *   IN1 → Pin 8
 *   IN2 → Pin 9
 *   IN3 → Pin 10
 *   IN4 → Pin 11
 */

#include <AccelStepper.h>

// ── Motor configuration ──────────────────────────────────────────────────────
// 28BYJ-48 in half-step mode: 2048 steps per revolution
// AccelStepper::HALF4WIRE = 4-wire unipolar, half-stepping
constexpr uint16_t STEPS_PER_REV = 2048;
constexpr float MAX_SPEED = 512.0f; // steps/sec (~15 rpm; hard limit ~17 rpm)
constexpr float ACCELERATION =
    256.0f; // steps/sec² — gentle ramp to avoid stalls

AccelStepper stepper(AccelStepper::HALF4WIRE, 8, 10, 9, 11);

// ── State machine ────────────────────────────────────────────────────────────
enum class MotorState {
    CLOCKWISE,
    PAUSE_AFTER_CW,
    COUNTERCLOCKWISE,
    PAUSE_AFTER_CCW
};
MotorState state = MotorState::CLOCKWISE;
uint32_t pauseStartedAt = 0;

constexpr uint16_t PAUSE_MS = 500;

// ── Arduino lifecycle ────────────────────────────────────────────────────────
void setup() {
    Serial.begin(9600);
    stepper.setMaxSpeed(MAX_SPEED);
    stepper.setAcceleration(ACCELERATION);
    stepper.moveTo(STEPS_PER_REV); // queue the first move
    Serial.println(F("Clockwise"));
}

void loop() {
    // run() must be called as frequently as possible — it steps the motor
    // one step at a time and returns true while a move is in progress.
    stepper.run();

    switch (state) {

    case MotorState::CLOCKWISE:
        if (stepper.distanceToGo() == 0) {
            pauseStartedAt = millis();
            state = MotorState::PAUSE_AFTER_CW;
        }
        break;

    case MotorState::PAUSE_AFTER_CW:
        if (millis() - pauseStartedAt >= PAUSE_MS) {
            stepper.moveTo(0); // back to origin = one rev CCW
            Serial.println(F("Counter-clockwise"));
            state = MotorState::COUNTERCLOCKWISE;
        }
        break;

    case MotorState::COUNTERCLOCKWISE:
        if (stepper.distanceToGo() == 0) {
            pauseStartedAt = millis();
            state = MotorState::PAUSE_AFTER_CCW;
        }
        break;

    case MotorState::PAUSE_AFTER_CCW:
        if (millis() - pauseStartedAt >= PAUSE_MS) {
            stepper.moveTo(STEPS_PER_REV);
            Serial.println(F("Clockwise"));
            state = MotorState::CLOCKWISE;
        }
        break;
    }
}

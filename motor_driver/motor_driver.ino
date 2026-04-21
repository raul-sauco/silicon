/**
 * DC Motor Driver — H-Bridge Demo (e.g. L293D / L298N)
 *
 * Wiring:
 *   Pin 3 (PWM) → DIRA — direction A
 *   Pin 4        → DIRB — direction B
 *   Pin 5 (PWM) → EN   — enable / speed (PWM)
 */

// ── Pin definitions ──────────────────────────────────────────────────────────
constexpr uint8_t PIN_ENABLE = 5; // PWM-capable — controls speed
constexpr uint8_t PIN_DIRA = 3;
constexpr uint8_t PIN_DIRB = 4;

// ── Motor direction constants ────────────────────────────────────────────────
enum class Direction { FORWARD, REVERSE };

// ── Motor control helpers ────────────────────────────────────────────────────

/** Stop the motor immediately (fast stop — both inputs LOW). */
void motorStop() {
    analogWrite(PIN_ENABLE, 0);
    digitalWrite(PIN_DIRA, LOW);
    digitalWrite(PIN_DIRB, LOW);
}

/**
 * Run the motor at a given speed and direction.
 * @param speed      0–255  (0 = off, 255 = full speed)
 * @param direction  Direction::FORWARD or Direction::REVERSE
 */
void motorRun(uint8_t speed, Direction direction) {
    if (direction == Direction::FORWARD) {
        digitalWrite(PIN_DIRA, HIGH);
        digitalWrite(PIN_DIRB, LOW);
    } else {
        digitalWrite(PIN_DIRA, LOW);
        digitalWrite(PIN_DIRB, HIGH);
    }
    analogWrite(PIN_ENABLE, speed);
}

/**
 * Coast the motor to a stop (slow stop — disable enable pin only).
 * Direction pins are left set so the motor decelerates naturally.
 */
void motorCoast() {
    analogWrite(PIN_ENABLE, 0);
}

// ── Demo sequences ───────────────────────────────────────────────────────────

/** Alternate direction 5 times, 500 ms each way. */
void demoBackAndForth() {
    Serial.println(F("Demo 1: Back and forth"));
    for (uint8_t i = 0; i < 5; i++) {
        motorRun(255, Direction::FORWARD);
        delay(500);
        motorRun(255, Direction::REVERSE);
        delay(500);
    }
    motorStop();
    delay(2000);
}

/** Demonstrate the difference between a coast stop and a fast stop. */
void demoStopModes() {
    Serial.println(F("Demo 2: Coast stop vs fast stop"));

    // Coast stop
    motorRun(255, Direction::FORWARD);
    delay(3000);
    motorCoast();
    delay(1000);

    // Fast stop
    motorRun(255, Direction::REVERSE);
    delay(3000);
    motorStop();
    delay(2000);
}

/** Ramp speed down then back up using PWM. */
void demoPWM() {
    Serial.println(F("Demo 3: PWM speed ramp down then up"));

    constexpr uint8_t steps[] = {255, 180, 128, 50, 128, 180, 255};
    constexpr uint16_t stepDelay = 2000;

    motorRun(steps[0], Direction::FORWARD);
    for (uint8_t i = 0; i < sizeof(steps); i++) {
        analogWrite(PIN_ENABLE, steps[i]);
        Serial.print(F("  Speed: "));
        Serial.println(steps[i]);
        delay(stepDelay);
    }
    motorStop();
    delay(10000);
}

// ── Arduino lifecycle ────────────────────────────────────────────────────────
void setup() {
    pinMode(PIN_ENABLE, OUTPUT);
    pinMode(PIN_DIRA, OUTPUT);
    pinMode(PIN_DIRB, OUTPUT);
    motorStop(); // ensure motor is off on boot
    Serial.begin(9600);
}

void loop() {
    demoBackAndForth();
    demoStopModes();
    demoPWM();
}

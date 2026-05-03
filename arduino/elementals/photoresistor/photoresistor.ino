#include <Arduino.h>

constexpr uint8_t PIN_INPUT = A0;
constexpr uint32_t BAUD_RATE = 9600;
constexpr uint16_t ADC_MAX = 1023;
constexpr uint16_t DELAY_MAX_MS = 1000;

// Map raw ADC reading to a sensible blink delay
inline uint16_t adcToDelay(uint16_t adc) {
    return map(adc, 0, ADC_MAX, 0, DELAY_MAX_MS);
}

void setup() {
    pinMode(LED_BUILTIN, OUTPUT);
    Serial.begin(BAUD_RATE);
}

void loop() {
    const uint16_t raw = analogRead(PIN_INPUT);
    const uint16_t delay_ms = adcToDelay(raw);

    digitalWrite(LED_BUILTIN, HIGH);
    delay(delay_ms);
    digitalWrite(LED_BUILTIN, LOW);
    delay(delay_ms);

    Serial.print(F("raw="));
    Serial.print(raw);
    Serial.print(F("  delay_ms="));
    Serial.println(delay_ms);
}

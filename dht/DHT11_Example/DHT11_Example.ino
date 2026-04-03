#include <dht_nonblocking.h>

const uint8_t PIN_DHT = 2;
const unsigned long MEASURE_INTERVAL = 3000;

DHT_nonblocking dht_sensor(PIN_DHT, DHT_TYPE_11);

static bool measure_environment(float &temperature, float &humidity) {
    static unsigned long measurement_timestamp = millis();

    if (millis() - measurement_timestamp > MEASURE_INTERVAL) {
        if (dht_sensor.measure(&temperature, &humidity)) {
            measurement_timestamp = millis();
            return true;
        }
    }
    return false;
}

void setup() {
    Serial.begin(9600);
}

void loop() {
    float temperature;
    float humidity;

    if (measure_environment(temperature, humidity)) {
        char temp_str[8];
        char hum_str[8];
        dtostrf(temperature, 4, 1, temp_str);
        dtostrf(humidity, 4, 1, hum_str);

        char buffer[32];
        sprintf(buffer, "T = %s C, H = %s%%", temp_str, hum_str);
        Serial.println(buffer);
    }
}

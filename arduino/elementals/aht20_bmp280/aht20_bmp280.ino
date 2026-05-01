#include <Adafruit_AHTX0.h>
#include <Adafruit_BMP280.h>

Adafruit_AHTX0 aht;
Adafruit_BMP280 bmp;

void setup() {
    Serial.begin(9600);

    if (!aht.begin()) {
        Serial.println("AHT20 not found");
        while (1)
            ;
    }
    Serial.println("AHT20 found");

    if (!bmp.begin(0x77)) {
        Serial.println("BMP280 not found");
        while (1)
            ;
    }
    Serial.println("BMP280 found");
}

void loop() {
    sensors_event_t humidity, temp_aht;
    aht.getEvent(&humidity, &temp_aht);

    char buffer[64];
    char temp_str[8];
    char hum_str[8];
    char pres_str[10];

    dtostrf(temp_aht.temperature, 4, 1, temp_str);
    dtostrf(humidity.relative_humidity, 4, 1, hum_str);
    dtostrf(bmp.readPressure() / 100.0, 6, 2, pres_str);

    sprintf(buffer, "T: %sC  H: %s%%  P: %shPa", temp_str, hum_str, pres_str);
    Serial.println(buffer);

    delay(500);
}

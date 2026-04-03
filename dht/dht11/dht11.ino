#include <DHT.h>

DHT dht(2, DHT11);

void setup() {
    Serial.begin(9600);
    dht.begin();
    delay(3000);
}

void loop() {
    Serial.println(dht.readTemperature());
    Serial.println(dht.readHumidity());
    delay(3000);
}

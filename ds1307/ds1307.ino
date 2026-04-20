#include <RTClib.h>

RTC_DS1307 rtc;

void setup() {
    Serial.begin(9600);
    Wire.begin();

    if (!rtc.begin()) {
        Serial.println("DS1307 not found");
        while (1)
            ;
    }

    if (!rtc.isrunning()) {
        Serial.println("RTC not running, setting time");
        // Sets to compile time automatically
        rtc.adjust(DateTime(F(__DATE__), F(__TIME__)));
    }

    Serial.println("DS1307 found and running");
}

void loop() {
    DateTime now = rtc.now();

    char buffer[32];
    sprintf(buffer, "%04d-%02d-%02d %02d:%02d:%02d", now.year(), now.month(),
            now.day(), now.hour(), now.minute(), now.second());
    Serial.println(buffer);

    delay(1000);
}

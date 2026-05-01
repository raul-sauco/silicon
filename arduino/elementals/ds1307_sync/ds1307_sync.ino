#include <RTClib.h>
#include <Wire.h>

RTC_DS1307 rtc;

// Sync the RTC time with the computer time sending it via serial
// `date +"%Y-%m-%d %H:%M:%S" > /dev/ttyACM0`
void setup() {
    Serial.begin(9600);
    Wire.begin();

    if (!rtc.begin()) {
        Serial.println("DS1307 not found");
        while (1)
            ;
    }
    Serial.println("Send time as: YYYY-MM-DD HH:MM:SS");
}

void loop() {
    if (Serial.available()) {
        String input = Serial.readStringUntil('\n');
        if (input.length() >= 19) {
            uint16_t year = input.substring(0, 4).toInt();
            uint8_t month = input.substring(5, 7).toInt();
            uint8_t day = input.substring(8, 10).toInt();
            uint8_t hour = input.substring(11, 13).toInt();
            uint8_t minute = input.substring(14, 16).toInt();
            uint8_t second = input.substring(17, 19).toInt();
            rtc.adjust(DateTime(year, month, day, hour, minute, second));
            Serial.println("Time set");
        }
    }

    DateTime now = rtc.now();
    char buffer[32];
    sprintf(buffer, "%04d-%02d-%02d %02d:%02d:%02d", now.year(), now.month(),
            now.day(), now.hour(), now.minute(), now.second());
    Serial.println(buffer);
    delay(1000);
}

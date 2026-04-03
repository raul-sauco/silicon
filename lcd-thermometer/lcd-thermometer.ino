#include <LiquidCrystal.h>

constexpr uint8_t PIN_THERMISTOR = A0;
constexpr uint8_t LCD_RS = 7, LCD_EN = 8;
constexpr uint8_t LCD_D4 = 9, LCD_D5 = 10, LCD_D6 = 11, LCD_D7 = 12;

constexpr float SERIES_RESISTOR = 10000.0f;
constexpr float STEINHART_A = 0.001129148f;
constexpr float STEINHART_B = 0.000234125f;
constexpr float STEINHART_C = 0.0000000876741f;

LiquidCrystal lcd(LCD_RS, LCD_EN, LCD_D4, LCD_D5, LCD_D6, LCD_D7);

float readTempC() {
    int raw = analogRead(PIN_THERMISTOR);
    float lnR = log(SERIES_RESISTOR * (1024.0f / raw - 1.0f));
    float tempK =
        1.0f / (STEINHART_A + (STEINHART_B + STEINHART_C * lnR * lnR) * lnR);
    return tempK - 273.15f;
}

void setup() {
    lcd.begin(16, 2);
    Serial.begin(9600);
}

void loop() {
    float tempC = readTempC();
    Serial.println(tempC);
    lcd.setCursor(0, 0);
    lcd.print("Temp         C  ");
    lcd.setCursor(6, 0);
    lcd.print(tempC);
    delay(500);
}

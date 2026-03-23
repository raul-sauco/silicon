// Pins
#define PIN_RED   6
#define PIN_GREEN 5
#define PIN_BLUE  3

// Timing
const uint8_t FADE_DELAY = 10;

void setRGB(uint8_t r, uint8_t g, uint8_t b) {
  analogWrite(PIN_RED,   r);
  analogWrite(PIN_GREEN, g);
  analogWrite(PIN_BLUE,  b);
}

void fade(uint8_t fromR, uint8_t fromG, uint8_t fromB,
          uint8_t toR,   uint8_t toG,   uint8_t toB) {
  for (int i = 0; i < 255; i++) {
    uint8_t r = fromR + (toR - fromR) * i / 254;
    uint8_t g = fromG + (toG - fromG) * i / 254;
    uint8_t b = fromB + (toB - fromB) * i / 254;
    setRGB(r, g, b);
    delay(FADE_DELAY);
  }
}

void setup() {
  pinMode(PIN_RED,   OUTPUT);
  pinMode(PIN_GREEN, OUTPUT);
  pinMode(PIN_BLUE,  OUTPUT);
  setRGB(255, 0, 0);
}

void loop() {
  fade(255, 0,   0,   0, 255, 0);
  fade(0,   255, 0,   0, 0,   255);
  fade(0,   0,   255, 255, 0, 0);
}

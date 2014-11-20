#include <stdio.h>
#include <math.h>
#include <Wire.h>
#include <SPI.h>
#include <SdFat.h>
#include "Adafruit_MPL3115A2.h"
#include "Adafruit_GFX.h"
#include "Adafruit_ILI9341.h"
#include "Adafruit_GPS.h"
#include "MemoryFree.h"
#include "SoftwareSerial.h" // not used
#include "types.h"


// For the Adafruit shield, these are the default.
#define TFT_DC 9
#define TFT_CS 10
#define SD_CS 4
#define SPEEDOMETER 4 //int
#define JOYSTICK 5 //int
#define JOYSTICK_X 14
#define JOYSTICK_Y 15

// Use hardware SPI (on Uno, #13, #12, #11) and the above for CS/DC
Adafruit_ILI9341 tft = Adafruit_ILI9341(TFT_CS, TFT_DC);

// Create an instance of the pressure object
Adafruit_MPL3115A2 baro = Adafruit_MPL3115A2();

// Set up GPS
Adafruit_GPS GPS(&Serial2);

// keep track of speedometer
volatile unsigned long last_cycle;
volatile unsigned long cycle_time;

// joystick center
int x_centre, y_centre;
double zoom = 10000;
volatile bool streetnames = false;

// file system
SdFat sd;
// test file
SdFile file;
// file extent
uint32_t bgnBlock, endBlock;

void setup()
{
  Wire.begin();       // Join i2c bus
  Serial.begin(9600); // Start serial for output
  baro.begin();       // Get sensor online
  tft.begin();        // Start TFT
  GPS.begin(9600);    // Start GPS
  // Mount SD card
  if (!sd.begin(SD_CS, SPI_FULL_SPEED)) sd.initErrorHalt();
 
  // uncomment this line to turn on RMC (recommended minimum) and GGA (fix data) including altitude
  GPS.sendCommand(PMTK_SET_NMEA_OUTPUT_RMCGGA);
  // uncomment this line to turn on only the "minimum recommended" data
  //GPS.sendCommand(PMTK_SET_NMEA_OUTPUT_RMCONLY);
  // For parsing data, we don't suggest using anything but either RMC only or RMC+GGA since
  // the parser doesn't care about other sentences at this time
  
  // Set the update rate
  GPS.sendCommand(PMTK_SET_NMEA_UPDATE_1HZ);   // 1 Hz update rate
  // For the parsing code to work nicely and have time to sort thru the data, and
  // print it out we don't suggest using anything higher than 1 Hz

  // Request updates on antenna status, comment out to keep quiet
  //GPS.sendCommand(PGCMD_ANTENNA);

  // the nice thing about this code is you can have a timer0 interrupt go off
  // every 1 millisecond, and read data from the GPS for you. that makes the
  // loop code a heck of a lot easier!
  // Timer0 is already used for millis() - we'll just interrupt somewhere
  // in the middle and call the "Compare A" function
  OCR0A = 0xAF;
  TIMSK0 |= _BV(OCIE0A);
  
  // speedometer
  attachInterrupt(SPEEDOMETER, speedometer, FALLING);
  last_cycle = millis();
  cycle_time = 1000;
  
  // joystick
  attachInterrupt(JOYSTICK, buttonpress, FALLING);
  x_centre = analogRead(JOYSTICK_X);
  y_centre = analogRead(JOYSTICK_Y);

  tft.setTextSize(1);
  tft.fillScreen(ILI9341_BLACK);
  tft.setRotation(1);
  tft.setTextColor(ILI9341_WHITE);
  tft.setTextWrap(false);
}

void loop()
{
  Serial.print("freeMemory()=");
  Serial.println(freeMemory());
  float pressure = baro.getPressure();
  float temperature = baro.getTemperature();
  // if a sentence is received, we can check the checksum, parse it...
  if (GPS.newNMEAreceived()) {
    // this also sets the newNMEAreceived() flag to false
    GPS.parse(GPS.lastNMEA());
  }
  
  // logging stuff
  /*
  char filename[13];
  snprintf(filename, 13, "%d-%d-%d.log", GPS.year, GPS.month, GPS.day);
  File dataFile = SD.open(filename, FILE_WRITE);
  
  if (dataFile) {
    dataFile.println(String() +
    GPS.latitude_fixed + "," + GPS.lat + "," +
    GPS.longitude_fixed + "," + GPS.lon + "," +
    pressure + "," + temperature + "," + cycle_time
    );
    dataFile.close();
  }*/
  
  if (GPS.fix) {
    // this is not correct
    double lat_zoom = zoom / ((double)tft.width() / tft.height());
    double lon_zoom = zoom / lon_scale(GPS.latitude_fixed);
    Serial.println(lon_scale(GPS.latitude_fixed));
    Serial.println((double)tft.width() / tft.height());
    
    Rect bounds = {.sub = 0,
                   .x0 = GPS.longitude_fixed - lon_zoom,
                   .y0 = GPS.latitude_fixed - zoom,
                   .x1 = GPS.longitude_fixed + lon_zoom,
                   .y1 = GPS.latitude_fixed + zoom};
    rtree_lookup(&bounds);
    tft.fillCircle(tft.width()/2, tft.height()/2, 4, ILI9341_BLUE);
  }
  
  tft.setCursor(0, 0);
  tft.setTextSize(1);
  tft.setTextColor(ILI9341_WHITE);
  tft.print(pressure, 2);
  tft.println(" Pa");
  tft.print(temperature, 2);
  tft.println(" C");
  tft.print(60000 / cycle_time);
  tft.println(" RPM");

  tft.setTextSize(3);
  tft.setCursor(100, 200);
  tft.print(GPS.speed * 1.852, 2);
    tft.println(" Km/h");

  unsigned long time = millis() + 10000;
  bool btn = streetnames;
  while (millis() < time && btn == streetnames) {
    int y = y_centre - analogRead(JOYSTICK_Y);
    if (y > 20) {
      zoom *= 0.8;
      break;
    } else if (y < -20) {
      zoom *= 1.3;
      break;
    }
  }
  /*tft.println("Joystick:");
  tft.print("X: ");
  tft.print(x_centre - analogRead(JOYSTICK_X));
  tft.print(" Y: ");
  tft.print(y_centre - analogRead(JOYSTICK_Y));*/
  tft.fillScreen(ILI9341_BLACK);
  tft.setCursor(0, 0);
  tft.setTextSize(1);
}

//radians = (degrees * 71) / 4068
//degrees = (radians * 4068) / 71
double lon_scale(int32_t lat) {
  double dlat = lat / 10000000.0;
  double rad = dlat * 0.0174532925;
  return cos(rad);
}

// Interrupt is called once a millisecond, looks for any new GPS data, and stores it
SIGNAL(TIMER0_COMPA_vect) {
  GPS.read();
}

void speedometer() {
  unsigned long time = millis();
  if (time - last_cycle > 100) {
    cycle_time = time - last_cycle;
    last_cycle = time;
  }
}

void buttonpress() {
  Serial.println("button");
  streetnames = !streetnames;
}

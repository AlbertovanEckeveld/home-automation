
#include <time.h>                                               // include de time library

void setupTime() {                                              // 
  // Nederlandse tijd
  configTzTime(
    "CET-1CEST,M3.5.0/02,M10.5.0/03",
    "pool.ntp.org",
    "time.nist.gov"
  );

  struct tm timeinfo;

  Serial.print("Wachten op tijdsync");

  while (!getLocalTime(&timeinfo)) {
    Serial.print(".");
    delay(1000);
  }

  Serial.println();
  Serial.println("Tijd gesynchroniseerd");
}

void updateTime() {

  if (millis() - laatsteTimeSync >= TIME_SYNC_INTERVAL) {

    Serial.println("Tijd opnieuw synchroniseren...");

    configTzTime(
      "CET-1CEST,M3.5.0/02,M10.5.0/03",
      "pool.ntp.org",
      "time.nist.gov"
    );

    struct tm timeinfo;

    if (getLocalTime(&timeinfo, 5000)) {
      Serial.println("Synchronisatie gelukt");
      laatsteTimeSync = millis();
    } else {
      Serial.println("Synchronisatie mislukt");
    }
  }
}

void printCurrentTime() {
  struct tm timeinfo;
  char buffer[32];

  if (getLocalTime(&timeinfo)) {

    strftime(buffer, sizeof(buffer),
             "%d-%m-%Y %H:%M:%S",
             &timeinfo);

    Serial.println(buffer);

  } else {
    Serial.println("Geen tijd beschikbaar");
  }
}

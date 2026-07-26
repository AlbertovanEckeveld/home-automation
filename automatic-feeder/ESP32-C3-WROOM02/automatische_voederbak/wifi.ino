  
#include <WiFi.h>                                                   // include de WiFi library

bool connectWifi() {                                                // functie die de WiFi verbinding opzet
  WiFi.mode(WIFI_STA);                                              // WiFi modus WiFi_STA

  WiFi.setHostname("AlbertoVE-voederbak");                          // zet de hostname van de ESP32

  WiFi.begin(ssid, password);                                       // probeer een verbinding op te zetten met de gegeven SSID en wachtwoord

  Serial.print("Verbinden met WiFi");

  while (WiFi.status() != WL_CONNECTED) {                           // genereer in de serial een stippel lijntje zolang de verbinding nog niet gemaakt is
    delay(500);
    Serial.print(".");
  }

  Serial.println();
  Serial.println("Verbonden");

  Serial.print("IP-adres: ");
  Serial.println(WiFi.localIP());

  return true;                                                      // return true als de verbinding is gemaakt
}

  
bool checkWifi() {                                                  // functie die de WiFi verbinding controleert en herstelt als de verbinding verloren is

  if (WiFi.status() == WL_CONNECTED) {                              // return true als de verbinding nog gewoon actief is
    return true;
  }

  if (millis() - laatsteWifiCheck >= WIFI_CHECK_INTERVAL) {         // als de verbinding niet actief is, controleer met WIFI_CHECK_INTERVAL het weer gecontroleerd wordt

    laatsteWifiCheck = millis();                                    // zet de nieuwe WIFI_CHECK_INTERVAL tijd

    Serial.println("WiFi kwijt, opnieuw verbinden...");
    connectWifi();                                                  // probeer weer de verbinding op te zetten
  }

  return false;
}

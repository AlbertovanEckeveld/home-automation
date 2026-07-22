
void setupSensors() {                                               // functie die alle sensor GPIO's als input definieerd
  for (int i = 0; i < 4; i++) {                                     
    pinMode(sensors[i], INPUT);                                     
  };

  Serial.println("sensors ingesteld.");
};

//bool motor1Dicht() {                                              // functie die true of false returned als de sensor dedecteerd dat de klep van motor 1 dicht is (wordt niet gebruikt)
//  return digitalRead(SENSOR_MOTOR1_DICHT) == LOW;
//};

bool motor1Open() {                                                 // functie die true of false returned als de sensor dedecteerd dat de klep van motor 1 open is
  return digitalRead(SENSOR_MOTOR1_OPEN) == LOW;
};

//bool motor2Dicht() {                                              // functie die true of false returned als de sensor dedecteerd dat de klep van motor 2 dicht is (wordt niet gebruikt)
//  return digitalRead(SENSOR_MOTOR2_DICHT) == LOW;
//};

bool motor2Open() {                                                 // functie die true of false returned als de sensor dedecteerd dat de klep van motor 2 open is
  return digitalRead(SENSOR_MOTOR2_OPEN) == LOW;
};

bool beideOpen() {                                                  // functie die true of false returned als beide sensor dedecteerd dat de kleppen van motor 1 en 2 open zijn 
  return motor1Open() && motor2Open();
};

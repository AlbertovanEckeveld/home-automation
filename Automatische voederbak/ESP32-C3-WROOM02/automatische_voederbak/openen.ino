
void startMotor1Open() {

  if (motor1.defect) {
    return;
  };

  motor1.actief = true;
  motor1.startTijd = millis();

  aanzettenMotor1OpenDraaien();

  Serial.println("Motor 1 openen");
};

void startMotor2Open() {

  if (motor2.defect) {
    return;
  };

  motor2.actief = true;
  motor2.startTijd = millis();

  aanzettenMotor2OpenDraaien();

  Serial.println("Motor 2 openen");
};

void startOpenen() {

  Serial.println("Open opdracht");

  motor1.openBevestigd = false;
  motor2.openBevestigd = false;

  startMotor1Open();
  startMotor2Open();

  status = OPENEN;
};

void verwerkMotor1Openen() {

  if (!motor1.actief) {
    return;
  };

  if (motor1Open()) {

    uitzettenMotor1OpenDraaien();

    motor1.actief = false;
    motor1.openBevestigd = true;
    motor1.openPogingen = 0;

    Serial.println("Motor 1 open bevestigd");

    return;
  };

  if (millis() - motor1.startTijd > MOTOR1_OPEN_TIMEOUT) {

    uitzettenMotor1OpenDraaien();

    motor1.actief = false;

    motor1.openPogingen++;

    if (motor1.openPogingen < MAX_OPEN_POGINGEN) {

      Serial.println("Motor 1 nieuwe openingspoging");

      startMotor1Open();
      
    } else {

      motor1.defect = true;

      Serial.println("Motor 1 defect");
    };
  };
};

void verwerkMotor2Openen() {

  if (!motor2.actief) {
    return;
  };

  if (motor2Open()) {

    UitzettenMotor2OpenDraaien();

    motor2.actief = false;
    motor2.openBevestigd = true;
    motor2.openPogingen = 0;

    Serial.println("Motor 2 open bevestigd");

    return;
  };

  if (millis() - motor2.startTijd > MOTOR2_OPEN_TIMEOUT) {

    UitzettenMotor2OpenDraaien();

    motor2.actief = false;

    motor2.openPogingen++;

    if (motor2.openPogingen < MAX_OPEN_POGINGEN) {

      Serial.println("Motor 2 nieuwe openingspoging");

      startMotor2Open();
      
    } else {

      motor2.defect = true;

      Serial.println("Motor 2 defect");
    };
  };
};

void verwerkOpenen() {

  if (status != OPENEN) {
    return;
  };

  verwerkMotor1Openen();
  verwerkMotor2Openen();

  if (motor1.defect) {

    Serial.println("Openen mislukt: Motor1 defect");

    status = FOUT;

    return;
  };

  if (motor2.defect) {

    Serial.println("Openen mislukt: Motor2 defect");

    status = FOUT;

    return;
  };

  if (motor1.openBevestigd &&
      motor2.openBevestigd) {

    Serial.println("Beide kleppen open");

    status = OPEN;
    
    allRelaysOff();
  };
};

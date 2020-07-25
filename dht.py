#!/usr/bin/env python3
import time
import board
import adafruit_dht

dhtDevice = adafruit_dht.DHT11(board.D21)

while True:
    try:
        temperature_c = dhtDevice.temperature
        humidity = dhtDevice.humidity
        while temperature_c > 36:
            temperature_c = dhtDevice.temperature
            humidity = dhtDevice.humidity
        print("Temp: {:.1f} C  Humidity: {}%    \r".format(
            temperature_c, humidity
        ), end='')

    except RuntimeError as error:
        pass

    time.sleep(1.0)

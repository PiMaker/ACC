# ACC
AC remote control for Inventor Aria models via Infrared.

Encoding of IR signals was reverse-engineered on my own "Aria" AC, might need adjustment for other variants (more details in `scratchpad.txt`). Use at your own discretion.

Use `acc help` and `acc help send` for usage information.

## Additional files
The project relies on pigpio for sending codes via an attached IR LED. The file `irrp.py` from pigpio's examples is used for sending.

`dht.py` can be used as an example for monitoring room temperature via a DHT11/DHT12 model temperature and humidity sensor, but is unrelated to the rest of the project.

# License

This project is available under the terms of the MIT license. More information can be found in the file `LICENSE`.

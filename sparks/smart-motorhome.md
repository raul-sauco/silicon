# Smart Motorhome

## Main PCB repair

Need to fix the main control PCB, or display

The Nordelettronica NE185 is a complex "shunt" or distribution unit, and finding
the internal board-level schematic (the trace layout and component values) is
difficult because the manufacturer treats it as a non-serviceable part.

However, based on technical documentation for the NE185-S and NE185-T series,
here is the specific pinout and diagnostic information for the Power Input and
Display Connection you are looking for.

### Power Input Connections (Main Terminals)

If you have a total power loss, check the high-current input studs on the board.

- J1 (Automotive/Starter Battery): This is the input from the van's main
  battery. It is typically a large spade or bolt-on terminal.
- J2 (Service/Habitation Battery): This is the main input for your living area power.
- J3 / J4: These are usually the common negative (Ground) returns.

> Diagnostic Tip: Check the voltage between J2 and Ground. If there is no
> voltage, the issue is likely a "Mega Fuse" (typically 50A-100A) located near
> your leisure battery, not the board itself.

### Display / Control Panel Connection (JP11)

The connection to your wall-mounted display (like the NE172 or NE266) is handled
by a dedicated 4-pin port labeled JP11.

    Connector Type: 4-pole serial connector.

    Communication: This is a serial data link. If the display is blank but the
    board has power, check for a loose connection at JP11.

    The "10V Shutoff": If your service battery drops below 10V for more than 60
    seconds, the NE185 will automatically kill the connection to the display and
    all services to protect the battery. You must recharge the battery above 11V
    before the display will respond again.

### Pinout Legend for Small Signal Blocks

If you are tracing specific triggers (like why the display isn't showing "Mains
Connected" or "Engine Running"):

    JP6 (Pin 1): D+ Input (from the alternator). This signals the board to link the batteries and turn on the fridge.

    JP13 (Pin 3): Ignition/Key-on signal.

    JP13 (Pin 2): Mains presence signal (from the battery charger).

Best Websites for the PDF Schematics

If you need the full wiring diagrams (which show which colored wires go to which
pin), these three sites host the official PDF "Training Manuals":

- A&N Caravan Services: They have a dedicated "Resources" page with the most
  accurate repair manuals for Nordelettronica units in the UK/EU.
- Camperpunt.nl: Hosts a "Kit 2006" manual that includes the full internal logic
  and wiring for the NE185-S.
- ManualsLib: Search for "Nordelettronica NE185-15S" to get the 16-page
  technical layout.

Reference Video: If you want to see the physical layout of these JP connectors,
this video shows the board being probed:
[REPARACION CENTRALITA NORDELETTRONICA NE 185](https://www.youtube.com/watch?v=2wb2FjD63Jo).

### Diagnosing the Nordelettronica NE185

Set your lab power supply to 13.5V and limit the current to 1.0A for
bench testing.

---

#### 1. Power Input & Rail Stability

- Main Input: Connect (+) to J2 and (-) to J3 or J4.
- Current Check: Normal standby draw is 50mA to 150mA. If 0mA, check the
  input protection diode near J2.
- Logic Rail: Locate the voltage regulator. Measure the output pin; you
  must see a steady 5V DC. If missing, the logic will not boot.

#### 2. Display Communication (JP11 Port)

- Power Pins: Check the outer pins of JP11. You should see 12V.
  - If 0V: Check for a blown SMD fuse or burnt trace near the port.
- Data Pins: Inner pins should fluctuate between 3.3V and 5V, showing
  active serial communication.

#### 3. Signal Simulation (Triggers)

Apply a 12V signal to these pins to test board logic:

- Mains Presence: Apply 12V to JP13 Pin 2.
- Ignition/D+: Apply 12V to JP6 Pin 1. You should hear the battery
  coupling relay click.

#### 4. Physical Board Integrity

- Relay Test: Check coils with a multimeter. Healthy coils read between
  300 and 700 ohms.
- Solder Fatigue: Check for "ring" cracks on relay pins and high-current
  terminals caused by vibration and heat cycling.

## Voice recognition

Have the ability to use voice commands to interact with the motorhome.

The fact that the space is small should help carrying audio to the receiver.

### Hardware

- **ESP-S3**: Better AI integration should help run ESP-SR.
- **I2S Digital Microphone**: You cannot use a standard analog mic without an
  external ADC. The **INMP441** or **SPH0645** are the industry standards for
  ESP32 projects. They provide a digital I2S signal that the S3 can process directly.
- **Audio Output** (Optional but recommended): If you want the ESP to "talk back"
  or beep when it understands a command, you’ll need:
- **I2S DAC/Amplifier**: The MAX98357A is a popular "plug-and-play" mono amplifier.
- **Speaker**: A small 4Ω or 8Ω (2W-5W) speaker.
- **Stable Power Supply**: Voice recognition causes CPU spikes. Powering via a
  weak laptop USB port can sometimes cause crashes; a 5V 2A wall adapter is safer.

### Software Stack

Espressif’s voice recognition isn't just one library; it's a tiered system called
ESP-Skainet.

Component Function:

- WakeNet Listens for a specific "Wake Word" (e.g., "Hi ESP"). This saves power
  by not processing everything constantly.
- MultiNet This is what you need. It recognizes the 5-6 commands after the wake
  word is detected.
- AFE (Audio Front-End) This handles echo cancellation and noise suppression so
  the ESP can "hear" you over background noise.

### Pro-Tips for Beginners

The "Box" Shortcut: If the wiring feels daunting, look at the ESP32-S3-BOX-3.
It’s a ready-made kit from Espressif that has the mic, speaker, and S3 chip
already integrated into a neat case.

Command Customization: With MultiNet, you don't have to train the model with
your own voice. You simply define the commands in a text list (e.g., "Turn on
the light"), and the neural network handles the phonetic matching.

### Useful links

[How to build an AI Voice Assistant using ESP32-S3](https://www.youtube.com/watch?v=wzYkzo-9ijo)

This video provides a practical walkthrough of connecting the I2S microphone and
amplifier to an ESP32-S3 and setting up the firmware for voice interaction.

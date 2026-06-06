# A quick guide on testing 18650 batteries

Probably works for other batteries adjusting the voltage.

## Step 1: The Instant Knockout (Resting Voltage)

Multimeter test

- 3.0V to 4.2V: The cell is healthy and safely within its operational range. Move to Step 2.
- 2.5V to 3.0V: Deeply discharged, but salvageable. Charge it at a very slow current (∼100mA) until it hits 3.5V, then test it normally.
- Below 2.0V: Throw it away. When a lithium-ion cell sits this low for too long, copper microscopic bridges (dendrites) grow inside the chemistry. If you try to recharge a cell like this, it can develop a permanent internal short circuit, turn into a "heater," and cause a fire.

## Step 2: The "Heater" Test (Self-Discharge Check)

Put it into a standard charger (like a TP4056 module or a smart bench charger) and charge it fully to 4.2V.

- Monitor the temperature: Touch the battery with your finger while it charges. If the cell feels uncomfortably hot to the touch (above 45∘C), unplug it immediately. It has a high internal resistance or an internal short. Throw it away.
- The 48-Hour Rest: Once fully charged to 4.2V, take it off the charger and lay it flat on your workbench for 48 hours.
- The Re-test: Measure the voltage after 2 days. A healthy cell will sit right around 4.15V to 4.20V. If the voltage has dropped down below 4.0V just by sitting idle, it is a "self-discharger." It will drain itself flat in a campervan environment without any load connected. Throw it away.

## Step 3: The Load Test (Checking Voltage Sag)

Test cell behaviour when an application pulls power.

- Measure the exact no-load voltage of the fully charged cell _Vstart_. It should be around 4.2V.
- Connect a high-power power resistor, ideally a 4.7Ω or 10Ω ceramic cement resistor rated for 5W or more, across the battery terminals using alligator clips while keeping multimeter probes attached to the cell. Leave it connected for exactly 5 seconds and record the voltage under load _Vload_.

Analyze the results:

- Healthy Cell: The voltage should barely flinch, sagging slightly to maybe 4.0V or 4.1V before holding perfectly steady.
- Bad Cell: If the voltage instantly plummets straight down to 3.5V, 3.2V, or lower, the cell's internal resistance is completely shot. It will brown out your ESP32-C3 the exact millisecond the LoRa radio attempts to transmit. Throw it away.

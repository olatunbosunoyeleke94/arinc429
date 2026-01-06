import jsbsim
import math
import json
import subprocess
import os

# Path to your compiled Rust binary (adjust if needed)
ENCODER_PATH = "./target/release/arinc_encoder"  # or "debug" for faster builds

if not os.path.exists(ENCODER_PATH):
    print(f"Error: Rust encoder not found at {ENCODER_PATH}")
    print("Run: cargo build --release --bin arinc_encoder")
    exit(1)

fdm = jsbsim.FGFDMExec(None)
fdm.set_debug_level(0)  # Quiet startup

fdm.load_script("scripts/c1723.xml")
fdm.run_ic()

print("JSBSim + arinc429 integration started")
print("Encoding flight data into ARINC 429 words every ~1 second:\n")

step = 0
while fdm.run() and step < 10000:
    if step % 100 == 0:
        sim_time = fdm.get_sim_time()
        altitude_ft = fdm.get_property_value("position/h-sl-ft")
        groundspeed_kts = fdm.get_property_value("velocities/vg-kts")
        mach = fdm.get_property_value("velocities/mach")
        tat_c = fdm.get_property_value("environment/temperature-tat-degc")
        roll_deg = math.degrees(fdm.get_property_value("attitude/roll-rad"))

        # Prepare raw integer values for encoding
        data = {
            "GroundSpeed": round(groundspeed_kts / 0.125),
            "PressureAltitude": round(altitude_ft),
            "Mach": round(mach / 0.001),
            "Tat": round(tat_c / 0.25),
            "RollAngle": round(roll_deg / 0.01),
        }

        input_json = json.dumps(data)

        result = subprocess.run(
            [ENCODER_PATH],
            input=input_json,
            capture_output=True,
            text=True,
	    check=False
        )

        if result.returncode == 0:
            encoded = json.loads(result.stdout)
            print(f"t={sim_time:6.1f}s | Alt:{altitude_ft:7.0f}ft | GS:{groundspeed_kts:5.1f}kts")
            for label, hex_word in encoded.items():
                octal = {
                    "GroundSpeed": "012",
                    "PressureAltitude": "203",
                    "Mach": "205",
                    "Tat": "211",
                    "RollAngle": "324",
                }.get(label, "???")
                print(f"  {label:18} [{octal}] → 0x{hex_word}")
            print()
        else:
            print("Encoder failed:", result.stderr)

    step += 1

print("Simulation complete!")

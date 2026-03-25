import platform
import subprocess
import os
import sys

def main():

    host_os = platform.system()

    cd = os.path.dirname(os.path.abspath(__file__))

    wasabi = sys.argv[1:]

    if host_os == "Linux":
        script_path = os.path.join(cd, "launch_qemu.sh")
        print(f"Running on linux: {script_path}")
        subprocess.run(["bash", script_path] + wasabi)

    elif host_os == "Windows":
        script_path = os.path.join(cd, "launch_qemu.bat")
        print(f"Running on Windows: {script_paht}")
        subprocess.run([script_path] + wasabi, shell=True)

    else:
        print(f"Unsupported: {host_os}")
        sys.exit(1)

if __name__ == "__main__":
    main()
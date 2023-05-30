import os
import sys
import sysconfig
from pathlib import Path


def get_binary() -> Path:
    exe = "spvn" + sysconfig.get_config_var("EXE")
    path = Path(sysconfig.get_path("scripts")) / exe
    if path.is_file():
        return path
    user_scheme = sysconfig.get_preferred_scheme("user")
    path = Path(sysconfig.get_path("scripts", scheme=user_scheme)) / exe
    if path.is_file():
        return path

    raise FileNotFoundError(path)


if __name__ == "__main__":
    spvn = get_binary()
    sys.exit(os.spawnv(os.P_WAIT, spvn, ["spvn", *sys.argv[1:]]))

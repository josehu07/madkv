# Routines that help simplify the `service` just recipes.

import sys


def partid(args):
    """
    Example: node = "s1"
    Returns: "1"
    """
    if len(args) < 1:
        raise ValueError("missing argument to 'partid'")

    node = args[0]
    if node.startswith("s"):
        return node[1:]
    else:
        return "0"


def portof(args):
    """
    Example: servers = "1.2.3.4:3777,5.6.7.8:3778,9.10.11.12:3779" node = "s1"
    Returns: "3778"
    """
    if len(args) < 2:
        raise ValueError("missing argument to 'portof'")

    servers = args[0]
    ports = list(map(lambda a: a[a.find(":") + 1 :], servers.split(",")))

    node = int(partid(args[1:]))
    if node >= len(ports):
        return ""  # ignore out-of-bounds here

    port = ports[node].strip()
    if len(port) == 0:
        raise ValueError(f"node {node}'s API port is empty")

    return port


def main():
    if len(sys.argv) <= 1:
        raise ValueError("missing argument to 'justmod/xtract.py'")

    if sys.argv[1] == "partid":
        print(partid(sys.argv[2:]))
    elif sys.argv[1] == "portof":
        print(portof(sys.argv[2:]))


if __name__ == "__main__":
    main()

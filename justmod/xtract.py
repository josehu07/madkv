# Routines that help simplify the `service` just recipes.

import sys


def partid(args):
    """
    Example: node = "s1.2"
    Returns: "1"
    """
    if len(args) < 1:
        raise ValueError("missing argument to 'partid'")

    node = args[0]
    if node.startswith("s"):
        doti = node.find(".")
        if doti == -1:
            return node[1:]
        else:
            return node[1 : node.find(".")]
    else:
        return "0"


def repid(args):
    """
    Example: node = "s1.2"
    Returns: "2"
    """
    if len(args) < 1:
        raise ValueError("missing argument to 'repid'")

    node = args[0]
    doti = node.find(".")
    if doti == -1:
        return "0"
    else:
        return node[doti + 1 :]


def portof(args):
    """
    Example: servers = "1.2.3.4:3777,5.6.7.8:3778,9.10.11.12:3779" node = "s0.1" rf = "3"
    Returns: "3778"
    """
    if len(args) < 2:
        raise ValueError("missing argument to 'portof'")

    servers = args[0]
    ports = list(map(lambda a: a[a.find(":") + 1 :], servers.split(",")))

    pid = int(partid(args[1:]))
    rid = int(repid(args[1:]))
    rf = 1
    if len(args) >= 3:
        rf = int(args[2])
    elif pid == 0:  # may be managers list
        rf = len(ports)

    node = pid * rf + rid
    if node >= len(ports):
        return ""  # ignore out-of-bounds here

    port = ports[node].strip()
    if len(port) == 0:
        raise ValueError(f"node {args[1]}'s API port is empty")

    return port


def peersof(args):
    """
    Example: servers = "1.2.3.4:3707,5.6.7.8:3708,9.10.11.12:3709" node = "s0.1" rf = "3"
    Returns: "1.2.3.4:3707,9.10.11.12:3709"
    """
    if len(args) < 2:
        raise ValueError("missing argument to 'peersof'")

    servers = args[0]
    addrs = servers.split(",")

    rf = int(args[2]) if len(args) >= 3 else len(addrs)
    pid = int(partid(args[1:]))
    rid = int(repid(args[1:]))

    partaddrs = addrs[pid * rf : (pid + 1) * rf]
    if rid >= len(partaddrs):
        return ""  # ignore out-of-bounds here

    peeraddrs = ",".join(
        partaddrs[:rid] + partaddrs[rid + 1 :],  # exclude self
    )
    if len(peeraddrs) == 0:
        return "none"  # specially defined
    else:
        return peeraddrs


def main():
    if len(sys.argv) <= 1:
        raise ValueError("missing argument to 'justmod/xtract.py'")

    if sys.argv[1] == "partid":
        print(partid(sys.argv[2:]))
    elif sys.argv[1] == "repid":
        print(repid(sys.argv[2:]))
    elif sys.argv[1] == "portof":
        print(portof(sys.argv[2:]))
    elif sys.argv[1] == "peersof":
        print(peersof(sys.argv[2:]))


if __name__ == "__main__":
    main()

import sys


def Coq_BB5_to_bbchallenge(s: str) -> str:
    """
    >>> Coq_BB5_to_bbchallenge("BR0 HR1 AL1 CL0 DR1 CL1 CL1 ER0 BL1 ER1")
    '0RB---_1LA0LC_1RD1LC_1LC0RE_1LB1RE'
    >>> Coq_BB5_to_bbchallenge("BR0 HR1 CL0 BR0 DR1 EL0 ER1 AR0 CL1 BL1")
    '0RB---_0LC0RB_1RD0LE_1RE0RA_1LC1LB'
    """
    s = s.replace(" ", "")
    s_split = [s[i : i + 6] for i in range(0, len(s), 6)]
    new_split = []

    def translate(transition_s: str) -> str:
        if "H" in transition_s:
            return "---"
        return transition_s[-1] + transition_s[1] + transition_s[0]

    for line in s_split:
        new_split.append(translate(line[:3]) + translate(line[3:6]))
    return "_".join(new_split)


if __name__ == "__main__":
    Coq_file = sys.argv[1]
    with open(Coq_file) as f:
        file_content = f.read()

    for line in file_content.split("\n"):
        if not "makeTM" in line:
            continue
        TM_part, param_part = line.split(",")
        TM_str = TM_part.replace("makeTM", "").replace("(", "")
        param_str = param_part.replace("RWL", "").replace(")::", "").strip()
        print(Coq_BB5_to_bbchallenge(TM_str), param_str)

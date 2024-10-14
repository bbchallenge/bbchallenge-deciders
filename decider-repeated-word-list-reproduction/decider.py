from tm_tape import TMTape, TMHasHalted
from tm_regex_tape import TMRegexTape, BlockSimulationTimeout, FacingBlock

TM = "1RB1LC_1LA1RD_1LD1LA_1RA1RE_---1RB"
BLOCK_SIZE = 2
PLUS_THRESHOLD = 6
BLOCK_SIMULATION_TIMEOUT = 10000
VERY_VERBOSE = False
print(TM)
TM_tape = TMTape(TM, "", 0, "")

visited_regex_tapes: set[str] = set()
regex_tapes_to_visit: list[TMRegexTape] = [
    TMRegexTape.from_tm_tape(TM_tape, BLOCK_SIZE, PLUS_THRESHOLD)
]

while len(regex_tapes_to_visit) > 0:
    curr_regex_tape = regex_tapes_to_visit.pop()

    if str(curr_regex_tape) in visited_regex_tapes:
        continue

    visited_regex_tapes.add(str(curr_regex_tape))

    print(curr_regex_tape)

    try:
        curr_regex_tape.macro_step(BLOCK_SIMULATION_TIMEOUT, VERY_VERBOSE)
        regex_tapes_to_visit.append(curr_regex_tape)
    except TMHasHalted:
        print("Decider not successful (halting configuration reached)")
        exit(-1)
    except BlockSimulationTimeout:
        print(
            "Decider not successful (block simulation timeout {BLOCK_SIMULATION_TIMEOUT} reached)"
        )
    except FacingBlock:
        regex_tapes_to_visit += curr_regex_tape.get_plus_branches(VERY_VERBOSE)

print(f"Decider successful: TM does not halt")

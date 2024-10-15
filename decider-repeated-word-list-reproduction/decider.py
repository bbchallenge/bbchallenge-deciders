import argparse

from tm_tape import TMTape, TMHasHalted
from tm_regex_tape import TMRegexTape, BlockSimulationTimeout, FacingBlock

argparser = argparse.ArgumentParser(description="Repeated Word List decider (RepWL)")
argparser.add_argument(
    "-m",
    "--tm",
    type=str,
    help="The transition function of the Turing machine in the bbchallenge format, e.g. 0RB---_1LC1RC_1LD0RA_1RE0LD_1RA1RE",
)
argparser.add_argument(
    "-b", "--block-size", type=int, help="The block size to use for the decider"
)

argparser.add_argument(
    "-r",
    "--plus-repeat-threshold",
    type=int,
    help="The threshold for the plus operator",
)


argparser.add_argument(
    "-f",
    "--file-machines-list",
    type=str,
    help="The file containing the list of Turing machines with parameters",
)

argparser.add_argument(
    "-t",
    "--block-simulation-timeout",
    type=int,
    help="The block simulation timeout",
    default=1000,
)

argparser.add_argument(
    "-M",
    "--decider-max-visited-regex",
    type=int,
    help="The maximum number of visited regex tapes",
    default=150000,
)

argparser.add_argument(
    "--verbose",
    action=argparse.BooleanOptionalAction,
    default=False,
    help="Prints debug information",
)

argparser.add_argument(
    "--print-cert",
    action=argparse.BooleanOptionalAction,
    default=True,
    help="Prints the RepWL non-halt certificate(s)",
)

args = argparser.parse_args()

print(args)

FILE_MACHINES_LIST = None
if args.file_machines_list is not None:
    FILE_MACHINES_LIST = args["file-machines-list"]
else:
    if args.tm is None or args.block_size is None or args.plus_repeat_threshold is None:
        print(
            "Error: either provide a file with Turing machines and parameters or a single Turing machine with paramters"
        )
        exit(-1)
    TM = args.tm
    BLOCK_SIZE = args.block_size
    PLUS_THRESHOLD = args.plus_repeat_threshold

BLOCK_SIMULATION_TIMEOUT = args.block_simulation_timeout
DECIDER_MAX_VISITED_REGEX = args.decider_max_visited_regex
PRINT_CERT = args.print_cert
VERBOSE = args.verbose


if PRINT_CERT:
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

    if len(visited_regex_tapes) > DECIDER_MAX_VISITED_REGEX:
        print(
            f"Decider not successful (limit of `{DECIDER_MAX_VISITED_REGEX}` visited regex tapes reached)"
        )
        exit(-1)

    if PRINT_CERT:
        print(curr_regex_tape)

    try:
        curr_regex_tape.macro_step(BLOCK_SIMULATION_TIMEOUT, VERBOSE)
        regex_tapes_to_visit.append(curr_regex_tape)
    except TMHasHalted:
        print("Decider not successful (halting configuration reached)")
        exit(-1)
    except BlockSimulationTimeout:
        print(
            "Decider not successful (block simulation timeout {BLOCK_SIMULATION_TIMEOUT} reached)"
        )
    except FacingBlock:
        regex_tapes_to_visit += curr_regex_tape.get_plus_branches(VERBOSE)

print(f"Decider successful: TM does not halt")

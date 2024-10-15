import argparse

from tm_tape import TMTape, TMHasHalted
from tm_regex_tape import TMRegexTape, BlockSimulationTimeout, FacingBlock


class NoException(Exception):
    pass


class MaxVisitedRegex(Exception):
    pass


def deciderRep_WL(
    TM: str,
    block_size: int,
    plus_threshold: int,
    max_visited_regex: int,
    block_simulation_timeout: int,
    print_cert: bool,
    verbose: bool,
) -> tuple[bool, Exception]:
    if print_cert:
        print(TM)

    TM_tape = TMTape(TM, "", 0, "")

    visited_regex_tapes: set[str] = set()
    regex_tapes_to_visit: list[TMRegexTape] = [
        TMRegexTape.from_tm_tape(TM_tape, block_size, plus_threshold)
    ]

    while len(regex_tapes_to_visit) > 0:
        curr_regex_tape = regex_tapes_to_visit.pop()

        if str(curr_regex_tape) in visited_regex_tapes:
            continue

        visited_regex_tapes.add(str(curr_regex_tape))

        if len(visited_regex_tapes) > max_visited_regex:
            return MaxVisitedRegex, False

        if print_cert:
            print(curr_regex_tape)

        try:
            curr_regex_tape.macro_step(block_simulation_timeout, verbose)
            regex_tapes_to_visit.append(curr_regex_tape)
        except TMHasHalted:
            return False, TMHasHalted
        except BlockSimulationTimeout:
            return False, BlockSimulationTimeout
        except FacingBlock:
            regex_tapes_to_visit += curr_regex_tape.get_plus_branches(verbose)
    return True, NoException


def failure_reason_str(
    reason_failure: Exception, block_simulation_timeout: int, max_visited_regex: int
) -> str:
    if reason_failure == TMHasHalted:
        return "halting configuration reached"
    if reason_failure == BlockSimulationTimeout:
        return f"block simulation timeout `{block_simulation_timeout}` reached"
    if reason_failure == MaxVisitedRegex:
        return f"limit of `{max_visited_regex}` visited regex tapes reached"
    return "Unknown"


if __name__ == "__main__":

    argparser = argparse.ArgumentParser(
        description="Repeated Word List decider (RepWL)"
    )
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
        "--max-visited-regex",
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

    argparser.add_argument(
        "--print-params-stats",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="In case of a file with Turing machines and parameters, print statistics about the parameters (min, max, avg)",
    )

    args = argparser.parse_args()

    FILE_MACHINES_LIST = None
    if args.file_machines_list is not None:
        FILE_MACHINES_LIST = args.file_machines_list
    else:
        if (
            args.tm is None
            or args.block_size is None
            or args.plus_repeat_threshold is None
        ):
            print(
                "Error: either provide a file with Turing machines and parameters or a single Turing machine with paramters"
            )
            exit(-1)
        TM = args.tm
        BLOCK_SIZE = args.block_size
        PLUS_THRESHOLD = args.plus_repeat_threshold

    BLOCK_SIMULATION_TIMEOUT = args.block_simulation_timeout
    MAX_VISITED_REGEX = args.max_visited_regex
    PRINT_CERT = args.print_cert
    VERBOSE = args.verbose

    if FILE_MACHINES_LIST is None:
        success, reason_failure = deciderRep_WL(
            TM,
            BLOCK_SIZE,
            PLUS_THRESHOLD,
            MAX_VISITED_REGEX,
            BLOCK_SIMULATION_TIMEOUT,
            PRINT_CERT,
            VERBOSE,
        )

        if success:
            print(f"Decider successful: TM does not halt")
            exit(0)

        if reason_failure == TMHasHalted:
            print("Decider not successful (halting configuration reached)")
            exit(-1)
        if reason_failure == BlockSimulationTimeout:
            print(
                f"Decider not successful ({failure_reason_str(reason_failure, BLOCK_SIMULATION_TIMEOUT, MAX_VISITED_REGEX)})"
            )
            exit(-1)
        if reason_failure == MaxVisitedRegex:
            print(
                f"Decider not successful ({failure_reason_str(reason_failure, BLOCK_SIMULATION_TIMEOUT, MAX_VISITED_REGEX)})"
            )
            exit(-1)
    else:
        import tqdm

        with open(FILE_MACHINES_LIST) as f:
            file_content = f.read()

        at_least_one_failure = False

        params_stats_B = []
        params_stats_R = []

        num_TMs = 0
        for line in tqdm.tqdm(file_content.split("\n")):
            if line.strip() == "":
                continue
            TM, BLOCK_SIZE, PLUS_THRESHOLD = line.split(" ")

            params_stats_B.append(int(BLOCK_SIZE))
            params_stats_R.append(int(PLUS_THRESHOLD))

            success, reason_failure = deciderRep_WL(
                TM,
                int(BLOCK_SIZE),
                int(PLUS_THRESHOLD),
                MAX_VISITED_REGEX,
                BLOCK_SIMULATION_TIMEOUT,
                PRINT_CERT,
                VERBOSE,
            )

            num_TMs += 1

            if not success:
                at_least_one_failure = True
                print(
                    f"Failed to decide `{TM}` with parameters `block_size={BLOCK_SIZE}` and `plus_repeat_threshold={PLUS_THRESHOLD}`. Reason: {failure_reason_str(reason_failure, BLOCK_SIMULATION_TIMEOUT, MAX_VISITED_REGEX)}."
                )

        if args.print_params_stats:
            print(
                f"Statistics:\n\t-block_size: min={min(params_stats_B)}, max={max(params_stats_B)}, avg={round(sum(params_stats_B)/len(params_stats_B),1)}"
            )
            print(
                f"\t-plus_repeat_threshold: min={min(params_stats_R)}, max={max(params_stats_R)}, avg={round(sum(params_stats_R)/len(params_stats_R),1)}"
            )

        if at_least_one_failure:
            exit(-1)
        print(f"All {num_TMs} TMs have been decided successfully")
        exit(0)

from copy import deepcopy
import argparse


def i2l(i: int) -> str:
    return chr(i + ord("A"))


def l2i(l: str) -> int:
    return ord(l) - ord("A")


ZERO = "0"
ONE = "1"
UNKNOWN = "?"

RIGHT = "R"
LEFT = "L"


class TMHasHalted(Exception):
    pass


class TMCenteredTapeSegment(object):
    """Represents a TM tape limited to a finite segment always centered at the head.

    Example:
    >>> TM = "1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA"
    >>> tm = TMCenteredTapeSegment(TM, 3)
    >>> print(tm)
    000 [A0] 000
    >>> tm.step()
    'R'
    >>> print(tm)
    001 [B0] 00?
    >>> tm_zero, tm_one = tm.split()
    >>> print(tm_zero)
    001 [B0] 000
    >>> print(tm_one)
    001 [B0] 001
    >>> tm_zero.step()
    'L'
    >>> print(tm_zero)
    ?00 [C1] 000
    >>> tm_zero_2, tm_one_2 = tm_zero.split()
    >>> print(tm_zero_2)
    000 [C1] 000
    >>> print(tm_one_2)
    100 [C1] 000
    """

    def __init__(self, tm, radius):
        self.tm = tm.split("_")
        self.radius = radius
        self.state = 0

        self.before_head = [ZERO] * radius
        self.after_head = [ZERO] * radius
        self.at_head = ZERO

    def ngrams(self, which=None):
        if which == RIGHT:
            return "".join(self.after_head)
        if which == LEFT:
            return "".join(self.before_head)

        return "".join(self.before_head), "".join(self.after_head)

    def step(self):
        current_symbol = self.at_head

        if current_symbol == UNKNOWN:
            raise ValueError("Unknown read symbol")

        new_symbol, move, new_state = self.tm[self.state][
            3 * int(current_symbol) : 3 * int(current_symbol) + 3
        ]

        if new_symbol == "-":
            raise TMHasHalted()

        self.at_head = new_symbol
        self.state = l2i(new_state)
        if move == "R":
            self.before_head.append(self.at_head)
            self.before_head.pop(0)
            self.at_head = self.after_head.pop(0)
            self.after_head.append(UNKNOWN)
        elif move == "L":
            self.after_head.insert(0, self.at_head)
            self.after_head.pop()
            self.at_head = self.before_head.pop()
            self.before_head.insert(0, UNKNOWN)

        return move

    def split(self) -> tuple["TMCenteredTapeSegment", "TMCenteredTapeSegment"]:
        if str(self).count(UNKNOWN) != 1 or not (
            self.before_head[0] == UNKNOWN or self.after_head[-1] == UNKNOWN
        ):
            raise ValueError(
                f"Splitting {str(self)} is not possible: exactly one `?` on either extremity of the tape is required"
            )

        tm_zero, tm_one = deepcopy(self), deepcopy(self)
        if self.before_head[0] == UNKNOWN:
            tm_zero.before_head[0] = ZERO
            tm_one.before_head[0] = ONE
            return tm_zero, tm_one

        tm_zero.after_head[-1] = ZERO
        tm_one.after_head[-1] = ONE
        return tm_zero, tm_one

    def __str__(self):
        lngram, rngram = self.ngrams()
        head_str = f"[{i2l(self.state)}{self.at_head}]"
        return f"{lngram} {head_str} {rngram}"


def ngram_CPS_decider(
    tm, radius, max_context_count=1_000_000, verbose=False, print_cert=False
):
    local_context = TMCenteredTapeSegment(tm, radius)

    to_visit = [local_context]

    to_potentially_visit_for_ngram = {LEFT: {}, RIGHT: {}}

    reachable_local_contexts = set()

    reachable_ngrams = {}
    reachable_ngrams[RIGHT] = set()
    reachable_ngrams[LEFT] = set()

    while len(to_visit) > 0:
        current_local_context = to_visit.pop()

        if str(current_local_context) in reachable_local_contexts:
            continue

        reachable_local_contexts.add(str(current_local_context))

        if len(reachable_local_contexts) > max_context_count:
            raise ValueError(
                f"Exceeded maximum number of visited local contexts ({max_context_count}), please increase --max-context-count"
            )

        left_ngram, right_ngram = current_local_context.ngrams()
        reachable_ngrams[LEFT].add(left_ngram)
        reachable_ngrams[RIGHT].add(right_ngram)

        if verbose:
            print(current_local_context)
            print("\tLeft ngram:", left_ngram)
            print("\tRight ngram:", right_ngram)

        for DIR, ngram in [(LEFT, left_ngram), (RIGHT, right_ngram)]:
            if ngram in to_potentially_visit_for_ngram[DIR]:
                if verbose:
                    print(f"\tNewly reached {DIR}-ngram:", ngram, "!")
                for tm in to_potentially_visit_for_ngram[DIR][ngram]:
                    to_visit.append(tm)
                    if verbose:
                        print("\tNeed to visit:", tm)
                del to_potentially_visit_for_ngram[DIR][ngram]

        try:
            move = current_local_context.step()
        except TMHasHalted:
            if verbose:
                print("\tTM halts!")
            return False

        if verbose:
            print("\tAfter step:", current_local_context)

        tm_split = current_local_context.split()
        if verbose:
            print()
        for tm in tm_split:
            if verbose:
                print("Considering child:", tm)
            # If the ngram is reachable we add the new local context to the queue
            if tm.ngrams(move) in reachable_ngrams[move]:
                if verbose:
                    print(f"\tReachable ngram {move}-ngram:", tm.ngrams(move))
                    print("\tAdding to queue!")
                to_visit.append(tm)
            # Otherwise we remember the context for this ngram and will revisit if
            # we ever reach the ngram
            else:
                if verbose:
                    print(f"\tUnreachable {move}-ngram:", tm.ngrams(move))
                    print(f"\tAdding to the ngram's waitlist")
                ngram = tm.ngrams(move)
                if ngram not in to_potentially_visit_for_ngram[move]:
                    to_potentially_visit_for_ngram[move][ngram] = []
                to_potentially_visit_for_ngram[move][ngram].append(tm)
            if verbose:
                print()

    if print_cert:
        for DIR in [LEFT, RIGHT]:
            print(f"Reachable {DIR}-ngrams:", reachable_ngrams[DIR])

        print(
            f"Reachable local contexts ({len(reachable_local_contexts)}):",
            reachable_local_contexts,
        )

    return True


if __name__ == "__main__":

    argparser = argparse.ArgumentParser(
        description="n-gram Closed Position Set (NGramCPS)"
    )
    argparser.add_argument(
        "-m",
        "--tm",
        type=str,
        help="The transition function of the Turing machine in the bbchallenge format, e.g. 1RB---_0LC0RB_1RD1LD_0LE0RA_0RC0RA",
    )
    argparser.add_argument(
        "-r",
        "--radius",
        type=int,
        help="Size of the ngrams on both sides of the head, e.g. 2",
    )

    argparser.add_argument(
        "-M",
        "--max-context-count",
        type=int,
        help="The maximum number of visited local contexts",
        default=1_000_000,
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
        default=False,
        help="Prints the reached ngrams and local contexts",
    )

    argparser.add_argument(
        "--print-final-result",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Prints whether the machine does not halt or may halt",
    )

    args = argparser.parse_args()

    result = ngram_CPS_decider(
        args.tm, args.radius, args.max_context_count, args.verbose, args.print_cert
    )

    if result:
        if args.print_final_result:
            print(f"Success: {args.tm} does not halt")
        exit(0)

    if args.print_final_result:
        print(f"Failure: {args.tm} may halt")

    exit(-1)

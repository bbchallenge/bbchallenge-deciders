from typing import TypeVar, Generic, Callable, Literal


def i2l(i: int) -> str:
    return chr(i + ord("A"))


def l2i(l: str) -> int:
    return ord(l) - ord("A")


St = int
Σ = TypeVar("Σ")


class Transition(Generic[Σ]):
    def __init__(self, write: Σ, move: Literal["L", "R"], new_state: St):
        self.write: Σ = write
        self.move: Literal["L", "R"] = move
        self.new_state: St = new_state

    def __iter__(self):
        return iter((self.write, self.move, self.new_state))

    def __str__(self):
        return f"{self.write}{self.move}{i2l(self.new_state)}"

    def __repr__(self):
        return self.__str__()


TM = Callable[[St, Σ], Transition | None]
NGRAM = tuple[Σ]


def ngram_to_str(ngram: NGRAM) -> str:
    return "".join(map(lambda x: str(x), ngram))


class LocalContext(Generic[Σ]):
    def __init__(self, left: NGRAM[Σ], right: NGRAM[Σ], center: Σ, state: St):
        self.left: NGRAM[Σ] = left
        self.right: NGRAM[Σ] = right
        self.center: Σ = center
        self.state: int = state

    def __str__(self):
        return f"{self.left} [{i2l(self.state)} {self.center}] {self.right}"

    def __repr__(self):
        return str(self)

    def __hash__(self) -> int:
        return hash(self.__str__())

    def __lt__(self, other) -> bool:
        return str(self) < str(other)

    def __eq__(self, other) -> int:
        return (
            self.left == other.left
            and self.right == other.right
            and self.center == other.center
            and self.state == other.state
        )


# the list is not really necessary but it is to match
# the order of visit of Coq-BB5
A = TypeVar("A")


class ListSet(Generic[A]):
    def __init__(self):
        self.list: list[A] = []
        self.set: set[A] = set({})

    def is_in(self, lc: A) -> bool:
        return lc in self.set

    def ins(self, lc: A):
        if not self.is_in(lc):
            self.list = [lc] + self.list
            self.set.add(lc)


# set of ngrams {'000','001'} is represented as
# {NGRAM['00']: {'0', '1'}}
ngram_set = dict[NGRAM[Σ], ListSet[Σ]]


def ngram_set_to_str(ngram_s: ngram_set) -> str:
    to_ret = []
    for ngram in sorted(ngram_s.keys()):
        for v in ngram_s[ngram].list:
            to_ret.append(ngram_to_str(ngram) + str(v))
    return "{" + ", ".join(to_ret) + "}"


class AbstractExecState(Generic[Σ]):
    def __init__(
        self,
        left_ngrams: ngram_set,
        right_ngrams: ngram_set,
        local_contexts: ListSet[LocalContext[Σ]],
    ):
        self.left_ngrams: ngram_set = left_ngrams
        self.right_ngrams: ngram_set = right_ngrams

        self.local_contexts: ListSet[LocalContext[Σ]] = local_contexts

    def ins_left_ngram(self, ngram: NGRAM[Σ]):
        suffix = ngram[1:]
        if suffix in self.left_ngrams:
            self.left_ngrams[suffix].ins(ngram[0])
        else:
            new_list_set = ListSet[Σ]()
            new_list_set.ins(ngram[0])
            self.left_ngrams[suffix] = new_list_set

    def ins_right_ngram(self, ngram: NGRAM[Σ]):
        prefix = ngram[:-1]
        if prefix in self.right_ngrams:
            self.right_ngrams[prefix].ins(ngram[-1])
        else:
            new_list_set = ListSet[Σ]()
            new_list_set.ins(ngram[-1])
            self.left_ngrams[prefix] = new_list_set

    @classmethod
    def initial(cls, Σ0: Σ, len_l: int, len_r) -> "AbstractExecState":
        initial_local_context = ListSet()
        initial_local_context.ins(
            LocalContext(NGRAM([Σ0] * len_l), NGRAM([Σ0] * len_r), Σ0, 0)
        )

        Σ0_list_set1, Σ0_list_set2 = ListSet[Σ](), ListSet[Σ]()

        Σ0_list_set1.ins(Σ0)
        Σ0_list_set2.ins(Σ0)

        return cls[Σ](
            {NGRAM([Σ0] * (len_l - 1)): Σ0_list_set1},
            {NGRAM([Σ0] * (len_r - 1)): Σ0_list_set2},
            initial_local_context,
        )


def expand_local_context(
    tm: TM,
    lc: LocalContext[Σ],
    S: AbstractExecState[Σ],
) -> tuple[list[LocalContext[Σ]], bool]:
    state = lc.state
    symbol = lc.center

    transition = tm(state, symbol)

    if transition is None:
        return [], True

    write, move, new_state = transition

    to_return: list[LocalContext[Σ]] = []

    if move == "R":
        falling_left_ngram = lc.left
        S.ins_left_ngram(falling_left_ngram)
        possible_new_right_symbols: list[Σ] = S.right_ngrams.get(
            lc.right[1:], ListSet()
        ).list
        for s in possible_new_right_symbols:
            new_lc = LocalContext(
                lc.left[1:] + (write,),
                lc.right[1:] + (s,),
                lc.right[0],
                new_state,
            )

            if not S.local_contexts.is_in(new_lc):
                to_return.append(new_lc)
                S.local_contexts.ins(new_lc)

    elif move == "L":
        falling_right_ngram = lc.right
        S.ins_right_ngram(falling_right_ngram)
        possible_new_left_symbols: list[Σ] = S.left_ngrams.get(
            lc.left[:-1], ListSet()
        ).list
        for s in possible_new_left_symbols:
            new_lc = LocalContext(
                (s,) + lc.left[:-1],
                (write,) + lc.right[:-1],
                lc.left[-1],
                new_state,
            )
            if not S.local_contexts.is_in(new_lc):
                to_return.append(new_lc)
                S.local_contexts.ins(new_lc)

    return to_return, False


def NGramCPS_decider(
    tm: TM,
    Σ0: Σ,
    len_l: int,
    len_r: int,
    gas: int,
    verbose: bool = False,
    print_cert: bool = False,
) -> bool:
    S: AbstractExecState[Σ] = AbstractExecState.initial(Σ0, len_l, len_r)

    gas2 = gas
    while gas > 0:
        to_visit: list[LocalContext[Σ]] = S.local_contexts.list[:]
        any_updates = False

        while len(to_visit) > 0 and gas2 > 0:
            curr_local_context = to_visit[0]

            if verbose:
                print("Visiting", str(curr_local_context))

            to_visit = to_visit[1:]
            new_contexts_to_visit, halting_met = expand_local_context(
                tm, curr_local_context, S
            )

            if halting_met:
                return False

            if len(new_contexts_to_visit) > 0:
                any_updates = True

            to_visit = new_contexts_to_visit + to_visit
            gas2 -= 1
        gas -= 1
        if not any_updates:
            if print_cert:
                print("Reachable left-ngrams:", ngram_set_to_str(S.left_ngrams))
                print("Reachable right-ngrams:", ngram_set_to_str(S.right_ngrams))

                print(
                    f"Reachable local contexts ({len(S.local_contexts.list)}):",
                    S.local_contexts.list,
                )

            return True

    return False

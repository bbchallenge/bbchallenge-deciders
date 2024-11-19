from copy import deepcopy
from typing import Literal, TypeVar, Generic

Sigma = str
St = Literal["St0", "St1", "St2", "St3", "St4"]


class MidWord(object):
    def __init__(self, l: list[str], r: list[str], m: str, s: St):
        self.left = l
        self.right = r
        self.mid = m
        self.state = s

    def __str__(self):
        return f"MidWord('{"".join(self.left)}', '{"".join(self.right)}', {self.mid}, {self.state})"

    def __repr__(self):
        return self.__str__()

    def __eq__(self, other):
        return (
            self.left == other.left
            and self.right == other.right
            and self.mid == other.mid
            and self.state == other.state
        )

    def __lt__(self, other):
        return str(self) < str(other)

    def __hash__(self):
        return hash(str(self))


T = TypeVar("T")


def sort_set_str(set: set) -> str:
    return str(sorted(set)).replace("[", "{").replace("]", "}")


class SetOfEncoding(Generic[T]):
    """
    https://github.com/ccz181078/Coq-BB5/blob/384b4eff476867c196eb00d3e6ecedd6d85da811/CoqBB5/Prelims.v#L42

    Example:

    >>> s = SetOfEncoding[Sigma]()
    >>> s.set_ins('0')
    ((['0'], {'0'}), False)
    >>> s.set_ins('0')[0].set_ins('1')
    ((['1', '0'], {'0', '1'}), False)
    >>> s.set_ins('0')[0].set_ins('0')
    ((['0'], {'0'}), True)
    >>> s
    ([], {})

    >>> s = SetOfEncoding[MidWord]()
    >>> s.set_ins(MidWord([], [], '0', 'St0'))
    (([MidWord('', '', 0, St0)], {MidWord('', '', 0, St0)}), False)
    >>> s.set_ins(MidWord(["0","1"], ["1"], '0', 'St1'))
    (([MidWord('01', '1', 0, St1)], {MidWord('01', '1', 0, St1)}), False)
    >>> s.set_ins(MidWord(["0","1"], ["1"], '0', 'St1'))[0].set_ins(MidWord(["0","1"], ["1"], '0', 'St1'))
    (([MidWord('01', '1', 0, St1)], {MidWord('01', '1', 0, St1)}), True)
    >>> s.set_ins(MidWord(["0","1"], ["1"], '0', 'St1'))[0].set_ins(MidWord(["0","1"], ["11"], '0', 'St2'))
    (([MidWord('01', '11', 0, St2), MidWord('01', '1', 0, St1)], {MidWord('01', '1', 0, St1), MidWord('01', '11', 0, St2)}), False)

    """

    def __init__(self, fst: list[T] = [], snd: set[T] = set()):
        self.fst = fst
        self.snd = snd

    def set_ins(self, x: T) -> tuple["SetOfEncoding[T]", bool]:
        if x in self.snd:
            return self, True
        fst_copy = deepcopy(self.fst)
        snd_copy = deepcopy(self.snd)
        fst_copy.insert(0, x)
        snd_copy.add(x)
        return SetOfEncoding[T](fst_copy, snd_copy), False

    def __str__(self):
        return f"({self.fst}, {sort_set_str(self.snd)})"

    def __repr__(self):
        return f"({self.fst}, {sort_set_str(self.snd)})"


NGRAM = str
lsigma_to_str = lambda x: "".join(x)

xset_impl = dict[NGRAM, SetOfEncoding[Sigma]]
mset_impl = SetOfEncoding[MidWord]

# (* Compute pop_back' 0 [ 1 ; 2 ; 3 ; 4 ].
#    = ([0; 1; 2; 3], 4)
#    Compute pop_back' 0 nil.
#    = (nil, 0)
# *)
# Fixpoint pop_back'{T}(x:T)(ls:list T):(list T)*T :=
#   match ls with
#   | nil => (nil,x)
#   | h :: t => let (a,b):=pop_back' h t in (x::a,b)
#   end.


def pop_back_prime(h: T, t: list[T]) -> tuple[T, list[T]]:
    """
    Example:
        >>> pop_back_prime(0, [])
        ([], 0)
        >>> pop_back_prime(0, [1, 2, 3, 4])
        ([0, 1, 2, 3], 4)
    """
    if len(t) == 0:
        return t, h

    return [h] + t[:-1], t[-1]


def xset_ins0(
    xs: xset_impl, v: SetOfEncoding[Sigma], x1: list[Sigma], x2: Sigma
) -> tuple[xset_impl, bool]:
    new_v, flag = v.set_ins(x2)
    xs_copy = xs.copy()
    xs_copy[lsigma_to_str(x1)] = new_v
    return (xs_copy, flag)


def xset_ins(xs: xset_impl, x: list[Sigma]) -> tuple[xset_impl, bool]:
    if len(x) == 0:
        return xs, False

    h = x[0]
    t = x[1:]

    x1, x2 = pop_back_prime(h, t)

    if lsigma_to_str(x) in xs:
        v = xs[lsigma_to_str(x)]
        xset_ins0(xs, v, x1, x2)
    else:
        xset_ins0(xs, SetOfEncoding[Sigma](), x1, x2)


class AES_impl(object):
    def __init__(
        self,
        lset: xset_impl,
        rset: xset_impl,
        mset: mset_impl,
    ):
        self.lset = lset
        self.rset = rset
        self.mset = mset

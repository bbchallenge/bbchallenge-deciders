import copy
from tm_tape import TMTape, i2l, l2i, TMHasHalted, TMHeadAtTapeExtremity


class FacingBlock(Exception):
    pass


class BlockSimulationTimeout(Exception):
    pass


class RegexBlock(object):
    """Represents a regex of the form `B^k` or `B^k+ = B^kB^*`, where B is a fixed binary string."""

    def __init__(self, B: str, repeat: int, plus: bool):
        self.B = B
        self.repeat = repeat

        if repeat <= 0:
            raise ValueError("Repeat must be strictly greater than 0")

        self.plus = plus

    def __str__(self):
        return f"({self.B})^{self.repeat}{'+' if self.plus else ''}"

    def to_tm_tape(
        self, TM_bbchallenge_format, state, head_looking_after=True
    ) -> TMTape:
        if self.plus:
            raise FacingBlock()
        if head_looking_after:
            return TMTape(
                TM_bbchallenge_format,
                "",
                state,
                self.B * self.repeat,
                head_looking_after=head_looking_after,
                tape_extremity_infinite_zero=False,
            )
        return TMTape(
            TM_bbchallenge_format,
            self.B * self.repeat,
            state,
            "",
            head_looking_after=head_looking_after,
            tape_extremity_infinite_zero=False,
        )


class TMRegexTape(object):
    def __init__(
        self,
        TM_bbchallenge_format: str,
        before_head: list[RegexBlock] = [],
        state: int = 0,
        after_head: list[RegexBlock] = [],
        head_looking_after: bool = True,
        block_size: int | None = None,
        plus_threshold: int | None = None,
    ):
        """
        >>> TM = "1RB1LA_1LC1RE_0LE0LD_1LC0LD_0RA---"
        >>> TM_tape = TMTape(TM, "101010111", 0, "1010111111")
        >>> str(TM_tape)
        '0∞101010111A>10101111110∞'
        >>> TM_regex_tape = TMRegexTape.from_tm_tape(TM_tape, 2, 3)
        >>> str(TM_regex_tape)
        '0∞ (01)^3+ (11)^1 A> (10)^2 (11)^3+ 0∞'
        """
        self.TM_bbchallenge_format = TM_bbchallenge_format
        self.state = state
        self.head_looking_after = head_looking_after

        self.before_head = before_head[:]
        self.after_head = after_head[::-1]

        self.block_size = block_size
        self.plus_threshold = plus_threshold

    @classmethod
    def from_tm_tape(cls, tm_tape: TMTape, block_size: int, plus_threshold: int):
        blocks_before_head = []
        blocks_after_head = []

        padded_symbols_before_head = "".join(
            ["0"] * (len(tm_tape.before_head) % block_size) + tm_tape.before_head
        )

        for i in range(0, len(padded_symbols_before_head), block_size):
            block = padded_symbols_before_head[i : i + block_size]

            blocks_before_head.append(RegexBlock(block, 1, False))

        padded_symbols_after_head = "".join(
            ["0"] * (len(tm_tape.after_head) % block_size) + tm_tape.after_head[::-1]
        )
        for i in range(0, len(padded_symbols_after_head), block_size):
            block = padded_symbols_after_head[i : i + block_size]
            blocks_after_head.append(RegexBlock(block, 1, False))

        regexTape = cls(
            tm_tape.TM_bbchallenge_format,
            blocks_before_head,
            tm_tape.state,
            blocks_after_head,
            tm_tape.head_looking_after,
            block_size,
            plus_threshold,
        )
        regexTape.compress_and_generalise(plus_threshold)
        return regexTape

    def __str__(self):
        head_str = ""
        if self.head_looking_after:
            head_str = f"{i2l(self.state)}>"
        else:
            head_str = f"<{i2l(self.state)}"

        to_return = "0∞ "
        for block in self.before_head:
            to_return += str(block) + " "
        to_return += head_str
        for block in self.after_head[::-1]:
            to_return += " " + str(block)

        return to_return + " 0∞"

    def compress(self):
        """
        From https://github.com/savask/turing/blob/main/Repeat.hs

        Glue powers of the same word together, e.g. (110)^3 (110)^4 -> (110)^7,
        and update "at least" powers, e.g. (110)^3 (110)^4+ -> (110)^7+."""

        new_blocks_before_head: list[RegexBlock] = []
        for block in self.before_head:
            if (
                len(new_blocks_before_head) > 0
                and new_blocks_before_head[-1].B == block.B
            ):
                new_blocks_before_head[-1].repeat += block.repeat
                new_blocks_before_head[-1].plus = (
                    new_blocks_before_head[-1].plus or block.plus
                )
            else:
                new_blocks_before_head.append(block)

        new_blocks_after_head: list[RegexBlock] = []
        for block in self.after_head:
            if (
                len(new_blocks_after_head) > 0
                and new_blocks_after_head[-1].B == block.B
            ):
                new_blocks_after_head[-1].repeat += block.repeat
                new_blocks_after_head[-1].plus = (
                    new_blocks_after_head[-1].plus or block.plus
                )
            else:
                new_blocks_after_head.append(block)

        self.before_head = new_blocks_before_head[:]  # not sure copy is necessary
        self.after_head = new_blocks_after_head[:]  # not sure copy is necessary

        return

    def generalise(self, plus_threshold: int):
        """
        From https://github.com/savask/turing/blob/main/Repeat.hs

        Replace powers by "at least" versions if there are at least 'plus_threshold' repetitions.
        """
        for block in self.before_head:
            if block.repeat >= plus_threshold:
                block.plus = True
                block.repeat = plus_threshold

        for block in self.after_head:
            if block.repeat >= plus_threshold:
                block.plus = True
                block.repeat = plus_threshold
        return

    def compress_and_generalise(self, plus_threshold: int):
        self.compress()
        self.generalise(plus_threshold)

    def read_block(self) -> RegexBlock:
        if self.head_looking_after and len(self.after_head) > 0:
            return self.after_head[-1]
        if not self.head_looking_after and len(self.before_head) > 0:
            return self.before_head[-1]

        if self.block_size is not None:
            return RegexBlock("0" * self.block_size, 1, False)

        return RegexBlock("0", 1, False)

    def write_blocks(self, blocks: list[RegexBlock]):
        assert len(blocks) > 0
        if self.head_looking_after:
            if len(self.after_head) == 0:
                self.after_head += blocks[::-1]
            else:
                self.after_head[-1] = blocks[-1]
                self.after_head += blocks[::-1][1:]
        else:
            if len(self.before_head) == 0:
                self.before_head += blocks
            else:
                self.before_head[-1] = blocks[0]
                self.before_head += blocks[1:]

    def move_head(self, move_right: bool, n_blocks=1):
        if not move_right:
            if self.head_looking_after:
                self.head_looking_after = False
            else:
                for _ in range(n_blocks):
                    self.after_head.append(self.before_head.pop())
        else:
            if not self.head_looking_after:
                self.head_looking_after = True
            else:
                for _ in range(n_blocks):
                    self.before_head.append(self.after_head.pop())

    def macro_steps(self, n_steps: int, block_simulation_timeout: int = 1000):
        for _ in range(n_steps):
            self.macro_step(block_simulation_timeout)

    def macro_step(self, block_simulation_timeout: int = 10000, verbose=False):
        """
        >>> TM = "1RB1LA_1LC1RE_0LE0LD_1LC0LD_0RA---"
        >>> TM_tape = TMTape(TM, "101010111", 0, "1010111111")
        >>> str(TM_tape)
        '0∞101010111A>10101111110∞'
        >>> TM_regex_tape = TMRegexTape.from_tm_tape(TM_tape, 2, 3)
        >>> str(TM_regex_tape)
        '0∞ (01)^3+ (11)^1 A> (10)^2 (11)^3+ 0∞'
        >>> TM_regex_tape.macro_step()
        >>> str(TM_regex_tape)
        '0∞ (01)^3+ (11)^1 <A (10)^2 (11)^3+ 0∞'
        >>> TM_regex_tape.macro_step()
        >>> str(TM_regex_tape)
        '0∞ (01)^3+ <A (11)^1 (10)^2 (11)^3+ 0∞'
        >>> TM = "---1RB_---0RC_---0RD_---0RE_---1RF_---1RA"
        >>> TM_tape = TMTape(TM, "", 0, "111111")
        >>> TM_regex_tape = TMRegexTape.from_tm_tape(TM_tape, 2, 5)
        >>> str(TM_regex_tape)
        '0∞ A> (11)^3 0∞'
        >>> TM_regex_tape.macro_steps(1)
        >>> str(TM_regex_tape)
        '0∞ (10)^1 (00)^1 (11)^1 A> 0∞'
        >>> TM = "---1LB_---0LC_---0LD_---0LE_---1LF_---1LA"
        >>> TM_tape = TMTape(TM, "111111", 0, "", head_looking_after=False)
        >>> TM_regex_tape = TMRegexTape.from_tm_tape(TM_tape, 2, 5)
        >>> str(TM_regex_tape)
        '0∞ (11)^3 <A 0∞'
        >>> TM_regex_tape.macro_steps(1)
        >>> str(TM_regex_tape)
        '0∞ <A (11)^1 (00)^1 (01)^1 0∞'
        """
        block = self.read_block()
        block_tm = block.to_tm_tape(
            self.TM_bbchallenge_format, self.state, self.head_looking_after
        )

        if verbose:
            print("\t", block_tm)

        step = 0
        while step < block_simulation_timeout:
            try:
                block_tm.step(raise_if_at_extremity=True)
                if verbose:
                    print("\t", block_tm)
                step += 1
            except TMHasHalted:
                raise TMHasHalted()
            except TMHeadAtTapeExtremity:
                break

        if step == block_simulation_timeout:
            raise BlockSimulationTimeout()

        if verbose:
            print("\t", block_tm)

        regex_tape = TMRegexTape.from_tm_tape(
            block_tm, self.block_size, self.plus_threshold
        )

        n_blocks = 0

        if block_tm.head_looking_after:
            n_blocks = len(regex_tape.before_head)
            self.write_blocks(regex_tape.before_head)
        else:
            n_blocks = len(regex_tape.after_head)
            self.write_blocks(regex_tape.after_head[::-1])

        self.move_head(block_tm.head_looking_after, n_blocks)
        self.state = block_tm.state
        self.compress_and_generalise(self.plus_threshold)

    def get_plus_branches(self, verbose=True) -> list["TMRegexTape"]:
        """Using mxdy's flavor (instead of savask's), when facing a block with a plus, e.g. a^3+, we explore two branches:
        - a a^2
        - a a^3+
        """
        block = self.read_block()
        assert block.plus

        to_return: list["TMRegexTape"] = []

        if not self.head_looking_after:
            # a^3+ a <S
            new_before_head_1 = self.before_head[:]
            new_before_head_1.append(RegexBlock(block.B, 1, False))

            new_regex_tm_1 = TMRegexTape(
                self.TM_bbchallenge_format,
                new_before_head_1,
                self.state,
                self.after_head[::-1],
                self.head_looking_after,
                self.block_size,
                self.plus_threshold,
            )
            # copy is needed here for some Python weirdness, otherwise we end up with 2 copies
            # of new_regex_tm_2 in to_return
            to_return.append(copy.deepcopy(new_regex_tm_1))

            if verbose:
                print("\t< Branch 1", new_regex_tm_1)

            # a^2 a <S
            new_before_head_2 = self.before_head[:]
            new_before_head_2[-1].plus = False

            if new_before_head_2[-1].repeat > 1:
                new_before_head_2[-1].repeat -= 1
                new_before_head_2.append(RegexBlock(block.B, 1, False))

            new_regex_tm_2 = TMRegexTape(
                self.TM_bbchallenge_format,
                new_before_head_2,
                self.state,
                self.after_head[::-1],
                self.head_looking_after,
                self.block_size,
                self.plus_threshold,
            )

            # copy is needed here for some Python weirdness, otherwise we end up with 2 copies
            # of new_regex_tm_2 in to_return
            to_return.append(copy.deepcopy(new_regex_tm_2))
            if verbose:
                print(
                    "\t< Branch 2",
                    new_regex_tm_2,
                )
        else:
            # S> a a^3+
            new_after_head_1 = self.after_head[:]
            new_after_head_1.append(RegexBlock(block.B, 1, False))

            new_regex_tm_1 = TMRegexTape(
                self.TM_bbchallenge_format,
                self.before_head[:],
                self.state,
                new_after_head_1[::-1],
                self.head_looking_after,
                self.block_size,
                self.plus_threshold,
            )

            # copy is needed here for some Python weirdness, otherwise we end up with 2 copies
            # of new_regex_tm_2 in to_return
            to_return.append(copy.deepcopy(new_regex_tm_1))
            if verbose:
                print("\t> Branch 1", new_regex_tm_1)

            # S> a a^2
            new_after_head_2 = self.after_head[:]

            new_after_head_2[-1].plus = False
            if new_after_head_2[-1].repeat > 1:
                new_after_head_2[-1].repeat -= 1
                new_after_head_2.append(RegexBlock(block.B, 1, False))

            new_regex_tm_2 = TMRegexTape(
                self.TM_bbchallenge_format,
                self.before_head[:],
                self.state,
                new_after_head_2[::-1],
                self.head_looking_after,
                self.block_size,
                self.plus_threshold,
            )

            # copy is needed here for some Python weirdness, otherwise we end up with 2 copies
            # of new_regex_tm_2 in to_return
            to_return.append(copy.deepcopy(new_regex_tm_2))
            if verbose:
                print("\t> Branch 1", new_regex_tm_2)

        return to_return

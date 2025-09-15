def i2l(i: int) -> str:
    return chr(i + ord("A"))


def l2i(l: str) -> int:
    return ord(l) - ord("A")


class TMHasHalted(Exception):
    pass


class TMHeadAtTapeExtremity(Exception):
    """Used to detect cases `... S> 0∞` or `0∞ <S ...`"""

    pass


class TMTape(object):
    def __init__(
        self,
        TM_bbchallenge_format: str,
        before_head: str = "",
        state: int = 0,
        num_symbols: int = 2,
        after_head: str = "",
        head_looking_after: bool = True,
        tape_extremity_infinite_zero: bool = True,
    ):
        """
        >>> TM = "1RB1LA_1LC1RE_0LE0LD_1LC0LD_0RA---"
        >>> tm_tape = TMTape(TM, "101010111", 0, "1010111111")
        >>> str(tm_tape)
        '0∞101010111A>10101111110∞'
        >>> tm_tape.step()
        >>> str(tm_tape)
        '0∞101010111<A10101111110∞'
        """
        self.TM_bbchallenge_format = TM_bbchallenge_format
        self.state = state
        self.num_symbols = num_symbols
        self.head_looking_after = head_looking_after

        self.before_head = list(before_head)
        self.after_head = list(after_head)[::-1]

        self.tape_extremity_infinite_zero = tape_extremity_infinite_zero

    def __str__(self):
        head_str = ""
        if self.head_looking_after:
            head_str = f"{i2l(self.state)}>"
        else:
            head_str = f"<{i2l(self.state)}"
        before_head = "".join(self.before_head) if len(self.before_head) > 0 else ""
        after_head = "".join(self.after_head[::-1]) if len(self.after_head) > 0 else ""
        return (
            f"0∞{before_head}{head_str}{after_head}0∞"
            if self.tape_extremity_infinite_zero
            else f"{before_head}{head_str}{after_head}"
        )

    def read(self) -> str:
        if self.head_looking_after and len(self.after_head) > 0:
            return self.after_head[-1]
        if not self.head_looking_after and len(self.before_head) > 0:
            return self.before_head[-1]
        return "0"  # Tape extension is implemented by the write operation

    def current_transition(self) -> str:
        state_offset = 3 * self.num_symbols * self.state
        symbol_offset = 3 * current_read
        offset = state_offset + symbol_offset
        current_read = int(self.read())
        return self.TM_bbchallenge_format.replace("_", "")[
            offset : offset + 3
        ]

    def write(self, to_write: str):
        if self.head_looking_after:
            if len(self.after_head) == 0:
                self.after_head.append(to_write)
            else:
                self.after_head[-1] = to_write
        else:
            if len(self.before_head) == 0:
                self.before_head.append(to_write)
            else:
                self.before_head[-1] = to_write

    def move_head(self, move_direction: str):
        if move_direction == "L":
            if self.head_looking_after:
                self.head_looking_after = False
            else:
                self.after_head.append(self.before_head.pop())
        elif move_direction == "R":
            if not self.head_looking_after:
                self.head_looking_after = True
            else:
                self.before_head.append(self.after_head.pop())

    def head_at_extemity(self) -> bool:
        """Returns True in cases `... S> 0∞` and `0∞ <S ...`"""
        return (len(self.before_head) == 0 and not self.head_looking_after) or (
            len(self.after_head) == 0 and self.head_looking_after
        )

    def step(self, raise_if_at_extremity: bool = False):

        if self.head_at_extemity() and raise_if_at_extremity:
            raise TMHeadAtTapeExtremity()

        to_write, move_direction, new_state = self.current_transition()
        if new_state == "-":
            raise TMHasHalted()
        self.write(to_write)
        self.move_head(move_direction)

        self.state = l2i(new_state)


if __name__ == "__main__":
    # Testing TM engine on bb5 champion
    # TM = "1RB1LC_1RC1RB_1RD0LE_1LA1LD_---0LA"
    # print(TM)
    # TM_tape = TMTape(TM)
    # i = 0
    # print(i, TM_tape)

    # while True:
    #     try:
    #         TM_tape.step()
    #         i += 1
    #         if i % 1000000 == 0:
    #             print(i)
    #     except TMHasHalted:
    #         i += 1
    #         print(TM_tape)
    #         print("The machine has halted at step", i)
    #         break
    pass

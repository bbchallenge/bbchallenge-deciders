from enum import Enum
from io import BytesIO

import numpy as np


class FAR_DeciderTypes(Enum):
    FAR_DFA_ONLY = 10
    FAR_DFA_NFA = 11


class FAR_EntryHeader:
    DVF_HEADER_SIZE = 12

    def __init__(
        self, machine_id: int, decider_type: FAR_DeciderTypes, info_length: int
    ):
        self.machine_id = machine_id
        self.decider_type = decider_type
        self.info_length = info_length

    def __repr__(self):
        return str(self)

    def __str__(self):
        return f"Machine #{self.machine_id}\nDecider {self.decider_type}\nProof length {self.info_length}"

    @classmethod
    def from_bytes(cls, entry_bytes: bytes):
        entry_bytes = BytesIO(entry_bytes)
        machine_id = int.from_bytes(entry_bytes.read(4), byteorder="big")
        decider_type = FAR_DeciderTypes(
            int.from_bytes(entry_bytes.read(4), byteorder="big")
        )
        info_length = int.from_bytes(entry_bytes.read(4), byteorder="big")
        return cls(machine_id, decider_type, info_length)


class FAR_EntryDFANFA:
    def __init__(
        self,
        direction_right_to_left: bool,
        nb_dfa_states: int,
        nb_nfa_states: int,
        dfa_transitions: list[list[int]],
        nfa_transitions,
        accept_vector,
    ):
        self.direction_right_to_left = direction_right_to_left
        self.nb_dfa_states = nb_dfa_states
        self.nb_nfa_states = nb_nfa_states
        self.dfa_transitions = dfa_transitions[:]
        self.nfa_transitions = nfa_transitions[:]
        self.accept_vector = accept_vector[:]
        self.steady_state = np.zeros(self.accept_vector.shape).astype(bool)
        self.steady_state[0, -1] = True

    def __repr__(self):
        return str(self)

    def __str__(self):
        direction = "right-to-left" if self.direction_right_to_left else "left-to-right"

        return (
            f"Direction: {direction}\nNumber of DFA states: {self.nb_dfa_states}\nNumber of NFA states: {self.nb_nfa_states}\n"
            f"\nDFA transitions:\n{self.dfa_transitions}\n"
            f"\nNFA transition matrix 0:\n{self.nfa_transitions[0].astype(int)}\n"
            f"\nNFA transition matrix 1:\n{self.nfa_transitions[1].astype(int)}\n"
            f"\nAccept vector:\n{self.accept_vector.astype(int)}\n"
        )

    def DFA_to_graphviz(self, d):
        for i in range(self.nb_dfa_states):
            for r in range(2):
                from_ = str(i)
                to = str(self.dfa_transitions[i][r])
                d.edge(from_, to, label=str(r))

    def NFA_i_to_state_name(self, i) -> str:
        if i == self.nb_nfa_states - 1:
            state_name = "⊥"
        else:
            dfa_state = i // 5
            letter = chr(ord("A") + i % 5)
            state_name = str(dfa_state) + letter
        return state_name

    def NFA_to_graphviz(self, d):
        for i in range(self.nb_nfa_states):
            state_name = self.NFA_i_to_state_name(i)
            d.node(
                state_name,
                shape="doublecircle" if self.accept_vector[0, i] else "circle",
            )
        for r in range(2):
            for i in range(self.nb_nfa_states):
                state_from = self.NFA_i_to_state_name(i)
                for j in range(self.nb_nfa_states):
                    if self.nfa_transitions[r][i][j]:
                        state_to = self.NFA_i_to_state_name(j)
                        d.edge(state_from, state_to, label=str(r))

    def to_graphviz(self):
        from graphviz import Digraph

        d = Digraph()
        self.DFA_to_graphviz(d)
        self.NFA_to_graphviz(d)
        return d

    @classmethod
    def from_bytes(cls, entry_bytes: bytes):
        entry_bytes = BytesIO(entry_bytes)
        direction_byte = entry_bytes.read(1)
        direction_byte = int.from_bytes(direction_byte, byteorder="big")
        direction_right_to_left = direction_byte == 1
        nb_dfa_states = int.from_bytes(entry_bytes.read(2), byteorder="big")
        nb_nfa_states = int.from_bytes(entry_bytes.read(2), byteorder="big")
        dfa_transitions = []
        for i in range(nb_dfa_states):
            dfa_transitions.append(
                [
                    int.from_bytes(entry_bytes.read(1), byteorder="big"),
                    int.from_bytes(entry_bytes.read(1), byteorder="big"),
                ]
            )

        # Magical formula which gives the number of bytes
        # When padding with x bits with 0s
        # Equivalent to n + 7 // 8
        nb_bytes = (nb_nfa_states + 7) >> 3
        nfa_transitions = []
        for r in range(2):
            the_bytes = entry_bytes.read(nb_nfa_states * nb_bytes)
            nfa_transitions.append(
                np.unpackbits(
                    np.frombuffer(
                        the_bytes, np.uint8, count=nb_nfa_states * nb_bytes
                    ).reshape((nb_nfa_states, nb_bytes)),
                    1,
                    nb_nfa_states,
                    "little",
                ).astype(bool)
            )

        the_bytes = entry_bytes.read(nb_bytes)
        accept_vector = np.unpackbits(
            np.frombuffer(the_bytes, np.uint8, count=nb_bytes).reshape((1, nb_bytes)),
            1,
            nb_nfa_states,
            "little",
        ).astype(bool)

        return cls(
            direction_right_to_left,
            nb_dfa_states,
            nb_nfa_states,
            dfa_transitions,
            nfa_transitions,
            accept_vector,
        )


class FAR_DVF:
    def __init__(self, n_entries):
        self.n_entries = n_entries

    @classmethod
    def from_file(cls, file_path, pre_scan=True):
        f = open(file_path, "rb")
        n_entries = int.from_bytes(f.read(4), byteorder="big")

        to_return = cls(n_entries)
        to_return.file_path = file_path

        # pre scans to know the position of each entries
        if pre_scan:
            cursor_position = 4
            cursor_positions = []
            i_entry = 0
            while i_entry != n_entries:
                cursor_positions.append(cursor_position)
                header = FAR_EntryHeader.from_bytes(
                    f.read(FAR_EntryHeader.DVF_HEADER_SIZE)
                )
                cursor_position += FAR_EntryHeader.DVF_HEADER_SIZE
                f.seek(header.info_length, 1)
                cursor_position += header.info_length
                i_entry += 1

            to_return.cursor_positions = cursor_positions
        else:
            to_return.cursor_positions = None

        f.close()
        return to_return

    def ith_entry(self, f, i_entry, verbose=False, just_header=False):
        if i_entry < 0 or i_entry >= self.n_entries:
            raise EOFError(
                f"Entry {i_entry} does not exist. There are {self.n_entries} entries."
            )

        if self.cursor_positions is not None:
            f.seek(self.cursor_positions[i_entry])
        else:
            f.seek(4)  # first 4 bytes are number of entries
            curr_entry = 0
            while curr_entry != i_entry:
                header = FAR_EntryHeader.from_bytes(
                    f.read(FAR_EntryHeader.DVF_HEADER_SIZE)
                )
                f.seek(header.info_length, 1)
                curr_entry += 1

        header = FAR_EntryHeader.from_bytes(f.read(FAR_EntryHeader.DVF_HEADER_SIZE))

        if just_header:
            return header

        entry = None
        if header.decider_type == FAR_DeciderTypes.FAR_DFA_NFA:
            entry = FAR_EntryDFANFA.from_bytes(f.read(header.info_length))
        if verbose:
            print(f"Entry {i_entry}")
            print(header)

        return header, entry

    def __str__(self) -> str:
        return f"{self.n_entries} entries"

    def __repr__(self) -> str:
        return str(self)

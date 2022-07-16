import tqdm
from bbchallenge_utils import *

DB_PATH = "../all_5_states_undecided_machines_with_global_header"
UNDECIDED_INDEX_PATH = "../bb5_undecided_index"


def get_states_with_undefined_transition(machine: bytes):
    to_return = []
    nb_states = len(machine) // 6  # 3 bytes per transition, 6 per state
    for i_state in range(nb_states):
        if machine[6 * i_state + 2] == 0 or machine[6 * i_state + 5] == 0:
            to_return.append(i_state)

    return to_return


def get_states_reachable_from(
    machine: bytes, i_state: int, debug: bool = False
):
    # BFS logic
    visited = set({})
    to_visit = [i_state]
    if debug:
        print()

    while len(to_visit) != 0:
        curr_state = to_visit[0]
        to_visit = to_visit[1:]

        if curr_state in visited:
            continue

        visited.add(curr_state)

        goto_0 = machine[6 * curr_state + 2]
        if goto_0 != 0:
            to_visit.append(goto_0 - 1)

        goto_1 = machine[6 * curr_state + 5]
        if goto_1 != 0:
            to_visit.append(goto_1 - 1)

        if debug:
            print(curr_state, goto_0, goto_1, to_visit)
    return visited


def decider_closed_states(
    machine: bytes, debug: bool = False, reachables_to_return: set = {}
):
    """If the machine ever enters a closed set of states where
    all transitions are defined, then it never halts.

    Note that by:

    1. By construction, any state of a machine of bbchallenge that
    contains at least one defined transition has been reach at some point in the
    execution of the machine.

    2. Furthermore, there are no machines in the db with a state that has 2 undefined transitions,
    since this is solved by BB(4) = 107 during enumeration. This means that with point 1. we know
    that all the states of the machine are reached at some point by the machine when started from
    blank tape.

    Hence we don't need to run the machine at all, but we just need to check its code to see if there
    exists such a closed set of states where all transitions are defined. We know that it'll be reached
    and that the machine does not halt.
    """

    states_with_one_undefined_transition = get_states_with_undefined_transition(
        machine
    )

    if debug:
        pptm(machine)
        print(states_with_one_undefined_transition)

    nb_states = len(machine) // 6

    for i_state in range(nb_states):
        reachables = get_states_reachable_from(machine, i_state, debug)
        if debug:
            print(i_state, reachables)
        closed_states = True
        for reachable_state in reachables:
            if reachable_state in states_with_one_undefined_transition:
                closed_states = False
                break

        if closed_states:
            reachables_to_return.update(reachables)
            return True

    return False


if __name__ == "__main__":
    undecided_machines_indices = get_indices_from_index_file(
        UNDECIDED_INDEX_PATH
    )

    print("machine_id; machine; closed_states")

    for machine_id in tqdm.tqdm(undecided_machines_indices):
        machine = get_machine_i(DB_PATH, machine_id)
        closed_states = set({})
        if decider_closed_states(machine, reachables_to_return=closed_states):
            print(machine_id, end="; ")
            print(format_machine(machine), end="; ")
            print(set(map(lambda x: chr(ord("A") + x), closed_states)))

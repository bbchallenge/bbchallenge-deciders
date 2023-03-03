def pptm(machine, return_repr=False):
    def ithl(i):
        return chr(ord("A") + i)

    def g(move):
        if move == 0:
            return "R"
        return "L"

    from tabulate import tabulate

    headers = ["s", "0", "1"]
    table = []

    nb_states = len(machine) // 6

    for i in range(nb_states):
        row = [ithl(i)]
        for j in range(2):
            write = machine[6 * i + 3 * j]
            move = machine[6 * i + 3 * j + 1]
            goto = machine[6 * i + 3 * j + 2] - 1

            if goto == -1:
                row.append("---")
                continue

            row.append(f"{write}{g(move)}{ithl(goto)}")
        table.append(row)

    if not return_repr:
        print(tabulate(table, headers=headers))
    else:
        return tabulate(table, headers=headers)


def load_machine_from_db(machine_db_file, i, db_has_header=True):
    c = 1 if db_has_header else 0
    machine_db_file.seek(30 * (i + c))
    return machine_db_file.read(30)

import os
from tabulate import tabulate


def get_machine_i(machine_db_path, i, db_has_header=True):
    with open(machine_db_path, "rb") as f:
        c = 1 if db_has_header else 0
        f.seek(30 * (i + c))
        return f.read(30)


def get_indices_from_index_file(index_file_path):
    index_file_size = os.path.getsize(index_file_path)

    machines_indices = []
    with open(index_file_path, "rb") as f:
        for i in range(index_file_size // 4):
            chunk = f.read(4)
            machines_indices.append(int.from_bytes(chunk, byteorder="big"))

    return machines_indices


R, L = 0, 1


def ithl(i):
    return chr(ord("A") + i)


def g(move):
    if move == R:
        return "R"
    return "L"


def pptm(machine, return_repr=False):
    headers = ["s", "0", "1"]
    table = []

    for i in range(5):
        row = [ithl(i)]
        for j in range(2):
            write = machine[6 * i + 3 * j]
            move = machine[6 * i + 3 * j + 1]
            goto = machine[6 * i + 3 * j + 2] - 1

            if goto == -1:
                row.append("???")
                continue

            row.append(f"{write}{g(move)}{ithl(goto)}")
        table.append(row)

    if not return_repr:
        print(tabulate(table, headers=headers))
    else:
        return tabulate(table, headers=headers)


def format_machine(machine):
    to_return = []
    for i in range(5):
        for j in range(2):
            write = machine[6 * i + 3 * j]
            move = machine[6 * i + 3 * j + 1]
            goto = machine[6 * i + 3 * j + 2] - 1

            if goto == -1:
                to_return.append("---")
                continue

            to_return.append(f"{write}{g(move)}{ithl(goto)}")

    return "_".join(to_return)

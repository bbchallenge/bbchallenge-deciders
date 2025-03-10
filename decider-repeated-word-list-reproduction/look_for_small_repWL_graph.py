from decider import deciderRep_WL

BB5_DB = "/Users/cosmo/Documents/projects/bbchallenge/Coq-BB5/CoqBB5/BB5/BB5_Extraction/BB5_verified_enumeration.csv"
candidates = 0
with open(BB5_DB, "r") as f:
    for line in f:
        if line.strip() == "":
            continue
        if "machine" in line:
            continue
        TM = line.split(",")[0].strip()
        success, reason_failure, node_count, regex_branching_met = deciderRep_WL(
            TM,
            2,
            3,
            20,
            1000,
            False,
            False,
            True,
            "",
            False,
            False,
        )

        if success and regex_branching_met and node_count < 15:
            print(TM, node_count)
            candidates += 1

        if candidates > 1000:
            break

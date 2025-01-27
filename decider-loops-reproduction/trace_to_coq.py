f = open("trace.txt", "r")
f_content = f.read()
f.close()

# (* Define the trace as a list of (ListES * Z) *)
# Definition trace : list (ListES * Z) :=
#   (* Each line of the trace is represented as a pair (ListES, head_position) *)
#   (* Format: (ListES l r m s, head_position) *)
#   (* Example: 10 [B1] 011111111111111111 -16 *)
#   ( {| l := nil; r := string_to_list_Σ "011111111111111111"; m := Σ1; s := St1 |}, (Z.to_nat -16) ) ::
#   ( {| l := string_to_list_Σ "1"; r := string_to_list_Σ "11111111111111111"; m := Σ0; s := St2 |}, -15 ) ::

to_pp = []
for line in f_content.split("\n"):
    l, sm, r, d = line.split(" ")
    if l == "":
        l = "nil"
    if r == "":
        r = "nil"
    sm_m = sm[-2]
    sm_i = ord(sm[1]) - ord("A")
    to_pp.append(
        f'( \u007b| l := string_to_list_Σ "{l}"; r := string_to_list_Σ "{r}"; m := Σ{sm_m}; s := St{sm_i} |\u007d, (({d})%Z) ) ::'
    )

for s in to_pp[::-1]:
    print(s)

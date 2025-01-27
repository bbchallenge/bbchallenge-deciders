Require Import Lia.
Require Import List.
Require Import ZArith.

From CoqBB5 Require Import TM.
From CoqBB5 Require Import BB52Statement.
From CoqBB5 Require Import CustomTactics.
From CoqBB5 Require Import TNF.
From CoqBB5 Require Import ListTape.
From CoqBB5 Require Import Prelims.
From CoqBB5 Require Import Encodings.

(* Begin: Loop decider implementation *)

Fixpoint verify_loop1(h0 h1:ListES*Z)(ls0 ls1:list (ListES*Z))(n:nat)(dpos:Z):bool :=
  let (es0,d0):=h0 in
  let (es1,d1):=h1 in
  St_eqb es0.(s) es1.(s) &&&
  Σ_eqb es0.(m) es1.(m) &&&
  (
    match n with
    | O =>
      match dpos with
      | Z0 => Z.eqb d1 d0
      | Zpos _ =>
        match es1.(r) with
        | nil => Z.ltb d1 d0
        | _ => false
        end
      | Zneg _ =>
        match es1.(l) with
        | nil => Z.ltb d0 d1
        | _ => false
        end
      end
    | _ => false
    end |||
    match ls0,ls1 with
    | h0'::ls0',h1'::ls1' =>
      verify_loop1 h0' h1' ls0' ls1' (Nat.pred n) dpos
    | _,_ => false
    end
  ).

Fixpoint find_loop1(h0 h1 h2:ListES*Z)(ls0 ls1 ls2:list (ListES*Z))(n:nat){struct ls1}:bool :=
  (
    (let (es0,d0):=h0 in
    let (es1,d1):=h1 in
    St_eqb es0.(s) es1.(s) &&&
    let (es2,d2):=h2 in
    St_eqb es0.(s) es2.(s) &&&

    Σ_eqb es0.(m) es1.(m) &&&
    Σ_eqb es0.(m) es2.(m) &&&

    (verify_loop1 h0 h1 ls0 ls1 (S n) (d0-d1)))
  ) |||

  match ls2,ls1 with
  | h3::h2'::ls2',h1'::ls1' =>
    find_loop1 h0 h1' h2' ls0 ls1' ls2' (S n)
  | _,_ => false
  end.

Definition find_loop1_0(h0 h1:ListES*Z)(ls:list (ListES*Z)):bool :=
match ls with
| h2::ls' => find_loop1 h0 h1 h2 (h1::ls) ls ls' O
| nil => false
end.

(** Loop decider aux function

Args:
  - tm: TM Σ, the Turing machine that the NGramCPS decider is checking.
  - n: nat, gas parameter
  - es:ListES, current configuration (ExecState), using the `ListES` representation, see ListTape.v
  - d: Z, current head position index on the tape
  - ls: list (ListES*Z), list of visited configurations and head positions

Returns:
  - HaltDecideResult:
    - Result_Halt s i: the Turing machine halts at configuration (s,i)
    - Result_NonHalt: the Turing machine does not halt
    - Result_Unknown: the decider cannot conclude
*)
Fixpoint loop1_decider0(tm:TM Σ)(n:nat)(es:ListES)(d:Z)(ls:list (ListES*Z)):HaltDecideResult :=
match n with
| O => Result_Unknown
| S n0 =>
  match tm es.(s) es.(m) with
  | None => Result_Halt es.(s) es.(m)
  | Some tr =>
    let es' := ListES_step' tr es in
    let d' := (d+Dir_to_Z tr.(dir _))%Z in
    let ls' := ((es,d)::ls) in
    match n0 with
    | S n0' =>
      loop1_decider0 tm n0 es' d' ls'
    | O =>
      if find_loop1_0 (es',d') (es,d) ls then Result_NonHalt else
      loop1_decider0 tm n0 es' d' ls'
    end
  end
end.

(** Loop decider

Args:
  - n: nat, gas parameter
  - tm: TM Σ, the Turing machine that the NGramCPS decider is checking.

Returns:
  - HaltDecideResult:
    - Result_Halt s i: the Turing machine halts at configuration (s,i)
    - Result_NonHalt: the Turing machine does not halt
    - Result_Unknown: the decider cannot conclude
*)
Definition loop1_decider(n:nat)(tm:TM Σ):HaltDecideResult :=
  loop1_decider0 tm n {| l:=nil; r:=nil; m:=Σ0; s:=St0 |} Z0 nil.
Require Import ZArith.
Require Import Lia.
Require Import List.
Require Import FSets.FMapPositive.

From CoqBB5 Require Import Prelims.

From CoqBB5 Require Import BB52Statement.
From CoqBB5 Require Import Encodings.
From CoqBB5 Require Import CustomListRoutines.

Section NGramCPS.

Hypothesis Σ:Set.
Hypothesis len_l:nat.
Hypothesis len_r:nat.

Hypothesis Σ_enc: Σ->positive.
Hypothesis Σ_enc_inj: is_inj Σ_enc.
Hypothesis listΣ_enc: (list Σ)->positive.
Hypothesis listΣ_enc_inj: is_inj listΣ_enc.

Hypothesis Σ0:Σ.

Record MidWord:Set := {
  l:list Σ;
  r:list Σ;
  m:Σ;
  s:St;
}.

Definition MidWord_enc(mw:MidWord):positive :=
  let (l,r,m,s):=mw in
  enc_list ((St_enc s)::(Σ_enc m)::(listΣ_enc l)::(listΣ_enc r)::nil).

Definition xset_impl:Type := (PositiveMap.tree (SetOfEncodings Σ)).
Definition mset_impl:Type := SetOfEncodings MidWord.

(* Abstract Exect State *)
Record AES_impl := {
  lset': xset_impl;
  rset': xset_impl;
  mset': mset_impl;
}.

Definition xset_ins0(xs:xset_impl)(v:SetOfEncodings Σ)(x1:list Σ)(x2:Σ):xset_impl*bool :=
  let (v',flag):=(set_ins Σ_enc v x2) in
  (PositiveMap.add (listΣ_enc x1) v' xs, flag).

Definition xset_ins(xs:xset_impl)(x:list Σ):xset_impl*bool :=
  match x with
  | h::t =>
    let (x1,x2):=pop_back' h t in
    match PositiveMap.find (listΣ_enc x1) xs with
    | Some v =>
      xset_ins0 xs v x1 x2
    | None =>
      xset_ins0 xs (nil, PositiveMap.empty unit) x1 x2
    end
  | nil => (xs,false)
  end.

Definition mset_ins0(ms:mset_impl)(mw:MidWord):mset_impl*bool :=
  set_ins MidWord_enc ms mw.

Fixpoint mset_ins(q:list MidWord)(ms:mset_impl)(flag:bool)(f:Σ->MidWord)(ls:list Σ):((list MidWord)*mset_impl)*bool :=
match ls with
| nil => ((q,ms),flag)
| h::t =>
  let (ms',flag'):=mset_ins0 ms (f h) in
  let q' := if flag' then q else ((f h)::q) in
  mset_ins q' ms' (andb flag flag') f t
end.

Definition xset_as_list(xs:xset_impl)(x1:list Σ):list Σ :=
  match PositiveMap.find (listΣ_enc x1) xs with
  | Some v => fst v
  | None => nil
  end.

Definition update_AES_MidWord(tm:TM Σ)(q:list MidWord)(mw:MidWord)(SI:AES_impl):((list MidWord)*AES_impl)*bool :=
let (l0,r0,m0,s0):=mw in
let (ls,rs,ms):=SI in
  match l0,r0 with
  | hl::l1,hr::r1 =>
    match tm s0 m0 with
    | Some tr =>
      let (s1,d,o):=tr in
      match d with
      | Dpos =>
        let (ls',flag1):= xset_ins ls l0 in
        match
          mset_ins q ms true
            (fun x => {|
              l:=o::(pop_back hl l1);
              m:=hr;
              r:=r1++(x::nil);
              s:=s1;
            |}) (xset_as_list rs r1)
        with (q',ms',flag2) =>
          ((q',{| lset':=ls'; rset':=rs; mset':=ms' |}), andb flag1 flag2)
        end
      | Dneg =>
        let (rs',flag1):= xset_ins rs r0 in
        match
          mset_ins q ms true
            (fun x => {|
              r:=o::(pop_back hr r1);
              m:=hl;
              l:=l1++(x::nil);
              s:=s1;
            |}) (xset_as_list ls l1)
        with (q',ms',flag2) =>
          ((q',{| lset':=ls; rset':=rs'; mset':=ms' |}), andb flag1 flag2)
        end
      end
    | _ => ((q,SI),false)
    end
  | _,_ => ((q,SI),false)
  end.

Fixpoint update_AES(tm:TM Σ)(ms:list MidWord)(SI:AES_impl)(flag:bool)(n:nat):AES_impl*bool*nat :=
  match n with
  | O => (SI,false,O)
  | S n0 =>
    match ms with
    | nil => (SI,flag,n0)
    | mw::ms0 =>
      let (S',flag'):=update_AES_MidWord tm ms0 mw SI in
      let (q',SI'):=S' in
      update_AES tm q' SI' (andb flag flag') n0
    end
  end.

Definition check_InitES_InAES (S:AES_impl):bool:=
  let (ls,rs,ms):=S in
  (snd (mset_ins0 ms {| l:=repeat Σ0 len_l; r:=repeat Σ0 len_r; m:=Σ0; s:=St0 |}) &&
  snd (xset_ins ls (repeat Σ0 len_l)) &&
  snd (xset_ins rs (repeat Σ0 len_r))) %bool.

Fixpoint NGramCPS_decider_0(m n:nat)(tm:TM Σ)(S:AES_impl):bool :=
match m with
| O => false
| S m0 =>
  match update_AES tm (fst (mset' S)) S true n with
  | (S',flag,n0) =>
      if flag then check_InitES_InAES S'
      else NGramCPS_decider_0 m0 n0 tm S'
  end
end.

Definition NGramCPS_decider(m:nat)(tm:TM Σ):bool :=
  match len_l,len_r with
  | S _,S _ =>
    NGramCPS_decider_0 m m tm
    {|
      lset':=fst (xset_ins (PositiveMap.empty _) (repeat Σ0 len_l));
      rset':=fst (xset_ins (PositiveMap.empty _) (repeat Σ0 len_r));
      mset':=fst (mset_ins0 (nil,PositiveMap.empty _) {| l:=repeat Σ0 len_l; r:=repeat Σ0 len_r; m:=Σ0; s:=St0 |});
    |}
  | _,_ => false
  end. 

End NGramCPS.

Import ListNotations.
Print xset_ins.

Compute fst (xset_ins Σ Σ_enc listΣ_enc (PositiveMap.empty _) ([Σ1;Σ0])).


Definition NGramCPS_decider_impl2_0 (len_l len_r m:nat) tm :=
  NGramCPS_decider Σ len_l len_r Σ_enc (listΣ_enc) Σ0 m tm.

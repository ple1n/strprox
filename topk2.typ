
Notes for http://www.vldb.org/pvldb/vol9/p828-deng.pdf

- Theorems have been further generalized than what is in the paper
- Corretions for errors
- More precise defintions
- Better notation

#set heading(numbering: "1.")

#heading(numbering: "1.", "Matching prefix")

\

consider a full query string, $q=q[1,"len"]$

$
cases(q[1,j], 0<=j<="len"=|q|, q[1,0]=phi.alt) 
$

we are currently searching $q[i]$

consider any full stored string $s$, a prefix is $s[1,k]=s_k$

an (end) matching prefix $s[1,j]$ is defined to be a prefix ending with $q[i] = s[j]$ $=> "ED" := "ED"(q[1,i],s[1,j]) = "ED"(q[1,i-1],s[1,j-1])$ 

which calculates the ED between current query and the matching prefix.

for substring $q[1,i-1] "and" s[1,j-1]$, set of matchings is $M_(q,s)$ where $"ED"=min_(m in M)(m_(|i-1|,|j-1|))$

Therefore for a *matching prefix*, it must be possible to find the minimal $m_(k,|i-1|,|j-1|)$. This computation depends on $m_k$ and $(|i-1|,|j-1|)$ determinstically

The root node has $"ED"=1$, and $m^"root"_(k,|i-1|,|j-1|)=max(|i-1|,|j-1|)$ For a matching prefix and a $q_i$ they may have no other matchings except root. At this point the theorem still holds. (according to the paper it seems.)  

$M(q_0,s)={(i=0,j=0,"ED"=0)}$

To make sure we can calculate the matching prefix ED to query, we make sure for a newly discovered matching prefix, all previous matchings are discovered. 

Equally, finding matchings further from a known matching makes sure the set of "previous matchings" is exhaustive.

Q1: It's not clear how that way of scanning descendents against $q[i]$ the "previous matchings" are exhaustive

Thus, the ED of $q[i]$ and $n$ (prefix of node n) is computed by iterating over "previous matchings"

Q1 is answered by, $m$ is an active matching of $q_(i-1)$. $m in A subset.eq M$. Therefore we dont need exhaustive $M$. 

$M=M(q_(i-1),n."parent")$. Trivially, $A subset.eq M$. 

Require $"eq"(q_i,n)=min(m_(i-1,|n|-1))<=tau => $ We need $m_(i-1,|n|-1) <= tau$, Find min. 

$
cases(
  "for any" m  \
  m_(|q|)=m."ed"+|q|-m.i \
  m_(|i-1|) <= m_(i-1,|n|-1) ("trivial")
)
=>  forall "m we need", m_(|q|) <= tau
\
m in A_q <=> m_(|q|) <= tau
=> forall m_("need") in A(q_(i-1))
$

Therefore $M(q_(i-1),n."parent")$ (the previous matchings) is not needed, but only the relevant subset.

Q2: How is $A(q_(i-1))$ exhaustive by that algorithm. ie. prove for some q $forall  m_(|q|) <= tau => m in A_q$

For some $q, |q|=i, forall  m_(|q|) <= tau => m in A_q$
$
cases(
  1. forall m.i = |q| = i space ("end matching prefixes"),  
  2. forall m.i < i  space ("taken from" A(q_(i-1))) 
)
$

Type 1 is collected through iterating over descendents, and filtering the matchings by $"ed" < tau$, while ed is computed by looping over $A(q_(i-1))$. One ed is computed for each $j-1 => j$

Q3: How is type 1 searching exhaustive. 

$tack "for certain depth of nodes", forall m in A(q_(i-1)), m_(i-1,|n|-1) > tau $, so they are always excluded.

$tack "for" m_1(i_1,n_1) in A(q_(i-1)), forall m_2 :=(i_2=|q|, n=(c,d)), 
d in.not [n_1.d+1,n_1.d+1+tau] => m_2."ed" > tau$

$
m_2(|q|)=m_2."ed"+|q|-m_2.i=_(m_2.i=|q|)m_2."ed" \
"lev"(a,b) in [ |\|a|-|b|\|, max(|a|,|b|)] \
m_2."ed"="ed"(q,n) in [ |i_2-d|, max(i_2,d)]
$

The paper does utilize features of some particular edit distance algorithm, which are assumptions. TODO: list them later.

No this is different that what is presented in the algorithm. 

$m_2."ed"_min > tau => m_2."ed" > tau => |i_2-d| > tau $

$tack d in.not [i_2-tau,i_2+tau] =K => m_2."ed" >_"certainly" tau \ 
tack m_2."ed" <= tau => d in [i_2 -tau, i_2 + tau]
$

$
forall m={q_i,s_j} => m."ed" <= tau =>m."ed">=|i-j| \
=>|i-j|<=tau \
not |i-j|<=tau =>not m."ed"<=tau
$

#align(end + horizon, "theorem ed-delta")

$
m."ed"=b =>m."ed"<=b and m."ed">=b \
=> (forall m => m."ed"_min=|i-j|<=b)  and (forall m=> m."ed"_max =max(|q|,|s|) >= b)
$

== Experiment 

```rs
    fn first_deducing(
        &'stored self,
        active_matching_set: &MatchingSet<'stored, UUU, SSS>,
        character: char,
        query_len: usize,
        threshold: usize,
    ) -> MatchingSet<'stored, UUU, SSS> {
        let mut best_edit_distances = HashMap::<SSS, UUU>::new();
        for matching in active_matching_set.iter() {
            let node = matching.node;
            let node_prefix_len = node.depth as usize;
            // lines 5-7 of MatchingBasedFramework, also used in SecondDeducing
            for depth in node_prefix_len + 1
                ..=min(
                    node_prefix_len + threshold + 1,
                    self.inverted_index.max_depth(),
                )
            {
                self.traverse_inverted_index(&matching, depth, character, |descendant| {
                    // the depth of a node is equal to the length of its associated prefix
                    let bound = matching.deduced_edit_distance(
                        query_len - 1,
                        node.depth.saturating_sub(1) as usize,
                    );
                    let bound = bound as UUU;
                    let id = descendant.id() as SSS;
                    let pred = depth >= query_len - threshold && depth <= query_len + threshold;
                    if !pred {
                        let k = bound <= threshold as UUU;
                        if k {
                            println!("breach");
                        } 
                    }
```

The above code, via hand-testing, seems to work.

The `best_edit_distances` is a map, $n_2 -> "ed"$ 

$
m_1(|q|-1,|n_2|-1)=m_1."ed"+x_(x>=0)<=tau => m_1."ed"<=tau
$
Thus we can find the minimum of it given exhaustive $m_1$.

$
m_2(i=|q|,n_2) \
"by lev", n_2.d in K\
forall n_2, "all " m_1 in A(q_(i-1)) "are visited" \
n_2."ed" = min(m_1(i-1,|n|-1)) "one value per" m_1, |n|
$

Q4: I'm not sure what justifies the $[\|n|+1,|n|+1+tau]$

$
"for an " m_2(i=|q|,n_2), forall "s" in n_2, exists p = s_(|n|), s.t. "ed"(q,p) <= tau 
  => s in R(q,T)
$

For other matchings, EDs are over $q_(k), k<i=|q|$. EDs over $q_i$ are not necessarily $<= tau$

On lemma 2

$
  "PED"(q,s)=min_(m in M(q,s))(m_(|q|)) \
  tack  "PED"(q,s) = k => exists m_1  in M(q,s), "st." m_1(|q|)=k
$

This is what the paper implies.

$
forall (q,s), "ped"(q,s) = k => exists m_1(q_i,s_j), "st." m_1(|q|)=k \
"given" m_1(|q|)=k, forall s in m_1, "ped"(q,s) <= k 
$

$m:=(q_i,s_j)=(i,n=s_j,"ed")$

Prove $ M={m | m(|q|)<=k} "produces an exhaustive" R, forall s in R, "ped"(q,s)<=k $

$
forall s, "ped"(q,s)=k_1<= k => exists m_1(|q|)=k_1<=k, m_1 in M
$

Inverted Index $f_i: d->c->"vec"_"node"$ 

== Theorem for $m_1$ <algo_expand>

Let me call this, mathing set expanding algorithm, as it expands the matching set based on an updated $q$, finding all m such that $m."ed" <= tau$, based on $M_t$.

Further reducing the search range

We require $m_1(|q|-1,|n_2|-1)<=tau$.

$
forall m_1, n_2 => m_1(|q|-1) <= m_1(|q|-1,|n_2|-1) <= tau => m_1 in A(q_(i-1))  \
"when" m_1.i <= i-1
$

$
forall m_1, n_2 => m_1(|q|-1,|n_2|-1)<=tau => m_1 in P(i-1,tau)
$

$
m_1=(i_1,n_1=(c_1,d_1)) \
cases(
"we require" k=m_1(|q|-1,|n_2|-1) = m_1."ed"+max(|q|-1-i_1,|n_2|-1-|n_1|) <= tau \
k>=|n_2|-1-|n_1| 
) \ 
=> |n_2|-1-|n_1| <= tau => |n_2|<=|n_1|+tau+1 \
"by defintion of m(q,s)", q>=m.i and s>=m.j \
=> |n_2|>=|n_1|+1
\
cases(
|q|-1-i_1 <=tau => i_1 >= |q|-1-tau  \ 
m_1."ed" <= tau
) "this is per" m_1 ", the paper didn't talk about this" 
$

which holds, given $m_1$ exists.

$
forall m_1,m_1(alpha,|n_2|-1) <= tau 
=> |n_2| in [ |n_1|+1,|n_1|+tau+1 ]
\
"Narrow down the search domain by" P(m_1,n_2) => Q(m_1,n_2) "which is an interval" \
"Make sure" forall n_2, P(m_1,n_2) \
P, Q "for propositions"  \
$

Every set found by $P$ is a partial. 

$m_1(|q|-1,|n_2|-1)<=tau tack$ the matching set in question $M_t=M(q_(-1),n_2.s_(-1))$

The partials are aggregated by iterating over $m in M_t$. For each iteration find a partial.

$
forall m_1 in M_t => m_1(|q|-1) <= tau "which means we probably already have it" \
M_t subset.eq M_2 = {m_1|m_1(|q|-1) <= tau}
$

$M_t$ is exotic and can not be obtained, as it's different from each $n_2.s_(-1)$. 

We just iterate over $M_2$. When we find an $n_2$, we check that $P(m_1,n_2)$ holds.

So, this is the core algorithm that produces matchings based on previous matchings.

In $m_1(|q|-1) <= m_1(|q|-1,|s|)=x$, the $x$ part looks creative. The right-hand of *max* can be anything. 

$
m_1(|q|-1) <= m_1(|q|-1,|n|)
$

Any number put in the $|n|$ place, due to the nature of this formula, must mean a node depth.

By introducing, the $m(a,b)$ on the right, we establish a variable of $|n|$.

The end goal is to have $"ed"(q,n_2)<=tau$

$
m_2(|q|) =_(i=|q|) m_2."ed"  <= tau => forall s in m_1.S, "ped"(q,s)<=tau
$

$m_1(i-1,|n_2|-1)$ is an upper bound of $"ed"(q_(i-1),n_2."parent")$

$
"ed"(q_(i-1),n_2."parent") =_(q[i]=n_2."char") "ed"(q_i,n_2)
$

Therefore $m_1(|q|-1,|n_2|-1)<=tau$ but it's an over-requirement.

$
S_(-1):={s,|s|=|n_2|-1}, M_(-1)=M(q_(i-1),n_2."parent"), m_1 in M_(-1)
$

The condition is only satified by a subset of $S_(-1)$, denote it as $S'$

$
forall s in S' => m_1 in M_(-1) (=> s in n_1.S) \
 p_1 :m_1(|q|-1,|s|)<=tau
$

The target set is $S_t = {s, |s| = |n_2|-1 and "ed"(q_(i-1),n_2."parent")<=tau}$

Not every $s in S_t$ satisfies $p_1$

+ If $m_1(|q|-1,|s|) = "ed"(q_(i-1),s)$, the condition keeps $s$, and $s$ meets the goal. \
  There is no $<$ case. \
  $m_1$ is the minium in M.\
  Denote $S(m_1), forall s in S(m_1) => m_min=m_1$. 
  We can retrieve the complete $S(m_1)$ by this condition
+ If $m_1(|q|-1,|s|) > "ed"(q_(i-1),s)$, the condition might drop $s$ \
  Nodes in this case are dropped.

By iterating over every $m in M_(-1)$, for each iteration, we get $S(m)$ \

The loop composes the $S_t$, which is complete.

As, for each $s' in S_t$, the associated $M=M_(-1)$

$
cases(
m'_min in M_(-1) \
forall m in M_(-1) => S(m) subset.eq S_t
) => s' in S(m'_min) subset.eq S_t
\
S_T = S_t "extending each string by" q[i] \
forall s in S_T, 
"ed"(q,s) <=tau
$



== Theorem when $m.i < |q|$

$
beta = {m|m in A(q_i) and m.i < i=|q|} \
alpha = {m|m in A(q_i) and m.i = i=|q|}\

forall m, i, cases(
  m_i=m."ed"+ i-m.i,  
  m_(i-1)=m."ed"+(i-1)-m.i = m_i-1
)
\
forall m in beta => m in A_(i-1) \
m_(i)<=tau=>m_(i-1)=m_i-1=tau-1<=tau => m in A_(i-1)
\
m in A_(i-1) arrow.r.double.not m in beta
\
forall m in A_i, m.i<=i => forall m in A_(i-1), m.i <= i-1<i
$

Therefore in original code it filters the set, $A_(i-1)$ before taking it.

== Node and inverted  index

$
n={|n|="depth",c="character",N,S} \
N "for set of descendents",
S "for set of strings" \
f_i:d->c->vec_n
$ 

When searching, it looks for 
$f_i (d,c) sect n.N$, as (end) matchings.

Process of $sect$ takes a binary search. 
$vec_n$ is a sorted list, N is a range.

for two nodes $n_1$ is a descendant of $n_2 <=> n_1.N subset n_2.N$ 

$
 f_i (d,c)
$

The paper proposes to *aggregate* matchings $m(i,n)$ by node, which removes redundant binary search. For each $m_2$, $m_1$ are enumerated group by group. 

For $N_1=n_1.N subset n_2.N$, the binary search of $n_2$ is dropped, checks are performed on $N_1$, with some unnecessary nodes, but the search should be more expensive. The checks themselves suffice, so using $N_1$ instead of $N_2$ does not cause any problem.

== Active matching set

Lemma 2,

$
forall (q,s), "ped"(q,s)=min_(m in M(q,s))m_(|q|) \
A_i => forall m in A_i, m_(|q|=i)<=tau \
=> (forall m, forall s in m.S, exists m_1=m in M(q,s), m_1(|q|)=k<=tau \
=>"ped"(q,s)<= k
)
$

Any $s$ with that matching has a ped of at most k.

== TopK

$
q, R_q "for results of" q
$

Q1: Does the paper mean, by top-k, $|R_i|=k$  must be true ?

$
R_i:= R(q_i) \
forall s in R_(i-1), "ped"(q,s)<="ped"(q_(i-1),s) + 1\
=> R_(i-1) subset R_i "with ped upper bound (otherwise trivial)" \
=> (forall R_(i-1),R_i  => b_i <= b_(i-1) +1)
$

By deleting one char from $q$, which is the upper bound. 

$
"the trivial case": forall s, "ped"(q,s)=k => s in R_i
$

$
p_1:forall (q,s,i), "ped"(q_i,s) >= "ped"(q_(i-1),s) \
"when both sets are not capped":forall s in S => s in R_i and s in R_(i-1)  => p_1 
$

$
b_i:="ped"(q_i,s_b^i)=max_(s in R_i)("ped"(q_i,s))
$ (defines notation the associated s)

To prove $b_i >= b_(i-1)$

$
cases(
  1. s_b^i = s_b^(i-1) =>_p_1 "ped"(q_i,s_b^i) >=  "ped"(q_i,s_b^(i-1)) \
  2. s_b^i != s_b^(i-1): forall s in R_i\,s != s_b^i =>  "ped"(q_i,s_b^i)>= "ped"(q_i,s)  \
  s_b^(i-1) in R_i => b_i = "ped"(q_i,s_b^i) >= b_(i-1)
)
$

=== More, on the assumptions

Treating them as stateful variables, we can always add $R_(i-1)$ to $R_i$. $forall s in R_(i-1), "ped"(q,s)<="ped"(q_(i-1),s) + 1$. Thus this subset of $R_i$ has a max ped of $b_(i-1) + 1$. Trivially, its always possible to add some absurdly high-ped $s$ to $R_i$.

In the first case, by $p_1$, $b_i >= b_(i-1)$. In the second case, any other string has a ped $<=$ that of $b_i$, which includes $b_(i-1)$

Therefore, we assume we always want to get _best_ or better matchings into $R$, which is in motion. Thus, $b_i <= b_(i-1) + 1$ because we can always use $R_(i-1)$ as the upper bound.

In the same kind of motion, $b_i$ is either from $s_b^(i-1)$ or some other string with worse ped. Again, we can always add $s_b^(i-1)$ to $R_i$. Nothing prevents this. Now we have added $s_b^(i-1)$. We discuss the result by two cases, by making two hypotheses.

$
b_i = b_(i-1) "or" b_(i-1) + 1
$

If $s_b^(i-1) in.not R_i$, we want $forall s in R_i, "ped"(q,s)<=s_1 in (K=S-R_i) forall s_1$

$
"by" b_i = "ped"(q,s_b^i), s_b^i  in R_i, s_b^(i-1) in K => b_i <= b_(i-1) \
R_i != phi.alt
$
2. $s_b^i in R_i => b_i >= b_(i-1)$

It seems $R_i$ is treated as a changing variable. 

=== Reiterate

+ $R_i= phi.alt$

  There is no $b_i$. 

+ $R_i = {"any" s, "ped"(q,s)=0}$

  $b_i = 0 <= b_(i-1)$
  In this case no theorems stated in the paper work. 

+ $exists s_1 in R_(i-1) and s_1 in R_i$ 

  $forall s, "ped"(q,s)<="ped"(q_(i-1),s) + 1\
  => "ped"(q,s_1)<="ped"(q_(i-1),s_1) +1 \
  forall (q,s,i) "ped"(q_i,s) >= "ped"(q_(i-1),s) \
  => "ped"(q,s_1) >= "ped"(q_(i-1),s_1) \
\

  $
  
=== New theorem 

I dislike the notion of $b_i$ as it is ambiguous. 

$
forall s => "ped"(q_(i-1),s) <= "ped"(q,s) <="ped"(q_(i-1),s) + 1
$

$
forall s => "ped"(q_(i-2),s) <= "ped"(q_(i-1),s) <= "ped"(q,s) <="ped"(q_(i-1),s) + 1 <= "ped"(q_(i-2),s) + 2\
"ped"(q_(i-1),s) <= "ped"(q_(i-2),s) + 1
$

The bounds are determined by *available information*.

- With $ "ped"(q_(i-2),s)$, we can determine a coarse bound.
- With $"ped"(q_(i-1),s)$, it can be further narrowed down.

== Revisitng basic concepts

A matching $m={q_i,s_j}$. This should be the complete information. 

A node is either $n=phi.alt$, or $n=s_j$ (complete information)

$m$ can be used to calculate an upper bound of ED, for any $q,s$. The function only requires $|q|, |s|$, which is $m(|q|,|s|)=m."ed"+max(|q|-m.i,|s|-m.j)$.

m can be used to calculate an upper bound of PED, for any $q$. The function only requires $|q|$
$
m(|q|)=m."ed"+|q|-m.i
$

Here, "any $q$" must extend $q_i$. ($q_i = q[1,i]$). Otherwise $m."ed"$ makes no sense.

// Theorems

=== Theorem upper-bounding PED of Leaves

Given a $m$ and a $|q|$, we can determine the upper bound of PED for all strings sharing $m$. It doesn't even matter what $q$ is.

This can be deduced by supposing $M(q,s)={m}$, applying the equation in the paper, and the actual set can only add more members so it can only get lower.

$
forall s in n_m.S => m in M(q,s) => "ped"(q,s) <= m(|q|)
$

=== Theorem upper-bounding ED of leaves

calculate a upperbound ED, $x= m(|q|,|s|)$, given $|q|,|s|=k$, 

$
forall s in n_m.S and |s|=k => m in M(q,s) => "ed"(q,s) <= x= m(|q|,|s|) \ 

forall s in n_m.S and |s| < k => m in M(q,s) => "ed"(q,s) <= m(|q|,|s|) < x
$

The situation gets complex for $|s| > k$, and it's not worth talking about.

== b-matching

$
forall m in A_i <=> m_(|q|) <= tau \
m_(|q|)=_(m.i=|q|)m."ed" <= tau \
m.i<|q| => m."ed" < m_(|q|)<= tau \
\
m={q_i,s_j}
$

For a continuous query $q$.

*Definition* $forall m in M(q_i) =>$ $m."ed"<=b <=>$ m is a b-matching of $q_i$

$
forall s in S, q =>"ped"(q,s)=k <= b 
=> "ped"(q,s)= exists m_min (|q|)=k=m."ed"+|q|-m_min .i <=b 
=> m."ed" <= b
\
m_min := "any" m "at" min_(m in M(q,s,))(m_(|q|)) := m_min^M(p,q)
$

Suppose we have a set $P(q,b),forall m."ed"<=b => m in P(q,b)$

$forall s in S, "ped"(q,s)<=b => m_min."ed"<=b =>m in P(q,b)$ 

Existence of such a string $=>$ It's reachable from $P(q,b)$

Notice, there is an error in `6.2 Calculating the b-Matching Set`.

== $P_1=P(q_(i-1),b) -> P_2=P(q_i,b-1)$

$
P_2 = cases(
  m.i < i => m in P(i-1,b-1) 
  \
  m.i = i => "use the expansion algo to find all "  m "such that" m."ed" <= tau =_("here") b-1
) \ 
m.i < i and m."ed" <= b-1 <= b => (
  forall m in P_2 => m in P_1 ,\
  exists m in P_1 in.not P_2
)
$

// prove the adapted algo of @algo_expand 

=== Reiterated @algo_expand

The goal is to find *all* $n_2$ such that $phi:"ed"(q_i,n_2)<=tau$.

$m_2={q_i,n_2}=>q[i]=n_2."char"$

$
forall n_2 => "ed"(q_(i-1), n_2.p)="ed"(q_i,n_2) "by assumption" \
forall m_1 => forall n_2 in m_1.D => (
  forall q => alpha: m_1(|q|-1,|n_2|-1) <= tau => cases(
    |n_2| in [ |n_1|+1,|n_1|+tau+1 ] \
    i_1 >= |q| - 1 - tau \
    m_1."ed" <= tau
  )
)
$

$
P => Q tack not P => not Q \
forall m_1 => alpha =>m_1."ed" <= b-1 =>m_1 in P(i-1,b-1)
$

For a $m$ and a node 

+ $n_2$ must be a descendant as that is by defintion of DED. (This is implied by 2)
+ $alpha$ holds implies $Q$. Any violation negates $alpha$

To find all $n_2$ for $m_1$ that satisfies $alpha$, we only need to search $Q$, as $overline(Q)$ has been negated.

We want a jump from $m_1(a,b)$ to $"ed"(q_(i-1),n_2.p)$. That requires $m_1=m_min in M(q_(i-1),n_2.p)$.

For $f: m_1 -> N ="set of nodes"$

$ 
forall n_2 in f(m_1) := N => alpha and cases(delim:"|",
  m_1=n_2.m_min  \
  m_1 != n_2.m_min "(the over-required part)" 
) \ 
m_min <=> exists M_"ed" (|a|,|b|), m_min (|a|,|b|) = "ed"(a,b)
$

Thus $f$ is finding all $n_2 in m_1.N and alpha(n_2)$, which is not what we want as it concerns $alpha$ not ed-based.

$
"ed" "is about" "ed"(a,b),M_"ed",m_min \
n_2.m_min ->_"refers to" "any" "ed".m_min "where" "ed" = "ed"(q_i,n_2) \
forall m,n =>alpha(m,n) => phi(n) \
forall n, phi(n) arrow.double.not (forall m =>alpha(m,n)) \
forall n, phi(n) => exists m,alpha(m,n)
$

$m_1=n_2.m_min <=> exists m_min (|a|,|b|)="ed"(q_i,n_2) $  where $(|a|,|b|)$ is from a suitable ed target such that $"ed"(a,b)="ed"(q_i,n_2)$.

$f(m_1)$ produces two set of nodes, the *exact set*, $e$, and the *over-required*, $rho$.

$
f(m_1)=e union rho \ e sect rho = phi.alt \
forall n in e => n.m_min = m_1
tack forall n.m_min != m_1 => n in.not e \
"and given" n "we know its" m_min \
$

Exhaustiveness:

$f$ is exhaustive over $n_2$. 

$
forall m_1 => forall n_2,alpha(m_1,n_2)=>n_2 in f(m_1)
$

It remains exhaustive by adding an AND.

$
forall m_1 => forall n_2,alpha(m_1,n_2) and exists n_2.m_min=m_1 "where the ed matches" alpha =>n_2 in f(m_1) \
n_2 in e \
exists n_2.m_min=m_1 "where the ed matches" alpha => (alpha(m_1,n_2)  <=> phi(n_2)) "denote it as" phi_1 \
forall m_1 => forall n_2,phi_1(n_2) and exists n_2.m_min=m_1 "where the ed matches" alpha => n_2 in e subset.eq f(m_1) \
n.m_min "must be in a context of" {n.M_"ed",|a|,|b|} "to make sense"
$

When $m_1."ed"$ matches $alpha$, it means $alpha$ has the DED computed for $m_1."ed"$. 

same goes for $rho$. Both $e$ and $rho$ are exhaustive.

$
phi(n_2) => exists m, alpha(m,n_2) \
"due to limited information, we know" n_2.m_min "can be any" m in n_2.M_"ed"  \
M_"ed" = "ed".M "for any ed such that ed" = "ed"(q_i,n_2)\
forall n, phi(n) => n in S:=union.big_(n.m_min forall n) f(m) \
forall n, phi(n) => n in f(n.m_min) subset.eq S

$

by covering every possible $m$, we eliminate the right hand of AND. This is exhaustive due to $forall$

$
forall n_2,phi(n_2) => n_2 in e subset.eq f(n_2.m_min) "by having" n_2.m_min=m_1 "where the ed matches" alpha "of" f \
"also it cannot be proved that" e = f(n_2.m_min) 
$


We don't compute $n.m_min$, so $f(n.m_min)$ can not be known, but we can compute $S$. The set $M_min = n.m_min forall n$ is not attainable either. We use a superset $M_e={m | m in n.M_"ed"}=union.big M(q_(i-1),n_2.p) supset.eq M_min$ instead. If optimization was possible, we can try narrowing $M_e$ (M extended). 

We compute $f(m_1)$ over $M$, collect all results as $N_a$

How ? As we visit every $n_2$ we compute $f$ within the iteration.

$
exists m_1=m_min in M => f(m_1) in S \
$

$
(forall n => phi(n)) => \ forall n, n.m_min (|q|-a,|n|-b) ="ed"(q,n)= n.m_min."ed" + max(|q|-a-m.i,|q|-b-|n|) <= tau => n.m_min."ed" <= tau \ 
a,b in bb(N) \
max(|q|-a-m.i,|q|-b-|n|) in [0,+infinity) \
tack M_e subset.eq {m | m."ed"<=tau} \
"for every" "ed"(a,b)="eq"(q,n), "matching set of the minimal ed is denoted as" n.M_min \
tack M_e subset.eq {m | exists n in S => m in n.M_min} \
"We revise" M_e = union.big_(n in S) M(q_(i-1),n.p) sect {m | m."ed"<=tau}
=> forall m in M_e => m.i<=i-1 => P(q_(i-1),tau) \
forall m_1 (|q|-1,|n|-1)>m_2 (|q|-1,|n|-1) \ => m_1 "can be ditched as certainly" m_1 != n.m_min "but we will encounter" n.m_min "eventually" \
forall m in S => phi => "ed"(q_i,n)<=tau \
"as a part of" m={q_i,n} => m.i = i\
m."ed" = "ed"(q_i,n) <= tau - 1 => m."ed" <= tau \
// below is useless
// "ed"(q_i,n) <= "ed"(q_(i-1),n) + 1 \
// "ed"(q_(i-1),n) <= "ed"(q_(i),n) + 1  => "ed"(q_(i),n) in ["ed"(q_(i-1),n) -1,"ed"(q_(i-1),n) + 1] \
// forall m in S => phi => "ed"(q_(i-1),n) -1 <= "ed"(q_i,n)<=tau -1 => "ed"(q_(i-1),n) <= tau
$

$S subset.eq N_a$ where we don't know $S$, but we have $N_a$

$
forall n in N_a - S => alpha(n) \
"in contrast": forall n in S => phi(n)
$

What's the difference between $S$ and $N_a$ ?

$
forall n, phi(n) => n in S subset.eq N_a  \
"it means S is exhaustive over certain elements while" N_a - S "means nothing"
$

The strprox iterates over a very large $M_e$ 

The exhaustiveness from $P_1$ to $P_2$ has been proved.

=== Clarify $n.m_min$

x, y for integer. a, b for string.

$
alpha = m_1(|q|-1,|n_2|-1) <= tau space "proposition" \
"generalized:" alpha = m_1(x,y) <=tau \
"ed" = {a,b,M(a,b)} \
n.m_min = {m,x,y} "so we have" m(x,y)="ed"(q,n) "when"
m = m <- min_(m in M(q,n)) m(|q|,|n|) \
n.m_min (x,y) = "ed"(q,n) \
$

ED equivalents

$
m(x,y)="ed"(q,n) = "ed"(q_k,n_b) \
"as we don't increase k,b than |q|,|n|" \
M(q_k,n_b) subset.eq M(q,n) \
forall m in M(q_k,n_b) => m(k,b) <= m(|q|,|n|)
$

In implementation, all nodes are computed the same way. 

$alpha = n.m_min (x,y) <= tau$ x,y as associated with $n.m_min$

$
forall n => n.m_min .x=|q|-1, y=|n|-1
$

Over $x,y$, $m_min$ computes to ed. ${m,x,y}$ is hence the complete definition of $n.m_min$.

$
M_1={m | m in n.M_min} = union.big_(n in S) M(q_(i-1),n.p)
=> forall m in M_1 => m.i<=i-1 => m in P(q_(i-1),tau)
$

== $P_2 -> P_3= P(q_i,b)$

We just need $P_4=P_3-P_2=$ exact b-matchings $=>m."ed"=b$.

Now, we are considering the $P_4$ as if we already have it. 

We consider every $m in P_4$ 
$
m=(q_i,s_j), s[i]=_("by def")q[j] => m."ed"="ed"(q_i,s_j)="ed"(q_(i-1),s_(j-1)) \
=> exists m_1 := m_min^(m in M(q_(i-1),s_(j-1))), m_1(i-1,j-1)=m."ed"  \
=> m_1."ed"<=m_1(i-1,j-1)=m_1."ed"+max(...)=m."ed"=b \
=> cases(
  m_1."ed"=b => m_1 in P_4,
  m_1."ed"<b => m_1 in P_2
)
$ 

Every $m in P_4$ has an $m_min in P_4 union P_2$ as a "handle", reachable from it, denote it as $h(m)$

$
forall m_1 in P_4 => m_1 in union.big_(m in P_4 union P_2) m.D \
forall m_1 in P_4 => h(m) in P_4 union P_2 \
h(h(m)) in P_4 union P_2
$

Consider, for some $m in P_4$, $h(m) in P_4 or h(m) P_2$

Let $h^k (m) := underbracket(h(h(m)),k "times"), k in NN$. 

For some m, $forall k != x => h^k (m) != h^x (m)$, as it's a tree.

If for some m, $forall k => h^k (m) in P_4$, it means $P_4$ has infinitely many elements, which is false.
Therefore $exists k, h^k (m) in P_2$

The calculation requires a different algorithm. 


$
  P_4={m={q_i,s_j}={i,n,"ed"} | forall m_1 in P_2, m in m_1.D, m.n."char" = q[m.i] 
  \ and m_1.i < m.i "as" m_1 "is expected to be" m_min "of" m <== i_1<=i-1=>i_1<i \
    and m(i-1,j-1)=b and beta(m)
  } \
  beta(m) = forall "ed" exists.not m_2={m_2.i,m_2.j,m_2."ed"} in P_2 \
  q[m.i]=n "while no such" m in P_2 "which is exhaustive over ed" <= b-1 
  =>  "ed"(q_i,n) > b-1 \
  m(i-1,j-1)=b "reveals one upper bound, so" "ed"(q_i,n) <= b \
  "Therefore, " m(i-1,j-1)=b and beta(m) => "ed"(q_i,n)=b
$

=== Optimization: $|n_2|$

$m_1$ is known.

$
m_1(i-1,j-1)=m."ed"=b=>
m_1(i-1,j-1) <= b
=> cases(
  j-1-|n_1|<=b => m.j <= |n_1|+b+1,
  i-1-i_1<=b => m.i <=b+1+i_1
) 
$


$
m_1(i-1,j-1)=m_1."ed"+max(i-1-i_1,j-1-|n_1|)=b \
max(i-1-i_1,j-1-|n_1|)=b-m_1."ed" \
"either is true" => "in either case the other is false" \
i-1-i_1 = b-m_1."ed" => i=b+1+i_1 -m_1."ed" \
j=b+1+|n_1|-m_1."ed"
$ 

$
"left" cases(
  i=b+1+i_1 -m_1."ed"=k+1+i_1,
  j <= b+1+|n_1|,
  j-1-|n_1|<b-m_1."ed"\ => j<b+1+|n_1|-m_1."ed"=k+1+|n_1|
) 
"right" cases(
  j=b+1+|n_1|-m_1."ed"=k+1+|n_1|,
  i<b+1+i_1 -m_1."ed"=k+1+i_1
) \
"third" i-1-i_1=j-1-|n_1|=b-m_1."ed"=k => cases(
  i=k+1+i_1,
  j=k+1+|n_1|,
  i-i_1=j-|n_1| = k+1
) \

\
P_4 = "left" union "right" union "third" = cases(
  i=k+1+i_1,
  j<k+1+|n_1|
) union cases(
  i<k+1+i_1,
  j=k+1+|n_1|
) union cases(
  i=k+1+i_1,
  j=k+1+|n_1|
) \
i>i_1 and j>|n_1|
$

== Streaming Query

The use of $P(i,b)$ comes at a cost. It requires exhaustiveness, which may demand heavy computation.

$
P(i-1,1) ->_1 P(i,0) ->_2 P(i,1) ->_2 P(i,n), n in NN \
P(i,1) ->_1 P(i+1,0)
$

$arrow(2)$ means `SecondDeducing`.

The $arrow(1)$ takes an exhaustive matching set over $q_(i-1)$ for all m with $"ed"<=1$. The produced $P(i,0)$ isn't streamed. You have to produce *the entire set*, and then sort it by dped.

When $b$ is set to a large number, if we only take the first few results from $P$ it does not guarantee small dped.

If we have multiple matchings with equal dped, or multiple $s in S$ with same ed, we don't care which one is taken first.

Weird how in $arrow(1)$ the paper uses a larger than necesary set. 


$
P(i-1,0) ->_1 P(i,0) ->_2 P(i,1) ->_2 P(i,n), n in NN \
P(i,0) ->_1 P(i+1,0)
$

$arrow(1)$ is exhaustive as all $m$ that is needed is in the previous set.

Both $arrow(1)$ and $arrow(2)$ try to use the "ed to $m_min$" trick, and prove that we have visited all $m_min$

"Required set" #sym.arrow "result set"

- Both process need required set to be fulfilled to produce exhaustive result.
- The result set may be extraneous.

The process proposed in paper is, roughly.  

$
P(0,0)={root} ->_1 P(1,0) ->_2 P(1,1) -> "exhausted for" P(1,n)
$

The paper asserts that $P_"prev"=P(0,0)$ to $P$ increase b by at most one. 

$
P(1,1) ->_1 P(2,1) ->_2 P(2,2) -> "exhausted" \
P(2,2) ->_1 P(3,2) ->_2 P(3,3) \
P(i,b) -> P(i+1,b+1) "or" P(i+1,b)
$

It's unclear how it is "exhausted".

Denote all satisfying matches $m$ for $q_i$, where $m_i<=b$, as $D(i,b)$

$
D(i,b)={m | m_i <= b} \
m_i = m."ed"+i-m.i => m.i<=i and m."ed"<=b \
=>  m in P(i,b) \
D(i,b) subset.eq P(i,b) \
exists m | m."ed"=b and m.i<i => m in P(i,b) and m in.not D(i,b)
$

$D(i,b)$ is followed by a set of leaves of upperbound PED to $q_i$ of $b$

For $a,b$, $"ped"(a,b)=k => min_(y<=|b|) "ed"(a,b_y) =k$

Denote an extension over $b$ as $b'$

$
forall a,b' => "ped"(a,b')<= k="ed"(a,b_y)
$

== Cache

Data structures in use

+ a prefix tree of in-cache strings.
+ a btreemap/binaryheap keeping track of the tree leaves. 

Each time we need to clean the cache, the largest element is popped. The tree is pruned up to a node with more than one children starting from the popped leaf, as that node necessarily leads to another un-cleaned leaf.


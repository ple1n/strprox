A matching is an algebraic structure that is kept in the transient execution state.

$
  s[1,"len"]=s="a string" =(emptyset,c_1,c_2,c_3,....) \
  "Matching" := cases(
    i->q[1,i],
    j->s[1,j]->n("char"=s[j],"depth"=j),
    Delta = "ed"(q_i,s_j)
  ) \
  "where" q "is a query string" and s_j "is a specific string", \ "which, given a matching, represents a prefix" \
  s_j < s_k | s_k in S_T "we use partial order to denote prefix/suffix" \
  M_"root" := M_0 = (i=0,j=0,Delta=0) \
  Delta(m):=Delta(q_i, s_j):=Delta_m
$

The algorithm heavily makes use of the notion of matching, and axiomatic assumptions about the distance function.

== Properties the distance funtion in use must satisfy

$
  Delta(a, b) | a, b "are strings" \
  "+" "denotes concatenation"
$


$
  &"Law of sum" & Delta(a)+Delta(b)=Delta(a+b) \
  &"That it is possible to find a matching in (q,s) such that"
  Delta(m) + Delta("remainder") = Delta(a, b)
$

For levenshtein distance, suppose we have a matching M, the remainder string should contain no matchings $=>Delta("remainder")=max(|"either part of the remainder"|)$

M always exists, because we have $M_0$

$
  min(Delta(m) + Delta_max ("remainder")) = Delta(a, b) \
  "iff m is taken at the right most of both strings" \
  Delta(m) + Delta_max ("remainder") := m(|q|,|s|)
$

This is the estimation function based on a matching, and two string lengths.

$
  m(|q|,|s|) = ceil(Delta(q, s)) \
  q, s in SS \
  SS "denotes all possible strings" \
  ceil(f(x)) "denotes an alternative function for f which gives one upper bound of" f(x)
$

The above implies we can derive $Delta(a, b)$ from a matching.

$exists m_0 in M(q,s) => m_0(|q|,|s|)=Delta(q, s)$

while $forall m != m_0 in M(q,s) = ceil(Delta(q, s))$

$
  "prefix edit distance" = m(|q|) = Delta_m + |q| - i_m = ceil("ped"(|q|))
$

again the estimation takes a $|q|$, only the length

$
  "ped"(q,s) =min_(m in M(q,s)) ceil("ped"(|q|))
$

by the same logic, there exists such an $m_0$

== Lemma by Ukkonen

$
  q[i] = s[j] => Delta(q[1,i], s[1,j])=Delta(q[1,i-1], s[j-1]) \
  forall(q_i, s_j) , m_2 = {q_i,s_j,Delta(q_i, s_j)} \
  Delta(m_2)=min_(m in M(q(i-1),s(j-1))) m(i-1,j-1)
  = Delta(q_i, s_j)
$

== To search for new matchings $P(q-1,b) ->_(f_1) P(q,b)$

$
  P(q,b) "is a set defined over string" q "and integer" b \
  P(q,b) <=> m in M(q,s) and Delta_m <= b
$


consider all matchings for a pair $(q,s)$

$
  forall m_0 = (q,s,Delta) => exists M(q,s) != emptyset.rev
$

where M is an ordered set, $exists m in M(q,s) and m(abs(q),abs(s)) = Delta m_0$ and it is considered the minimum.

$P(q,tau)=$ all matchings that has $Delta m<=tau$

for a given $m(abs(q),abs(s))=Delta m +N<=tau$

$m_0$ always exists, according to the original paper

$m_0(abs(q),abs(s))=Delta m_0+N=tau => Delta m_0 <=tau => m_0 in P(q,tau)$

therefore, for a given $m_1 =(q,s,Delta)$, we can find a $m_0 in P(q,tau)$

By lemma Ukkonen

=== Theorem, all searched-for nodes have $m_0$ within the search domain

$
  cases(m_2 = (q_i,s_j), i>=1, j>=1) \
  m_0(i-1,j-1)=Delta m_2 and m_0 in M(q_(-1),s_(-1)) \
$

=== Searching for $m_2$ from known $m_0$

$
  m_2:=(q,n) \
  forall (m_2,m_0)=>m_0(|q|-1,|n|-1)=Delta m_2 \
  tack m_0(|q|-1,|n|-1)<=b <=>Delta m_2<=b
$

$
  P(=q,tau) := {m|m=(q,s) and Delta m <= tau}\
  P(q,tau)-P(=q,tau) = cases(i<=|q|-1, Delta m <=tau) = P(q_(-1),tau) \
  P(q,tau) = sum_( 0<=i<=|q|) P(=q[i],tau)
$

This is written as sum because the sets do not overlap


== $bold(P(q,tau-1) ->_f_2 P(=q,tau))$

In the next algorithm, $f_2$, we try to produce $P(q,tau)$ from $P(q,tau-1)$

Precisely

$
  cases(P(=q,tau-1):={m|m=(q,s) and Delta m <=tau-1}, q "is the current query string", "denote" P tau :=P(=q,tau))
$

*This only produces matchings where $m.i=|q|$ *

with the equation

$
  forall m_2 in P tau => cases(
    delim: "|",
    alpha : Delta m_2 = tau => m_2 in P tau = P(=q,tau) "  (which we don't have yet)",
    beta : Delta m_2 < tau => m_2 in P(=q,tau-1)
  ) \
  forall (m_2,m_1) => Delta m_1 <= tau \ m_1 "being associated mimimum" \
  or cases(
    m_1 in P(q_(-1),tau),
    m_1 in P(q_(-1), tau-1)
  )
$

we can ignore the case of $alpha$

as we can show that, there are eventually matchings $m_3$ that lead up to $m_1$ where $m_1 in alpha$,

and $m_3 in P(q_(-1), tau-1)$



We enumerate over $P(q,tau-1)$, each item is denoted as $m_1$, the descendent nodes are searched.

such that

$
  cases(
    exists m_2,
    m_1(i-1,j-1)=tau,
    exists.not m_1 in P(q,tau-1) <=> Delta m_1 <= tau-1
  )
  =>Delta m_1=tau=Delta m_2
$


the theorem concerns the domain $M(q,s)$

$
  exists.not m in P(q,tau-1) => cases(
    Delta m > tau-1,
    m(abs(q),abs(s))=Delta m +N<=tau =>Delta m =tau and N=0
  )
$

which produces $P tau$

== Logic leading to Top-K answers


$
  ceil("ped"(s,q))=Delta m +|q|-m.i " the paper calls it deduced prefix edit distance" \
  "specifically, when" m.q = q \
  ceil("ped"(s,q)) = Delta m \
  "for" P(q,tau), "every" s in P(q,tau) =>ceil("ped"(s,q)) <= tau \
  tack "ped"(s,q) <= tau \
  cases(
    m=(x,y) \
    m.x=q,
    Delta m = Delta(x, y)
  ) => forall s >y, "ped"(s,q)<=Delta m
$

which means, for a certain threshold $tau$, if $exists m=(q,s)$, then $m in P(q,tau)$

(matching with last character of query, with any string in tree)

we can then find _all_ strings with this matching up to $tau$.

=== Completeness of results

Given $(q,s,b)$, the domain of $Delta m_0,k$ can be solved

$
  forall "ped"(q,s)<=b =>
  "ped"(q,s) = m_0(|q|) = Delta m_0 + |q|- |m.q| <=b \ =>
  cases(Delta m_0<=b=>m_0 in P(q,b), 0<=Delta m_0, k=|q|-|m.q|<=b)
$

according to the theorem in the paper, every PED has an associated minimal matching, $m_0$

For each $k->P(|q|-k,b)$,
$
  forall "ped"(q,s)<=b =>
  "ped"(q,s) = m_0(|q|) = Delta m_0 + k <=b \ =>
  0<=Delta m_0<=b -k
$

$m.q=q$ is therefore a special case.

$
  k=0=> 0<=Delta m_0<=b
$


== Algorithmic optimization


In solving the _deduced edit distance_ equation


$
  forall m_1 => (forall n_2 in m_1.D => (
      forall q => alpha: m_1(|q|-1,|n_2|-1) <= tau => cases(
        |n_2| in [ |n_1|+1,|n_1|+tau+1 ] \
        i_1 >= |q| - 1 - tau \
        m_1."ed" <= tau
      )
    ))
$


In finding $m$ when $m_1$ is known, which is used in $f_2$, also called _SecondDeducing_

$
  m_1=(i_1,n_1,Delta m_1) " the ancestor node" \
  m=(i,j,Delta m) " the descendent"
$

$
  m_1(i-1,j-1)=Delta m=b=>
  m_1(i-1,j-1) <= b
  => cases(
    j-1-|n_1|<=b => j <= |n_1|+b+1,
    i-1-i_1<=b => i <=b+1+i_1
  )
$


$
  P(q,b) "represents a set of matchings" m=(q,s) \
  forall (q,s) in P(q,b) => Delta(q, s) <=b \
  "as we increase" q -> q_(+1) \
  forall(q_(+1), s) => Delta(q_(+1), s) in {Delta(q, s),Delta(q, s)+1} = {b,b+1} \
  "which means" P(q_(+1),b+1) "is the largest search domain needed" \
  P(q_(+1),b+N) = P(q_(+1),b+1) "when" N>1
$

== Sorting matchings

$
  "query" = (q,b) \
  m=(a,s, Delta) \
  forall m => cases(|a|<=|q|, Delta<=b) \
  "the sorting uses a dimensionality reduction" \
  f =mat(b+1, -1) mat(|a|; Delta m)
$

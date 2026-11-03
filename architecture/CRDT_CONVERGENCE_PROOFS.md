# Mathematical Proofs & Strong Eventual Consistency (SEC) Guarantees

## 1. Join-Semilattice Properties

A state-based Conflict-Free Replicated Data Type (CvRDT) converges to an identical state across all peers if and only if its merge operator $\sqcup$ forms a **Join-Semilattice** $(S, \sqcup)$ ordered by partial order $\le$:

1. **Commutativity:**
   $$x \sqcup y = y \sqcup x \quad \forall x, y \in S$$
   *Proof:* The merge operation is defined by pairwise supremum over component timestamps and counters:
   $$\max(t_x, t_y) = \max(t_y, t_x)$$
   Order of packet arrival over the network does not alter final state.

2. **Associativity:**
   $$(x \sqcup y) \sqcup z = x \sqcup (y \sqcup z) \quad \forall x, y, z \in S$$
   *Proof:* $\max(\max(t_x, t_y), t_z) = \max(t_x, \max(t_y, t_z))$. Multi-hop peer propagation yields the same join regardless of forwarding topologies.

3. **Idempotence:**
   $$x \sqcup x = x \quad \forall x \in S$$
   *Proof:* $\max(t_x, t_x) = t_x$. Duplicate packet delivery and retransmissions produce zero state drift.

---

## 2. Last-Write-Wins (LWW) Deterministic Tie-Breaking

For an element $e$, let $R_{\text{add}}(e) = (t_a, n_a)$ and $R_{\text{rem}}(e) = (t_r, n_r)$ where $t \in \mathbb{N}$ is a millisecond timestamp and $n \in \Sigma^*$ is a unique peer identifier.

We define strict total order $\succ$ over records:
$$(t_1, n_1) \succ (t_2, n_2) \iff (t_1 > t_2) \lor (t_1 = t_2 \land n_1 >_{\text{lex}} n_2)$$

An element $e$ belongs to the active set if:
$$e \in \text{Add-Set} \land (e \notin \text{Remove-Set} \lor R_{\text{add}}(e) \succ R_{\text{rem}}(e))$$

Because $\succ$ is a strict total order, concurrent additions and deletions at the exact same physical millisecond resolve deterministically across all replicas without requiring consensus locks.

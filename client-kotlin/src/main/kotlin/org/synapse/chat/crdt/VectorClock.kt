package org.synapse.chat.crdt

enum class CausalOrder {
    EQUAL,
    HAPPENED_BEFORE,
    HAPPENED_AFTER,
    CONCURRENT
}

class VectorClock(
    val clock: MutableMap<String, Long> = mutableMapOf()
) {
    fun increment(nodeId: String): Long {
        val next = (clock[nodeId] ?: 0L) + 1L
        clock[nodeId] = next
        return next
    }

    fun get(nodeId: String): Long = clock[nodeId] ?: 0L

    fun set(nodeId: String, counter: Long) {
        clock[nodeId] = counter
    }

    /**
     * Join-semilattice merge: max(V_A[k], V_B[k])
     */
    fun merge(other: VectorClock) {
        for ((node, counter) in other.clock) {
            val current = clock[node] ?: 0L
            clock[node] = maxOf(current, counter)
        }
    }

    fun compare(other: VectorClock): CausalOrder {
        var selfGreater = false
        var otherGreater = false

        val allKeys = clock.keys + other.clock.keys

        for (key in allKeys) {
            val s = get(key)
            val o = other.get(key)
            if (s > o) selfGreater = true
            else if (o > s) otherGreater = true
        }

        return when {
            !selfGreater && !otherGreater -> CausalOrder.EQUAL
            selfGreater && !otherGreater -> CausalOrder.HAPPENED_AFTER
            !selfGreater && otherGreater -> CausalOrder.HAPPENED_BEFORE
            else -> CausalOrder.CONCURRENT
        }
    }
}

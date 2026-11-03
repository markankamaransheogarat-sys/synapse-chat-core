package org.synapse.chat.crdt

import org.synapse.chat.model.ChatMessage
import org.synapse.chat.model.SyncState

data class LWWRecord(
    val message: ChatMessage,
    val timestamp: Long,
    val nodeId: String
) {
    fun dominates(other: LWWRecord): Boolean {
        return if (this.timestamp != other.timestamp) {
            this.timestamp > other.timestamp
        } else {
            this.nodeId > other.nodeId
        }
    }
}

class LWWMessageSet {
    val addSet = mutableMapOf<String, LWWRecord>()
    val removeSet = mutableMapOf<String, LWWRecord>()

    fun add(message: ChatMessage, nodeId: String) {
        val record = LWWRecord(message, message.timestamp, nodeId)
        val existing = addSet[message.id]
        if (existing == null || record.dominates(existing)) {
            addSet[message.id] = record
        }
    }

    fun remove(messageId: String, timestamp: Long, nodeId: String) {
        val dummy = ChatMessage(messageId, "", nodeId, "", timestamp, isDeleted = true)
        val record = LWWRecord(dummy, timestamp, nodeId)
        val existing = removeSet[messageId]
        if (existing == null || record.dominates(existing)) {
            removeSet[messageId] = record
        }
    }

    fun contains(messageId: String): Boolean {
        val addRec = addSet[messageId] ?: return false
        val remRec = removeSet[messageId] ?: return true
        return addRec.dominates(remRec)
    }

    fun getActiveMessages(): List<ChatMessage> {
        return addSet.filter { (id, _) -> contains(id) }
            .values
            .map { it.message }
            .sortedBy { it.timestamp }
    }

    fun merge(other: LWWMessageSet) {
        for ((id, otherRec) in other.addSet) {
            val current = addSet[id]
            if (current == null || otherRec.dominates(current)) {
                addSet[id] = otherRec
            }
        }
        for ((id, otherRec) in other.removeSet) {
            val current = removeSet[id]
            if (current == null || otherRec.dominates(current)) {
                removeSet[id] = otherRec
            }
        }
    }
}

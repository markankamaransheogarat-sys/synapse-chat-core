package org.synapse.chat.model

enum class SyncState {
    PENDING_LOCAL, // Stored in local SQLite while offline
    SENDING,       // Socket frame dispatched, awaiting ACK
    SYNCED,        // Confirmed by cluster & CRDT merged
    FAILED         // Retries exhausted
}

data class ChatMessage(
    val id: String,
    val channelId: String,
    val senderId: String,
    val content: String,
    val timestamp: Long,
    val syncState: SyncState = SyncState.PENDING_LOCAL,
    val isDeleted: Boolean = false
)

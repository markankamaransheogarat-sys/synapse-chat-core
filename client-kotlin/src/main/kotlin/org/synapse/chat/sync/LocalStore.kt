package org.synapse.chat.sync

import org.synapse.chat.model.ChatMessage

interface LocalStore {
    suspend fun saveMessage(message: ChatMessage)
    suspend fun getPendingMessages(): List<ChatMessage>
    suspend fun markMessageSynced(messageId: String)
    suspend fun deleteMessage(messageId: String)
    suspend fun getAllChannelMessages(channelId: String): List<ChatMessage>
}

class InMemoryLocalStore : LocalStore {
    private val messages = mutableMapOf<String, ChatMessage>()

    override suspend fun saveMessage(message: ChatMessage) {
        messages[message.id] = message
    }

    override suspend fun getPendingMessages(): List<ChatMessage> {
        return messages.values.filter { it.syncState == org.synapse.chat.model.SyncState.PENDING_LOCAL }
    }

    override suspend fun markMessageSynced(messageId: String) {
        messages[messageId]?.let {
            messages[messageId] = it.copy(syncState = org.synapse.chat.model.SyncState.SYNCED)
        }
    }

    override suspend fun deleteMessage(messageId: String) {
        messages[messageId]?.let {
            messages[messageId] = it.copy(isDeleted = true)
        }
    }

    override suspend fun getAllChannelMessages(channelId: String): List<ChatMessage> {
        return messages.values.filter { it.channelId == channelId && !it.isDeleted }
            .sortedBy { it.timestamp }
    }
}

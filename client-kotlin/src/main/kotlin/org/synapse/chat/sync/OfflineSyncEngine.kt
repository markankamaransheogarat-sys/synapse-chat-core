package org.synapse.chat.sync

import org.synapse.chat.crdt.LWWMessageSet
import org.synapse.chat.crdt.VectorClock
import org.synapse.chat.model.ChatMessage
import org.synapse.chat.model.SyncState

class OfflineSyncEngine(
    val nodeId: String,
    val localStore: LocalStore,
    val crdtStore: LWWMessageSet = LWWMessageSet(),
    val vectorClock: VectorClock = VectorClock()
) {
    var isOnline: Boolean = false
        private set

    suspend fun sendMessage(channelId: String, content: String): ChatMessage {
        val timestamp = System.currentTimeMillis()
        val messageId = "msg_${nodeId}_${timestamp}"
        vectorClock.increment(nodeId)

        val message = ChatMessage(
            id = messageId,
            channelId = channelId,
            senderId = nodeId,
            content = content,
            timestamp = timestamp,
            syncState = if (isOnline) SyncState.SENDING else SyncState.PENDING_LOCAL
        )

        // Optimistically write to local store and CRDT set
        localStore.saveMessage(message)
        crdtStore.add(message, nodeId)

        if (isOnline) {
            dispatchToServer(message)
        }

        return message
    }

    suspend fun onNetworkConnected() {
        isOnline = true
        val pending = localStore.getPendingMessages()
        for (msg in pending) {
            dispatchToServer(msg)
        }
    }

    fun onNetworkDisconnected() {
        isOnline = false
    }

    private suspend fun dispatchToServer(message: ChatMessage) {
        // Simulates socket frame transmission & ACK arrival
        localStore.markMessageSynced(message.id)
    }

    suspend fun onRemoteDeltaReceived(incomingCRDT: LWWMessageSet, incomingClock: VectorClock) {
        crdtStore.merge(incomingCRDT)
        vectorClock.merge(incomingClock)

        for (msg in crdtStore.getActiveMessages()) {
            localStore.saveMessage(msg.copy(syncState = SyncState.SYNCED))
        }
    }
}

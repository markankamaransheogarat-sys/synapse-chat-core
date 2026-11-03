use synapse_chat_core::{CausalOrder, LWWElementSet, PNCounter, VectorClock};

#[test]
fn test_vector_clock_causality() {
    let mut v1 = VectorClock::new();
    let mut v2 = VectorClock::new();

    v1.set("A", 1);
    v1.set("B", 0);

    v2.set("A", 1);
    v2.set("B", 1);

    // v1 is ancestor of v2 (v1 < v2)
    assert_eq!(v1.compare(&v2), CausalOrder::HappenedBefore);
    assert_eq!(v2.compare(&v1), CausalOrder::HappenedAfter);

    // Concurrent conflict: A wrote on v1, B wrote on v2
    v1.set("A", 2); // v1: {A:2, B:0}
                    // v2: {A:1, B:1}
    assert_eq!(v1.compare(&v2), CausalOrder::Concurrent);
    assert_eq!(v2.compare(&v1), CausalOrder::Concurrent);
}

#[test]
fn test_vector_clock_semilattice_properties() {
    let mut a = VectorClock::new();
    a.set("node_1", 5);
    a.set("node_2", 2);

    let mut b = VectorClock::new();
    b.set("node_1", 3);
    b.set("node_2", 8);
    b.set("node_3", 1);

    // Idempotence: a merge a == a
    let mut a_copy = a.clone();
    a_copy.merge(&a);
    assert_eq!(a, a_copy);

    // Commutativity: a merge b == b merge a
    let mut m1 = a.clone();
    m1.merge(&b);

    let mut m2 = b.clone();
    m2.merge(&a);

    assert_eq!(m1, m2);
    assert_eq!(m1.get("node_1"), 5);
    assert_eq!(m1.get("node_2"), 8);
    assert_eq!(m1.get("node_3"), 1);
}

#[test]
fn test_lww_element_set_message_edits() {
    let mut set = LWWElementSet::new();

    // Node A posts message at t=100
    set.add("msg_1", "Original text", 100, "node_A");
    assert_eq!(set.get("msg_1").unwrap().content, "Original text");

    // Node B edits message at t=150
    set.add("msg_1", "Edited by B", 150, "node_B");
    assert_eq!(set.get("msg_1").unwrap().content, "Edited by B");

    // Stale edit from Node A arriving late at t=120 should be rejected
    set.add("msg_1", "Stale text", 120, "node_A");
    assert_eq!(set.get("msg_1").unwrap().content, "Edited by B");
}

#[test]
fn test_lww_element_set_tombstone_deletion() {
    let mut set = LWWElementSet::new();

    set.add("msg_1", "Hello world", 100, "node_A");
    assert!(set.contains("msg_1"));

    // User deletes message at t=110
    set.remove("msg_1", 110, "node_A");
    assert!(!set.contains("msg_1"));
    assert!(set.get("msg_1").is_none());

    // Concurrent merge from offline client containing the old add at t=100
    let mut offline_replica = LWWElementSet::new();
    offline_replica.add("msg_1", "Hello world", 100, "node_A");

    set.merge(&offline_replica);
    // Tombstone at t=110 must still dominate add at t=100
    assert!(!set.contains("msg_1"));
}

#[test]
fn test_pn_counter_reactions() {
    let mut c1 = PNCounter::new();
    let mut c2 = PNCounter::new();

    // Node A reacts (+1)
    c1.increment("node_A", 1);
    // Node B reacts (+1)
    c2.increment("node_B", 1);

    // Node A removes reaction (-1)
    c1.decrement("node_A", 1);

    // Merge distributed counters
    c1.merge(&c2);
    assert_eq!(c1.value(), 1); // 0 + 1 = 1
}

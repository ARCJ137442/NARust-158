//! # NARS中有关「存储」的内容
//!
//! # Storage management (原OpenNARS 1.5.8文档)
//!
//! All Items (Concept within Memory, TaskLinks and TermLinks within Concept, and Tasks within buffer) are put into Bags, which supports priority-based resources allocation. Also, bag supports access by key (String).
//!
//! A bag supports three major operations:
//!
//! - To take out an item by key.
//! - To take out an item probabilistically according to priority.
//! - To put an item into the bag.
//!
//! All the operations take constant time to finish.
//!
//! The "take out by priority" operation takes an item out probabilistically, with the probability proportional to the priority value.
//!
//! The probability distribution is generated from a deterministic table.
//!
//! All classes in package `nars.storage` extend `Bag`.
//!
//! In NARS, the memory consists of a bag of concepts. Each concept uniquely corresponds to a term, which uniquely corresponds to a String served as its name. It is necessary to separate a term and the corresponding concept, because a concept may be deleted due to space competition, and a term is removed only when no other term is linked to it. In the system, there may be multiple terms refer to the same concept, though the concept just refer to one of them. NARS does not follow a "one term, one concept" policy and use a hash table in memory to maps names into terms, because the system needs to remove a concept without removing the term that naming it.
//!
//! Variable terms correspond to no concept, and their meaning is local to the "smallest" term that contains all occurrences of the variable.
//!
//! From name to term, call Term.nameToTerm(String). From name to concept, call Concept.nameToConcept(String). Both use the name as key to get the concept from the concept hash table in memory.
//!
//! The main memory also contains buffers for new tasks. One buffer contains tasks to be processed immediately (to be finished in constant time), and the other, a bag, for the tasks to be processed later.

// 分派器
pub mod distributor;

// 包
pub mod bag;

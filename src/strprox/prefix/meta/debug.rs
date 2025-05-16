/// Structures to help with debugging
///
///

use std::collections::BTreeMap;

use super::*;


struct DebugInfo {
    m_source: BTreeMap<Matching<UUU>, MatchingContext>,
}

struct MatchingContext {
    lead: MatchingNode<UUU>,
    derive_func: MatchingDerivor
}

struct MatchingDerivor {

}

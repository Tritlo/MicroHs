//! Optional profiling payload attached to strict stack frames.
#[cfg(feature = "profile")]
use super::*;

#[cfg(feature = "profile")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::runtime) struct ProfileHead(pub(in crate::runtime) Option<NodeId>);

#[cfg(not(feature = "profile"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::runtime) struct ProfileHead;

impl ProfileHead {
    #[inline]
    pub(in crate::runtime) fn none() -> Self {
        #[cfg(feature = "profile")]
        {
            Self(None)
        }
        #[cfg(not(feature = "profile"))]
        {
            Self
        }
    }

    #[cfg(feature = "profile")]
    #[inline]
    pub(in crate::runtime) fn from_node(id: NodeId) -> Self {
        Self(Some(id))
    }

    #[cfg(feature = "profile")]
    #[inline]
    pub(in crate::runtime) fn node(self) -> Option<NodeId> {
        self.0
    }
}

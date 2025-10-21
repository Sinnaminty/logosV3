use crate::types::{Mimic, MimicUser};

impl MimicUser {
    /// adds this Mimic to the mimics member variable of this user's MimicUser struct.
    pub fn add_mimic(&mut self, mimic: Mimic) {
        self.mimics.push(mimic);
    }
}

mod modname {}

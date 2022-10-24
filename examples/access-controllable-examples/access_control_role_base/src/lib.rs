use near_plugins::AccessControlRole;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(AccessControlRole, Clone, Copy)]
pub enum Positions {
    LevelA,
    LevelB,
    LevelC
}

#[cfg(test)]
mod tests {
    use near_plugins::AccessControlRole;
    use crate::Positions;

    #[test]
    fn base_scenario() {
        let role: Positions = Positions::LevelA;

        assert_eq!(Positions::acl_super_admin_permission(), 1);
        assert_eq!(role.acl_permission(), 1 << 1);
        assert_eq!(role.acl_admin_permission(), 1 << 2);

        //https://docs.rs/bitflags/latest/bitflags/
        assert_eq!(crate::RoleFlags::LEVELA.bits, role.acl_permission() as u128);
    }
}

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AdminNotSet = 1,
    UnknownClientType = 2,
    ClientIdNotFound = 3,
    CounterpartyAlreadyRegistered = 4,
    ClientFrozen = 5,
    PortAlreadyRegistered = 6,
    InvalidIdentifierLength = 7,
    InvalidIdentifierPrefix = 8,
    InvalidIdentifierChar = 9,
}

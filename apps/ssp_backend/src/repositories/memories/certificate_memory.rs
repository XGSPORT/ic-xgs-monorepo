use ic_stable_structures::BTreeMap;

use crate::repositories::{
    Certificate, CertificateId, CertificateManagedUserIdKey, CertificateUserPrincipalKey,
};

use super::{
    Memory, CERTIFICATE_MANAGED_USER_ID_INDEX_MEMORY_ID, CERTIFICATE_MEMORY_ID,
    CERTIFICATE_USER_PRINCIPAL_INDEX_MEMORY_ID, MEMORY_MANAGER,
};

pub type CertificateMemory = BTreeMap<CertificateId, Certificate, Memory>;
pub type CertificateUserPrincipalIndexMemory =
    BTreeMap<CertificateUserPrincipalKey, CertificateId, Memory>;
pub type CertificateManagedUserIdIndexMemory =
    BTreeMap<CertificateManagedUserIdKey, CertificateId, Memory>;

pub fn init_certificates() -> CertificateMemory {
    BTreeMap::init(get_certificates_memory())
}

pub fn init_certificate_user_principal_index() -> CertificateUserPrincipalIndexMemory {
    BTreeMap::init(get_certificate_user_principal_index_memory())
}

pub fn init_certificate_managed_user_id_index() -> CertificateManagedUserIdIndexMemory {
    BTreeMap::init(get_certificate_managed_user_id_index_memory())
}

fn get_certificates_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(CERTIFICATE_MEMORY_ID))
}

fn get_certificate_user_principal_index_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(CERTIFICATE_USER_PRINCIPAL_INDEX_MEMORY_ID))
}

fn get_certificate_managed_user_id_index_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(CERTIFICATE_MANAGED_USER_ID_INDEX_MEMORY_ID))
}

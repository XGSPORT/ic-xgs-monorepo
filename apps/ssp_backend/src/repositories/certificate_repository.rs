use std::cell::RefCell;

use candid::Principal;
use ic_cdk::println;
use ic_certification::{labeled, labeled_hash, AsHashTree, Hash, RbTree};
use ic_stable_structures::Storable;
use serde::Serialize;
use serde_cbor::Serializer;

const SSP_CERTIFICATES_TREE_LABEL: &[u8] = b"ssp_certificates";

use super::{
    init_certificate_managed_user_id_index, init_certificate_user_principal_index,
    init_certificates, Certificate, CertificateId, CertificateManagedUserIdIndexMemory,
    CertificateManagedUserIdKey, CertificateManagedUserIdRange, CertificateMemory,
    CertificateUserPrincipalIndexMemory, CertificateUserPrincipalKey,
    CertificateUserPrincipalRange, Uuid,
};

/// SSP certificates tree structure:
/// ssp_certificates
/// └── <user_principal>
///     └── <certificate_id>
///         └── certificate cbor data hash
type IcCertificateTree = RbTree<Principal, RbTree<CertificateId, Hash>>;

struct CertificateState {
    certificates: CertificateMemory,
    certificate_user_principal_index: CertificateUserPrincipalIndexMemory,
    certificate_managed_user_id_index: CertificateManagedUserIdIndexMemory,
    ic_certificate_tree: IcCertificateTree,
}

impl Default for CertificateState {
    fn default() -> Self {
        Self {
            certificates: init_certificates(),
            certificate_user_principal_index: init_certificate_user_principal_index(),
            certificate_managed_user_id_index: init_certificate_managed_user_id_index(),
            ic_certificate_tree: RbTree::new(),
        }
    }
}

thread_local! {
    static STATE: RefCell<CertificateState> = RefCell::new(CertificateState::default());
}

pub struct UserCertificateWithCertification {
    pub certificate: Option<Certificate>,
    pub ic_certificate: Vec<u8>,
    pub ic_certificate_witness: Vec<u8>,
}

fn ic_certificate() -> Vec<u8> {
    ic_cdk::api::data_certificate().expect("No data certificate available")
}

#[derive(Default)]
pub struct CertificateRepository {}

impl CertificateRepository {
    pub fn get_certificate(&self, id: &CertificateId) -> UserCertificateWithCertification {
        match STATE.with_borrow(|s| s.certificates.get(id)) {
            Some(certificate) => {
                let ic_certificate_witness =
                    self.certificate_witness(&certificate.user_principal, Some(id));

                UserCertificateWithCertification {
                    certificate: Some(certificate),
                    ic_certificate: ic_certificate(),
                    ic_certificate_witness,
                }
            }
            None => UserCertificateWithCertification {
                certificate: None,
                ic_certificate: vec![],
                ic_certificate_witness: vec![],
            },
        }
    }

    pub fn get_certificates_by_user_principal(
        &self,
        user_principal: &Principal,
    ) -> Result<Vec<(CertificateId, Certificate)>, String> {
        let range = CertificateUserPrincipalRange::new(*user_principal)?;

        let certificates = STATE.with_borrow(|s| {
            s.certificate_user_principal_index
                .range(range)
                .map(|(_, id)| (id, s.certificates.get(&id).unwrap()))
                .collect()
        });

        Ok(certificates)
    }

    pub fn get_certificates_by_managed_user_id(
        &self,
        managed_user_id: &Uuid,
    ) -> Result<Vec<(CertificateId, Certificate)>, String> {
        let range = CertificateManagedUserIdRange::new(*managed_user_id)?;

        let certificates: Vec<_> = STATE.with_borrow(|s| {
            s.certificate_managed_user_id_index
                .range(range)
                .map(|(_, id)| (id, s.certificates.get(&id).unwrap()))
                .collect()
        });

        Ok(certificates)
    }

    pub async fn create_certificate(
        &self,
        certificate: Certificate,
    ) -> Result<CertificateId, String> {
        let id = CertificateId::new().await?;
        let user_principal = certificate.user_principal;
        let user_principal_key = CertificateUserPrincipalKey::new(user_principal, id)?;

        STATE.with_borrow_mut(|s| {
            s.certificates.insert(id, certificate.clone());
            s.certificate_user_principal_index
                .insert(user_principal_key, id);

            if let Some(managed_user_id) = certificate.managed_user_id {
                let managed_user_id_key = CertificateManagedUserIdKey::new(managed_user_id, id)?;

                s.certificate_managed_user_id_index
                    .insert(managed_user_id_key, id);
            }
            self.certify_certificate_data(&mut s.ic_certificate_tree, id, certificate);

            Ok::<(), String>(())
        })?;

        self.set_certified_data();

        Ok(id)
    }

    pub fn certify_all_certificates(&self) {
        let count = STATE.with_borrow_mut(|s| {
            s.certificates.iter().for_each(|(id, certificate)| {
                self.certify_certificate_data(&mut s.ic_certificate_tree, id, certificate);
            });
            s.certificates.len()
        });
        self.set_certified_data();
        println!("Certified {} certificates", count);
    }

    fn certify_certificate_data(
        &self,
        ic_certificate_tree: &mut IcCertificateTree,
        id: CertificateId,
        certificate: Certificate,
    ) {
        let user_principal = certificate.user_principal;
        let user_principal_bytes = user_principal.to_bytes();
        match ic_certificate_tree.get(&user_principal_bytes) {
            Some(_) => {
                ic_certificate_tree.modify(&user_principal_bytes, |inner| {
                    inner.insert(id, certificate.root_hash())
                });
            }
            None => {
                let mut tree = RbTree::new();
                tree.insert(id, certificate.root_hash());
                ic_certificate_tree.insert(user_principal, tree);
            }
        }
    }

    fn set_certified_data(&self) {
        STATE.with_borrow(|s| {
            let tree_hash = s.ic_certificate_tree.root_hash();
            let root_hash = labeled_hash(SSP_CERTIFICATES_TREE_LABEL, &tree_hash);
            ic_cdk::api::set_certified_data(&root_hash);
        });
    }

    fn certificate_witness(
        &self,
        user_principal: &Principal,
        certificate_id: Option<&CertificateId>,
    ) -> Vec<u8> {
        STATE.with_borrow(|s| {
            let witness = match certificate_id {
                Some(id) => s
                    .ic_certificate_tree
                    .nested_witness(user_principal.as_ref(), |inner| inner.witness(id.as_ref())),
                None => s
                    .ic_certificate_tree
                    .nested_witness(user_principal.as_ref(), |inner| inner.keys()),
            };
            let tree = labeled(SSP_CERTIFICATES_TREE_LABEL, witness);

            let mut data = vec![];
            let mut serializer = Serializer::new(&mut data);
            serializer.self_describe().unwrap();
            tree.serialize(&mut serializer).unwrap();

            data
        })
    }
}
